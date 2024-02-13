use std::{collections::HashMap, net::UdpSocket};

use cark_common::model::UdpMessage;

use crate::{IncomingEvent, OutgoingEvent};

pub struct Udp {
    socket: UdpSocket,
    addr_id_map: HashMap<std::net::SocketAddr, u64>,
    id_addr_map: HashMap<u64, std::net::SocketAddr>,
    outgoing_events: Vec<OutgoingEvent>,
}

impl Udp {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            addr_id_map: HashMap::new(),
            id_addr_map: HashMap::new(),
            outgoing_events: vec![],
        })
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
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

                    let message: UdpMessage = cark_common::read_from_slice(message).unwrap();
                    log::debug!("Received {:?}", message);

                    match message {
                        UdpMessage::Init { id } => {
                            log::info!("Client connected, addr: {}, id: {}", addr, id);

                            self.addr_id_map.insert(addr, id);
                            self.id_addr_map.insert(id, addr);
                        }
                        UdpMessage::Message { message } => {
                            let id = *self.addr_id_map.get(&addr).expect("Unknown address");
                            handler(IncomingEvent {
                                connection_id: id,
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
            let buf = cark_common::write_to_slice(&event.message, &mut buf).unwrap();

            if let Some(id) = event.connection_id {
                let addr = self.id_addr_map.get(&id).expect("Unknown id");
                self.socket.send_to(&buf, addr)?;
            } else {
                // Broadcast
                for addr in self.addr_id_map.keys() {
                    self.socket.send_to(&buf, addr)?;
                }
            }
        }

        Ok(())
    }
}
