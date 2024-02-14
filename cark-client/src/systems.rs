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

    let mut ddx = 0.0;
    let mut ddy = 0.0;
    let dv = 60.0;
    let fract = 0.04f32;

    return move |game, event, push_outgoing_event, push_outgoing_uevent| {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::W => {
                    ddy -= dv;
                }
                Key::S => {
                    ddy += dv;
                }
                Key::A => {
                    ddx -= dv;
                }
                Key::D => {
                    ddx += dv;
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
                    ddy += dv;
                }
                Key::S => {
                    ddy -= dv;
                }
                Key::A => {
                    ddx += dv;
                }
                Key::D => {
                    ddx -= dv;
                }
                _ => {}
            }
        }

        if let Some(args) = event.update_args() {
            let dt = args.dt as f32;

            if let Some(i) = game
                .characters
                .iter()
                .position(|c| c.id() == game.player_id)
            {
                let fract = fract.powf(dt);
                game.characters[i].velocity = [
                    game.characters[i].velocity[0] * fract + ddx * dt,
                    game.characters[i].velocity[1] * fract + ddy * dt,
                ];
                game.characters[i].position[0] += game.characters[i].velocity[0] * dt;
                game.characters[i].position[1] += game.characters[i].velocity[1] * dt;
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
    let mut i = 0;

    return move |game, event, push_outgoing_event, push_outgoing_uevent| {
        if let Some(_args) = event.update_args() {
            // Throttle the updates
            i += 1;
            if i % 3 == 0 {
                return;
            }

            if let Some(character) = game.characters.iter().find(|c| c.id() == game.player_id) {
                push_outgoing_uevent(model::ClientMessage::Position {
                    position: character.position,
                    velocity: character.velocity,
                });
            }
        }
    };
}
