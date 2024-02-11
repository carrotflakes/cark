use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;

use crate::{IncomingEvent, OutgoingEvent};

pub struct Connection {
    pub stream: TcpStream,
    pub buf: Vec<u8>,
    pub step: usize,
    pub closed: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> std::io::Result<Self> {
        log::info!("Client joined: {:?}", stream);

        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buf: vec![],
            step: 0,
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
                        cark_common::to_io(field, &mut self.stream).unwrap();
                        self.stream.flush()?;
                    }
                }
                OutgoingEvent::Message { message } => {
                    self.stream.write(message.as_bytes())?;
                    self.stream.flush()?;
                }
            }
        }

        if self.step == 0 {
            self.stream.write(b"Hello, world!\n")?;
            self.stream.flush()?;
            self.step = 1;
            push_incoming_event(IncomingEvent::Join {
                connection_id: self.id(),
            });
        }
        if self.step == 1 {
            self.stream.read_to_end(&mut self.buf)?;
            log::info!("Received: {:?}", self.buf);
            if self.buf.is_empty() {
                log::info!("Client left: {:?}", self.stream);
                self.closed = true;
            }
            let mes = String::from_utf8(self.buf.clone()).unwrap();
            push_incoming_event(IncomingEvent::from(mes));
            self.buf.clear();
            // self.step = 2;
        }
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}
