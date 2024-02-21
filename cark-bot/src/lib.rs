pub mod config;

use cark_client::{game::Game, Input};

pub fn system_bot() -> impl FnMut(&Game, &mut Input) {
    let mut dir = None;

    return move |game, input| {
        let Some(character) = game.player_character() else {
            return;
        };

        let mut new_dir = None;

        game.characters.iter().for_each(|c| {
            if c.id() == game.player_id || c.chunk_id != character.chunk_id {
                return;
            }

            let dx = c.position[0] - character.position[0];
            let dy = c.position[1] - character.position[1];
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 10.0 {
                if dx.abs() > dy.abs() {
                    if dx > 0.0 {
                        new_dir = Some(3);
                    } else {
                        new_dir = Some(2);
                    }
                } else {
                    if dy > 0.0 {
                        new_dir = Some(1);
                    } else {
                        new_dir = Some(0);
                    }
                }
            }
        });

        if let Some(new_dir) = new_dir {
            if let Some(dir) = dir {
                if new_dir != dir {
                    input.key_up[dir] = true;
                    input.key_down[new_dir] = true;
                }
            } else {
                input.key_down[new_dir] = true;
            }
        } else {
            if let Some(dir) = dir {
                input.key_up[dir] = true;
            }
        }
        dir = new_dir;
    };
}
