use cark_common::model;

use crate::{communication::Communication, game::Game, Input};

pub type BoxedSystemFn = Box<dyn FnMut(&mut Game, &Input, &mut Communication)>;

pub fn system_player_move() -> impl FnMut(&mut Game, &Input, &mut Communication) {
    let mut ddx = 0.0;
    let mut ddy = 0.0;
    let dv = 60.0;
    let fract = 0.04f32;

    return move |game, input, comm| {
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
                comm.push_tcp_event(model::ClientMessage::UpdateField(model::UpdateField {
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

            for j in 0..game.characters.len() {
                if i == j {
                    continue;
                }

                let pos1 = parry2d::na::Vector2::from(game.characters[i].position);
                let pos2 = parry2d::na::Vector2::from(game.characters[j].position);
                if let Ok(dist) = parry2d::query::details::distance(
                    &parry2d::na::Isometry2::new(pos1, 0.0),
                    &parry2d::shape::Ball::new(0.5),
                    &parry2d::na::Isometry2::new(pos2, 0.0),
                    &parry2d::shape::Ball::new(0.5),
                ) {
                    if dist == 0.0 && pos1 != pos2 {
                        let d = (pos1 - pos2).normalize() * 0.25;
                        game.characters[i].position[0] += d.x;
                        game.characters[i].position[1] += d.y;
                        game.characters[i].velocity[0] *= -0.5;
                        game.characters[i].velocity[1] *= -0.5;
                    }
                }
            }
        }
    };
}

pub fn system_player_action_push() -> impl FnMut(&mut Game, &Input, &mut Communication) {
    let mut i = 0;

    return move |game, input, comm| {
        // Throttle the updates
        i += 1;
        if i % 3 == 0 {
            return;
        }

        if let Some(character) = game.characters.iter().find(|c| c.id() == game.player_id) {
            comm.push_udp_event(model::ClientMessage::Position {
                position: character.position,
                velocity: character.velocity,
            });
        }
    };
}

pub fn system_compute_ups() -> impl FnMut(&mut Game, &Input, &mut Communication) {
    let mut last = std::time::Instant::now();

    return move |game, input, comm| {
        let now = std::time::Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;
        game.ups = game.ups * 0.9 + 1.0 / dt * 0.1;
    };
}
