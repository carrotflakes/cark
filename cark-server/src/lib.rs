pub mod connection;

use cark_common::{ClientMessage, ServerMessage};

pub struct Global {
    pub messages: Vec<String>,
    field: cark_common::Field,
}

impl Global {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            field: cark_common::Field::new(20, 20),
        }
    }

    pub fn process(
        &mut self,
        incoming_events: &mut Vec<IncomingEvent>,
        mut outgoing_events: impl FnMut(OutgoingEvent),
    ) {
        for event in incoming_events.drain(..) {
            log::info!("{:?}", &event);
            match &event.message {
                ClientMessage::Join(_) => outgoing_events(OutgoingEvent {
                    connection_id: Some(event.connection_id),
                    message: ServerMessage::Joined(cark_common::Joined {
                        field: self.field.clone(),
                    }),
                }),
                ClientMessage::PublicChatMessage(message) => {
                    self.messages.push(message.text.clone());
                    // outgoing_events(OutgoingEvent::from(message.text.clone()));
                }
                ClientMessage::UpdateField(x) => {
                    self.field.data[(x.position[1] * self.field.width + x.position[0]) as usize] =
                        x.value;
                    outgoing_events(OutgoingEvent {
                        connection_id: None,
                        message: ServerMessage::UpdateField(x.clone()),
                    });
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct IncomingEvent {
    connection_id: u64,
    message: ClientMessage,
}

pub struct OutgoingEvent {
    connection_id: Option<u64>,
    message: ServerMessage,
}
