use std::net::{SocketAddr, UdpSocket};

use cark_common::{
    model::{ClientMessage, ClientUdpMessage, ServerMessage, ServerUdpMessage},
    udp_stat::{SequenceGen, UdpStat},
};

pub struct Udp {
    socket: UdpSocket,
    outgoing_events: Vec<ClientMessage>,
    stat: UdpStat,
    sequence: SequenceGen,
}

impl Udp {
    pub fn new(server_addr: &str) -> std::io::Result<Self> {
        for port in 34254..65535 {
            match UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], port))) {
                Ok(socket) => {
                    log::info!("UDP socket bound to port {}", port);

                    socket.set_nonblocking(true)?;
                    socket.connect(server_addr)?;
                    return Ok(Self {
                        socket,
                        outgoing_events: vec![],
                        stat: UdpStat::new(),
                        sequence: SequenceGen::default(),
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {}
                Err(e) => return Err(e),
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "No available port",
        ))
    }

    pub fn push_event(&mut self, event: ClientMessage) {
        self.outgoing_events.push(event);
    }

    pub fn process(&mut self, mut handler: impl FnMut(ServerMessage)) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        // Receive
        loop {
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    let message = &buf[..size];

                    let message: ServerUdpMessage = cark_common::read_from_slice(&message).unwrap();
                    log::debug!("Received {:?}", message);

                    match message {
                        ServerUdpMessage::Init => todo!(),
                        ServerUdpMessage::Message { sequence, message } => {
                            self.stat.update(sequence);

                            handler(message);
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
            let message = ClientUdpMessage::Message {
                sequence: self.sequence.next(),
                message: event,
            };

            let buf = cark_common::write_to_slice(&message, &mut buf).unwrap();
            self.socket.send(&buf)?;
        }

        Ok(())
    }

    pub fn send_init(&mut self, id: u64) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        let message = ClientUdpMessage::Init { id };
        let buf = cark_common::write_to_slice(&message, &mut buf).unwrap();
        self.socket.send(&buf)?;

        Ok(())
    }

    pub fn stat(&self) -> &UdpStat {
        &self.stat
    }
}
