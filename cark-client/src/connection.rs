use std::{io::Read, net::TcpStream};

use crate::game::{Field, Game};

pub struct Connection {
    pub stream: TcpStream,
    pub buf: Vec<u8>,
}

impl Connection {
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buf: Vec::new(),
        })
    }

    pub fn process(&mut self, game: &mut Game) -> Result<(), std::io::Error> {
        self.stream.read_to_end(&mut self.buf)?;
        while !self.buf.is_empty() {
            let message: cark_common::ServerMessage = cark_common::read(&mut self.buf).unwrap();
            match message {
                cark_common::ServerMessage::Joined(joined) => {
                    game.set_field(Field::from_data(
                        joined.field.width,
                        joined.field.height,
                        joined.field.data,
                    ));
                }
            }
        }
        Ok(())
    }
}
