mod connection;
pub mod tcp;
pub mod udp;

use cark_common::{
    field::{ChunkId, Field},
    model::{Character, ClientMessage, JoinedCharacter, ServerMessage},
    udp_stat::Sequence,
};

pub struct Global {
    pub messages: Vec<String>,
    field: Field,
    characters: Vec<Character>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            field: Field::new(),
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
                    let chunk_id = ChunkId::MIN;
                    let position = [2.0, 2.0];
                    self.characters.push(Character {
                        id: user_id,
                        name: join.name.clone(),
                        chunk_id,
                        position,
                    });
                    push_tcp_event(OutgoingEvent {
                        connection_id: Some(event.connection_id),
                        message: ServerMessage::Joined(cark_common::model::Joined {
                            user_id,
                            chunk: self.field.chunk(chunk_id).unwrap().clone(),
                            characters: self
                                .characters
                                .iter()
                                .map(|c| JoinedCharacter {
                                    id: c.id,
                                    name: c.name.clone(),
                                    chunk_id: c.chunk_id.clone(),
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
                            chunk_id: chunk_id,
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
                // ClientMessage::UpdateField(x) => {
                //     self.field.data[(x.position[1] * self.field.width + x.position[0]) as usize] =
                //         x.value;
                //     push_tcp_event(OutgoingEvent {
                //         connection_id: None,
                //         message: ServerMessage::UpdateField(x.clone()),
                //     });
                // }
                ClientMessage::Position {
                    chunk_id,
                    position,
                    velocity,
                } => {
                    push_udp_event(OutgoingEvent {
                        connection_id: None,
                        message: ServerMessage::Position {
                            user_id: event.connection_id,
                            chunk_id: *chunk_id,
                            position: *position,
                            velocity: *velocity,
                        },
                    });

                    // for d in &Direction::ALL {
                    //     if let Some(id) = self.field.generate_chunk(*chunk_id, *d) {
                    //         log::info!("Chunk generated: id = {:?}", id);
                    //     }
                    // }
                }
                ClientMessage::RequestChunk { id, direction } => {
                    self.field.generate_chunk(*id, *direction);

                    if let Some(chunk) = self
                        .field
                        .chunk(*id)
                        .and_then(|c| ChunkId::new(c.related[direction.to_number()]))
                        .and_then(|id| self.field.chunk(id))
                    {
                        push_tcp_event(OutgoingEvent {
                            connection_id: Some(event.connection_id),
                            message: ServerMessage::Chunk {
                                chunk: chunk.clone(),
                            },
                        });
                    } else {
                        log::warn!("Chunk requested but not found: id = {:?}", id);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct IncomingEvent {
    connection_id: u64,
    // This will be 0 for TCP connections.
    sequence: Sequence,
    message: ClientMessage,
}

pub struct OutgoingEvent {
    connection_id: Option<u64>,
    message: ServerMessage,
}
