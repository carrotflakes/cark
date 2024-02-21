pub mod config;

use cark_client::{game::Game, Input};
use cark_common::field::CHUNK_SIZE;

pub fn system_bot() -> impl FnMut(&Game, &mut Input) {
    let mut dir = None;

    return move |game, input| {
        let Some(character) = game.player_character() else {
            return;
        };
        let chunks_around = game.field().chunks_around(character.chunk_id);

        let mut new_dir = None;

        game.characters.iter().for_each(|chara| {
            if chara.id() == game.player_id {
                return;
            }

            let Some(pos) = chunks_around.iter().find_map(|(p, c)| {
                c.and_then(|c| {
                    if c.id == chara.chunk_id {
                        Some(p)
                    } else {
                        None
                    }
                })
            }) else {
                return;
            };

            let dx = chara.position[0] + pos[0] as f32 * CHUNK_SIZE as f32 - character.position[0];
            let dy = chara.position[1] + pos[1] as f32 * CHUNK_SIZE as f32 - character.position[1];
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 6.0 {
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
