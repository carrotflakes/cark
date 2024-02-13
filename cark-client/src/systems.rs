use cark_common::model;

use crate::game::Game;

pub fn system_player_move(
) -> impl FnMut(&mut Game, &piston_window::Event, &mut dyn FnMut(model::ClientMessage)) {
    use piston_window::{Button, Key, PressEvent, ReleaseEvent, UpdateEvent};

    let mut dx = 0.0;
    let mut dy = 0.0;

    return move |game: &mut Game,
                 event: &piston_window::Event,
                 push_outgoing_event: &mut dyn FnMut(model::ClientMessage)| {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::W => {
                    dy -= 0.1;
                }
                Key::S => {
                    dy += 0.1;
                }
                Key::A => {
                    dx -= 0.1;
                }
                Key::D => {
                    dx += 0.1;
                }
                Key::Space => {
                    let position = [
                        game.character[0].position[0] as u32,
                        game.character[0].position[1] as u32,
                    ];
                    push_outgoing_event(model::ClientMessage::UpdateField(model::UpdateField {
                        position,
                        value: game.field().data()[position[1] as usize
                            * game.field().width() as usize
                            + position[0] as usize]
                            ^ 1,
                    }));
                }
                _ => {}
            }
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::W => {
                    dy += 0.1;
                }
                Key::S => {
                    dy -= 0.1;
                }
                Key::A => {
                    dx += 0.1;
                }
                Key::D => {
                    dx -= 0.1;
                }
                _ => {}
            }
        }

        if let Some(_args) = event.update_args() {
            game.character[0].position[0] += dx;
            game.character[0].position[1] += dy;
            game.character[0].position[0] =
                game.character[0].position[0].clamp(0.0, game.field().width() as f32);
            game.character[0].position[1] =
                game.character[0].position[1].clamp(0.0, game.field().height() as f32);
        }
    };
}
