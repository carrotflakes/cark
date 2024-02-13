mod connection;
pub mod tcp;
pub mod udp;

use cark_common::model::{Character, ClientMessage, Field, JoinedCharacter, ServerMessage};

pub struct Global {
    pub messages: Vec<String>,
    field: Field,
    characters: Vec<Character>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            field: Field::new(20, 20),
            characters: vec![],
        }
    }

    pub fn process(
        &mut self,
        incoming_events: &mut Vec<IncomingEvent>,
        mut push_tcp_event: impl FnMut(OutgoingEvent),
        mut push_udp_event: impl FnMut(OutgoingEvent),
    ) {
        for event in incoming_events.drain(..) {
            log::debug!("{:?}", &event);

            match &event.message {
                ClientMessage::Join(join) => {
                    let user_id = event.connection_id;
                    let position = [1.0, 1.0];
                    self.characters.push(Character {
                        id: user_id,
                        name: join.name.clone(),
                        position,
                    });
                    push_tcp_event(OutgoingEvent {
                        connection_id: Some(event.connection_id),
                        message: ServerMessage::Joined(cark_common::model::Joined {
                            user_id,
                            field: self.field.clone(),
                            characters: self
                                .characters
                                .iter()
                                .map(|c| JoinedCharacter {
                                    id: c.id,
                                    name: c.name.clone(),
                                    position: c.position,
                                })
                                .collect(),
                        }),
                    });
                    push_tcp_event(OutgoingEvent {
                        connection_id: None,
                        message: ServerMessage::PlayerJoined {
                            id: user_id,
                            name: join.name.clone(),
                            position,
                        },
                    });
                }
                ClientMessage::Leave => {
                    self.characters.retain(|c| c.id != event.connection_id);
                    push_tcp_event(OutgoingEvent {
                        connection_id: None,
                        message: ServerMessage::PlayerLeft {
                            user_id: event.connection_id,
                        },
                    });
                }
                ClientMessage::PublicChatMessage(message) => {
                    self.messages.push(message.text.clone());
                    // outgoing_events(OutgoingEvent::from(message.text.clone()));
                }
                ClientMessage::UpdateField(x) => {
                    self.field.data[(x.position[1] * self.field.width + x.position[0]) as usize] =
                        x.value;
                    push_tcp_event(OutgoingEvent {
                        connection_id: None,
                        message: ServerMessage::UpdateField(x.clone()),
                    });
                }
                ClientMessage::Position { position, velocity } => {
                    push_udp_event(OutgoingEvent {
                        connection_id: None,
                        message: ServerMessage::Position {
                            user_id: event.connection_id,
                            position: *position,
                            velocity: *velocity,
                        },
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
