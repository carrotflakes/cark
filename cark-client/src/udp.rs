use std::net::{SocketAddr, UdpSocket};

use cark_common::model::{ClientMessage, ServerMessage, UdpMessage};

pub struct Udp {
    socket: UdpSocket,
    outgoing_events: Vec<ClientMessage>,
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

                    let message: ServerMessage = cark_common::read_from_slice(&message).unwrap();
                    log::debug!("Received {:?}", message);
                    handler(message);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        // Send
        for event in self.outgoing_events.drain(..) {
            let message = UdpMessage::Message { message: event };
            let buf = cark_common::write_to_slice(&message, &mut buf).unwrap();
            self.socket.send(&buf)?;
        }

        Ok(())
    }

    pub fn send_init(&mut self, id: u64) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        let message = UdpMessage::Init { id };
        let buf = cark_common::write_to_slice(&message, &mut buf).unwrap();
        self.socket.send(&buf)?;

        Ok(())
    }
}
