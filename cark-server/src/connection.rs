use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;

use crate::{IncomingEvent, OutgoingEvent};

pub struct Connection {
    pub stream: TcpStream,
    pub buf: Vec<u8>,
    pub closed: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> std::io::Result<Self> {
        log::info!("Client connected: {:?}", stream);

        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buf: vec![],
            closed: false,
        })
    }

    pub fn id(&self) -> u64 {
        self.stream.as_raw_fd() as u64
    }

    fn write(&mut self, message: &cark_common::ServerMessage) -> std::io::Result<()> {
        match cark_common::to_io(message, &mut self.stream) {
            Ok(_) => {}
            Err(cark_common::PostcardError::SerializeBufferFull) => {
                log::info!("Client disconnected: {:?}", self.stream);
                self.closed = true;
                return Ok(());
            }
            Err(e) => panic!("{:?}", e),
        }
        self.stream.flush()
    }

    pub fn process(
        &mut self,
        mut push_incoming_event: impl FnMut(IncomingEvent),
        outgoing_events: &[OutgoingEvent],
    ) -> std::io::Result<()> {
        if self.closed {
            return Ok(());
        }

        for event in outgoing_events {
            if event.connection_id.is_none() || event.connection_id.unwrap() == self.id() {
                self.write(&event.message)?;
            }
        }

        match self.stream.read_to_end(&mut self.buf) {
            Ok(len) => {
                if len == 0 {
                    log::info!("Client disconnected: {:?}", self.stream);
                    self.closed = true;
                    return Ok(());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        };

        while !self.buf.is_empty() {
            // println!("{:?}", &self.buf);
            let message: cark_common::ClientMessage = match cark_common::read(&mut self.buf) {
                Ok(r) => r,
                Err(cark_common::PostcardError::DeserializeUnexpectedEnd) => break,
                Err(e) => panic!("{:?}", e),
            };
            self.buf.clear();
            push_incoming_event(IncomingEvent {
                connection_id: self.id(),
                message,
            });
        }

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}
