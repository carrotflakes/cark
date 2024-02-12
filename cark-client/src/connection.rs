use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::game::{Field, Game};

pub struct Connection {
    pub stream: TcpStream,
    pub buf: Vec<u8>,
    first: bool,
}

impl Connection {
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            buf: Vec::new(),
            first: true,
        })
    }

    pub fn process(&mut self, game: &mut Game) -> Result<(), std::io::Error> {
        if self.first {
            self.first = false;
            log::info!("Joining");
            cark_common::to_io(
                &cark_common::ClientMessage::Join(cark_common::Join {
                    name: "player1".to_string(),
                }),
                &mut self.stream,
            )
            .unwrap();
            self.stream.flush()?;
        }

        match self.stream.read_to_end(&mut self.buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        };

        while !self.buf.is_empty() {
            let message: cark_common::ServerMessage = cark_common::read(&mut self.buf).unwrap();
            log::info!("Receive {:?}", &message);
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
