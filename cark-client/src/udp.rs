use std::net::{SocketAddr, UdpSocket};

use cark_common::{
    model::{ClientUMessage, ClientUMessageBody},
    write_to_slice,
};

pub struct Udp {
    socket: UdpSocket,
    outgoing_events: Vec<ClientUMessageBody>,
    user_id: u64,
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
                        user_id: 0, // TODO
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

    pub fn push_event(&mut self, event: ClientUMessageBody) {
        self.outgoing_events.push(event);
    }

    pub fn process(&mut self) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        loop {
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    log::info!("Received {:?}", &buf[..size]);
                    // TODO
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        for event in self.outgoing_events.drain(..) {
            let message = ClientUMessage {
                user_id: self.user_id,
                body: event,
            };
            let buf = write_to_slice(&message, &mut buf).unwrap();
            self.socket.send(&buf)?;
        }

        Ok(())
    }
}
