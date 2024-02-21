use cark_common::model::ServerMessage;

use crate::{
    communication::Communication,
    game::{Character, Game},
    systems, Input,
};

pub struct Client {
    pub communication: Communication,
    pub game: Game,
    pub systems: Vec<systems::BoxedSystemFn>,
}

impl Client {
    pub fn new(mut communication: Communication, name: String) -> Self {
        communication.push_tcp_event(cark_common::model::ClientMessage::Join(
            cark_common::model::Join { name },
        ));
        Self {
            communication,
            game: Game::new(),
            systems: vec![
                Box::new(systems::system_player_move()),
                Box::new(systems::system_player_action_push()),
                Box::new(systems::system_compute_ups()),
                Box::new(systems::system_chunk_retriever()),
            ],
        }
    }

    pub fn process(&mut self, input: &Input) {
        for system in &mut self.systems {
            system(&mut self.game, &input, &mut self.communication);
        }

        let incoming_events = self.communication.process().unwrap();

        for event in incoming_events {
            // XXX
            if let cark_common::model::ServerMessage::Joined(joined) = &event {
                self.communication
                    .udp
                    .send_init(joined.user_id)
                    .or_else(map_err)
                    .unwrap();
            }

            handle_event(event, &mut self.game, &mut self.communication);
        }
    }
}

fn handle_event(event: ServerMessage, game: &mut Game, mut comm: &mut Communication) {
    match event {
        ServerMessage::Joined(joined) => {
            game.update_chunk(joined.chunk);
            game.characters = joined
                .characters
                .into_iter()
                .map(|c| Character::new(c.id, c.name, c.chunk_id, c.position))
                .collect();
            game.player_id = joined.user_id;
        }
        // ServerMessage::UpdateField(update_field) => {
        //     let mut field = game.field().to_owned();
        //     field.data_mut()[update_field.position[1] as usize * game.field().width() as usize
        //         + update_field.position[0] as usize] = update_field.value;
        //     game.set_field(field);
        //     // TODO: how to update the field?
        // }
        ServerMessage::Position {
            user_id,
            chunk_id,
            position,
            velocity,
        } => {
            if user_id == game.player_id {
                return;
            }
            if let Some(character) = game.characters.iter_mut().find(|c| c.id() == user_id) {
                character.chunk_id = chunk_id;
                character.position = position;
                character.velocity = velocity;
            }
        }
        ServerMessage::PlayerJoined {
            id,
            name,
            chunk_id,
            position,
        } => {
            if game.characters.iter().any(|c| c.id() == id) {
                return;
            }
            game.characters
                .push(Character::new(id, name, chunk_id, position));
        }
        ServerMessage::PlayerLeft { user_id } => {
            game.characters.retain(|c| c.id() != user_id);
        }
        ServerMessage::Chunk { chunk } => {
            log::info!("Chunk received: id = {:?}", chunk.id);
            game.update_chunk(chunk);
        }
    }
}

fn map_err(e: std::io::Error) -> Result<(), std::io::Error> {
    if e.kind() == std::io::ErrorKind::WouldBlock {
        Ok(())
    } else {
        Err(e)
    }
}
