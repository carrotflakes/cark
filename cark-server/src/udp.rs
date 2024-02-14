use std::net::{SocketAddr, UdpSocket};

use cark_common::{
    model::{ClientUdpMessage, ServerUdpMessage},
    udp_stat::{Sequence, SequenceGen, UdpStat},
};

use crate::{IncomingEvent, OutgoingEvent};

pub struct Udp {
    socket: UdpSocket,
    connections: Vec<Connection>, // TODO: Remove
    outgoing_events: Vec<OutgoingEvent>,
}

impl Udp {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            connections: vec![],
            outgoing_events: vec![],
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn push_event(&mut self, event: OutgoingEvent) {
        self.outgoing_events.push(event);
    }

    pub fn process(&mut self, mut handler: impl FnMut(IncomingEvent)) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        // Receive
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let message = &buf[..size];
                    log::debug!("Received {:?} from {}", message, addr);

                    let message: ClientUdpMessage = cark_common::read_from_slice(message).unwrap();
                    log::debug!("Received {:?}", message);

                    match message {
                        ClientUdpMessage::Init { id } => {
                            log::info!("Client connected, addr: {}, id: {}", addr, id);

                            self.connections.retain(|c| {
                                let detected = c.addr == addr;
                                if detected {
                                    // TODO
                                    log::info!("Client reconnected, addr: {}, id: {}", addr, id);
                                }
                                !detected
                            });

                            self.connections.push(Connection::new(id, addr));
                        }
                        ClientUdpMessage::Message { sequence, message } => {
                            let connection = self
                                .connections
                                .iter_mut()
                                .find(|c| c.addr == addr)
                                .expect("Unknown address");

                            connection.update(sequence);

                            handler(IncomingEvent {
                                connection_id: connection.id,
                                sequence,
                                message,
                            });
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        // Send
        for event in self.outgoing_events.drain(..) {
            if let Some(id) = event.connection_id {
                let connection = self
                    .connections
                    .iter_mut()
                    .find(|c| c.id == id)
                    .expect("Unknown id");

                let message = ServerUdpMessage::Message {
                    sequence: connection.sequence.next(),
                    message: event.message,
                };
                let buf = cark_common::write_to_slice(&message, &mut buf).unwrap();

                self.socket.send_to(&buf, connection.addr)?;
            } else {
                // Broadcast
                for connection in &mut self.connections {
                    let message = ServerUdpMessage::Message {
                        sequence: connection.sequence.next(),
                        message: event.message.clone(),
                    };
                    let buf = cark_common::write_to_slice(&message, &mut buf).unwrap();
                    self.socket.send_to(&buf, connection.addr)?;
                }
            }
        }

        Ok(())
    }

    pub fn log_stat(&self) {
        for connection in &self.connections {
            log::info!(
                "Connection: id={}, addr={}, loss={:.2}%", //, rtt={:?}ms
                connection.id,
                connection.addr,
                connection.stat.loss_rate() * 100.0,
            );
        }
    }
}

pub struct Connection {
    id: u64,
    addr: SocketAddr,
    stat: UdpStat,
    sequence: SequenceGen,
}

impl Connection {
    pub fn new(id: u64, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            stat: UdpStat::new(),
            sequence: SequenceGen::default(),
        }
    }

    pub fn update(&mut self, sequence: Sequence) {
        self.stat.update(sequence);
    }
}
