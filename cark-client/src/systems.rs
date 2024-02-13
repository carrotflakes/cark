use cark_common::model;
use piston_window::UpdateEvent;

use crate::game::Game;

pub type BoxedSystemFn = Box<
    dyn FnMut(
        &mut Game,
        &piston_window::Event,
        &mut dyn FnMut(model::ClientMessage),
        &mut dyn FnMut(model::ClientMessage),
    ),
>;

pub fn system_player_move() -> impl FnMut(
    &mut Game,
    &piston_window::Event,
    &mut dyn FnMut(model::ClientMessage),
    &mut dyn FnMut(model::ClientMessage),
) {
    use piston_window::{Button, Key, PressEvent, ReleaseEvent};

    let mut dx = 0.0;
    let mut dy = 0.0;

    return move |game, event, push_outgoing_event, push_outgoing_uevent| {
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
                    if let Some(i) = game
                        .characters
                        .iter()
                        .position(|c| c.id() == game.player_id)
                    {
                        let position: [u32; 2] = [
                            game.characters[i].position[0] as u32,
                            game.characters[i].position[1] as u32,
                        ];
                        push_outgoing_event(model::ClientMessage::UpdateField(
                            model::UpdateField {
                                position,
                                value: game.field().data()[position[1] as usize
                                    * game.field().width() as usize
                                    + position[0] as usize]
                                    ^ 1,
                            },
                        ));
                    }
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
            if let Some(i) = game
                .characters
                .iter()
                .position(|c| c.id() == game.player_id)
            {
                game.characters[i].position[0] += dx;
                game.characters[i].position[1] += dy;
                game.characters[i].position[0] =
                    game.characters[i].position[0].clamp(0.0, game.field().width() as f32);
                game.characters[i].position[1] =
                    game.characters[i].position[1].clamp(0.0, game.field().height() as f32);
            }
        }
    };
}

pub fn system_player_action_push() -> impl FnMut(
    &mut Game,
    &piston_window::Event,
    &mut dyn FnMut(model::ClientMessage),
    &mut dyn FnMut(model::ClientMessage),
) {
    return move |game, event, push_outgoing_event, push_outgoing_uevent| {
        if let Some(_args) = event.update_args() {
            if let Some(character) = game.characters.iter().find(|c| c.id() == game.player_id) {
                push_outgoing_uevent(model::ClientMessage::Position {
                    position: character.position,
                    velocity: [0.0, 0.0],
                });
            }
        }
    };
}
