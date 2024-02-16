use crate::tcp_connection::TcpConnection;
use crate::udp::Udp;

pub struct Communication {
    pub tcp: TcpConnection,
    pub udp: Udp,
}

impl Communication {
    pub fn new(tcp_addr: &str, udp_addr: &str) -> std::io::Result<Self> {
        Ok(Self {
            tcp: TcpConnection::new(tcp_addr)?,
            udp: Udp::new(udp_addr)?,
        })
    }

    pub fn push_tcp_event(&mut self, event: cark_common::model::ClientMessage) {
        self.tcp.push_event(event);
    }

    pub fn push_udp_event(&mut self, event: cark_common::model::ClientMessage) {
        self.udp.push_event(event);
    }

    pub fn process(&mut self) -> std::io::Result<Vec<cark_common::model::ServerMessage>> {
        let mut incoming_events = vec![];
        let mut handler = |message| incoming_events.push(message);

        self.tcp.process(&mut handler)?;
        self.udp.process(&mut handler)?;

        Ok(incoming_events)
    }
}
