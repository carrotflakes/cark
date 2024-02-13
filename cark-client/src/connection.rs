use std::{
    io::{Read, Write},
    net::TcpStream,
};

use cark_common::model::{ClientMessage, ServerMessage};

use crate::game::{Field, Game};

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

    pub fn process(&mut self, game: &mut Game) -> Result<(), std::io::Error> {
        for event in self.outgoing_events.drain(..) {
            cark_common::write(&event, &mut self.stream).unwrap();
            self.stream.flush()?;
        }

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
            log::info!("Receive {:?}", &message);
            match message {
                ServerMessage::Joined(joined) => {
                    game.set_field(Field::from_data(
                        joined.field.width,
                        joined.field.height,
                        joined.field.data,
                    ));
                }
                ServerMessage::UpdateField(update_field) => {
                    let mut field = game.field().to_owned();
                    field.data_mut()[update_field.position[1] as usize
                        * game.field().width() as usize
                        + update_field.position[0] as usize] = update_field.value;
                    game.set_field(field);
                    // TODO: how to update the field?
                }
            }
        }

        Ok(())
    }
}
