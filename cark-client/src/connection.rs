use std::{
    io::{Read, Write},
    net::TcpStream,
};

use cark_common::model::{ClientMessage, ServerMessage};

pub struct Connection {
    pub stream: TcpStream,
    pub buf: Vec<u8>,
    outgoing_events: Vec<ClientMessage>,
}

impl Connection {
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buf: Vec::new(),
            outgoing_events: vec![],
        })
    }

    pub fn push_event(&mut self, event: ClientMessage) {
        self.outgoing_events.push(event);
    }

    pub fn process(
        &mut self,
        mut handler: impl FnMut(ServerMessage),
    ) -> Result<(), std::io::Error> {
        // Send
        for event in self.outgoing_events.drain(..) {
            cark_common::write(&event, &mut self.stream).unwrap();
            self.stream.flush()?;
        }

        // Receive
        match self.stream.read_to_end(&mut self.buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        };

        while !self.buf.is_empty() {
            let message: ServerMessage = match cark_common::read(&mut self.buf) {
                Ok(r) => r,
                Err(cark_common::PostcardError::DeserializeUnexpectedEnd) => break,
                Err(e) => panic!("{:?}", e),
            };
            log::debug!("Receive {:?}", &message);

            handler(message);
        }

        Ok(())
    }
}
