use std::net::TcpListener;

use crate::{connection::Connection, IncomingEvent, OutgoingEvent};

pub struct Tcp {
    listener: TcpListener,
    connections: Vec<super::connection::Connection>,
    outgoing_events: Vec<OutgoingEvent>,
}

impl Tcp {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            connections: vec![],
            outgoing_events: vec![],
        })
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    pub fn push_event(&mut self, event: OutgoingEvent) {
        self.outgoing_events.push(event);
    }

    pub fn process(
        &mut self,
        mut push_incoming_event: impl FnMut(IncomingEvent),
    ) -> std::io::Result<()> {
        // Accept new connections
        for stream in self.listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            };
            self.connections.push(Connection::new(stream)?);
        }

        // Process existing connections
        for connection in &mut self.connections {
            connection
                .process(&mut push_incoming_event, &self.outgoing_events)
                .or_else(map_err)?;

            if connection.is_closed() {
                push_incoming_event(IncomingEvent {
                    connection_id: connection.id(),
                    sequence: 0,
                    message: cark_common::model::ClientMessage::Leave,
                });
            }
        }
        self.outgoing_events.clear();

        // Remove closed connections
        self.connections
            .retain(|connection| !connection.is_closed());

        Ok(())
    }

    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }
}

fn map_err(e: std::io::Error) -> Result<(), std::io::Error> {
    if e.kind() == std::io::ErrorKind::WouldBlock {
        Ok(())
    } else {
        Err(e)
    }
}
