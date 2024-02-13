use std::net::UdpSocket;

pub struct Udp {
    socket: UdpSocket,
}

impl Udp {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.socket.local_addr()
    }

    pub fn process(&mut self) -> std::io::Result<()> {
        let mut buf = [0; 1024];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let message = &buf[..size];
                    log::info!("Received {:?} from {}", message, addr);

                    let message: cark_common::model::ClientUMessage =
                        cark_common::read_from_slice(&message).unwrap();
                    log::info!("Received {:?}", message);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
