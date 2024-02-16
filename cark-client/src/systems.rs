use cark_common::model;

use crate::{game::Game, Input};

pub type BoxedSystemFn = Box<
    dyn FnMut(
        &mut Game,
        &Input,
        &mut dyn FnMut(model::ClientMessage),
        &mut dyn FnMut(model::ClientMessage),
    ),
>;

pub fn system_player_move() -> impl FnMut(
    &mut Game,
    &Input,
    &mut dyn FnMut(model::ClientMessage),
    &mut dyn FnMut(model::ClientMessage),
) {
    let mut ddx = 0.0;
    let mut ddy = 0.0;
    let dv = 60.0;
    let fract = 0.04f32;

    return move |game, input, push_outgoing_event, push_outgoing_uevent| {
        if input.key_down[0] {
            ddy -= dv;
        }
        if input.key_down[1] {
            ddy += dv;
        }
        if input.key_down[2] {
            ddx -= dv;
        }
        if input.key_down[3] {
            ddx += dv;
        }
        if input.key_down[4] {
            if let Some(i) = game
                .characters
                .iter()
                .position(|c| c.id() == game.player_id)
            {
                let position: [u32; 2] = [
                    game.characters[i].position[0] as u32,
                    game.characters[i].position[1] as u32,
                ];
                push_outgoing_event(model::ClientMessage::UpdateField(model::UpdateField {
                    position,
                    value: game.field().data()[position[1] as usize
                        * game.field().width() as usize
                        + position[0] as usize]
                        ^ 1,
                }));
            }
        }

        if input.key_up[0] {
            ddy += dv;
        }
        if input.key_up[1] {
            ddy -= dv;
        }
        if input.key_up[2] {
            ddx += dv;
        }
        if input.key_up[3] {
            ddx -= dv;
        }

        let dt = input.dt;

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
    };
}

pub fn system_player_action_push() -> impl FnMut(
    &mut Game,
    &Input,
    &mut dyn FnMut(model::ClientMessage),
    &mut dyn FnMut(model::ClientMessage),
) {
    let mut i = 0;

    return move |game, input, push_outgoing_event, push_outgoing_uevent| {
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
    };
}

pub fn system_compute_ups() -> impl FnMut(
    &mut Game,
    &Input,
    &mut dyn FnMut(model::ClientMessage),
    &mut dyn FnMut(model::ClientMessage),
) {
    let mut last = std::time::Instant::now();

    return move |game, input, push_outgoing_event, push_outgoing_uevent| {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;
        game.ups = game.ups * 0.9 + 1.0 / dt * 0.1;
    };
}
