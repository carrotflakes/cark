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

    pub fn process(
        &mut self,
        mut push_incoming_event: impl FnMut(IncomingEvent),
        outgoing_events: &[OutgoingEvent],
    ) -> std::io::Result<()> {
        if self.closed {
            return Ok(());
        }

        for event in outgoing_events {
            match event {
                OutgoingEvent::Joined {
                    connection_id,
                    field,
                } => {
                    if *connection_id == self.id() {
                        let message = cark_common::ServerMessage::Joined(cark_common::Joined {
                            field: field.clone(),
                        });
                        cark_common::to_io(&message, &mut self.stream).unwrap();
                        self.stream.flush()?;
                    }
                }
                OutgoingEvent::Message { message } => {
                    self.stream.write(message.as_bytes())?;
                    self.stream.flush()?;
                }
            }
        }

        match self.stream.read_to_end(&mut self.buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        };

        while !self.buf.is_empty() {
            println!("{}", self.buf.len());
            let message: cark_common::ClientMessage = cark_common::read(&mut self.buf).unwrap();
            match message {
                cark_common::ClientMessage::Join(_join) => {
                    log::info!("Receive ClientMessage::Join");
                    push_incoming_event(IncomingEvent::Join {
                        connection_id: self.id(),
                    });
                }
                cark_common::ClientMessage::PublicChatMessage(_) => todo!(),
            }
        }

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}
