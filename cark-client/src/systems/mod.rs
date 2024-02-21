use cark_common::{
    direction::Direction,
    field::{ChunkId, CHUNK_SIZE},
    model,
};

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
        // if input.key_down[4] {
        //     if let Some(i) = game
        //         .characters
        //         .iter()
        //         .position(|c| c.id() == game.player_id)
        //     {
        //         let position: [u32; 2] = [
        //             game.characters[i].position[0] as u32,
        //             game.characters[i].position[1] as u32,
        //         ];
        //         comm.push_tcp_event(model::ClientMessage::UpdateField(model::UpdateField {
        //             position,
        //             value: game.field().data()[position[1] as usize
        //                 * game.field().width() as usize
        //                 + position[0] as usize]
        //                 ^ 1,
        //         }));
        //     }
        // }

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

        let mut dt = input.dt;

        if let Some(i) = game
            .characters
            .iter()
            .position(|c| c.id() == game.player_id)
        {
            let chunk_id = game.characters[i].chunk_id;

            let fract = fract.powf(dt);
            game.characters[i].velocity = [
                game.characters[i].velocity[0] * fract + ddx * dt,
                game.characters[i].velocity[1] * fract + ddy * dt,
            ];

            let pos = parry2d::na::Vector2::from(game.characters[i].position);
            let character_shape = parry2d::shape::Ball::new(0.5);
            loop {
                let block = parry2d::shape::SharedShape::new(parry2d::shape::Cuboid::new(
                    parry2d::na::Vector2::new(0.5, 0.5),
                ));
                let field = game.field();
                let cx = game.characters[i].position[0] as i32;
                let cy = game.characters[i].position[1] as i32;
                let rect = [cx - 1, cy - 1, cx + 2, cy + 2];
                let shapes: Vec<_> = field
                    .view(chunk_id, rect)
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &v)| {
                        if v == 1 {
                            let x = i as i32 % 3 + cx - 1;
                            let y = i as i32 / 3 + cy - 1;
                            Some((
                                parry2d::na::Isometry2::new(
                                    parry2d::na::Vector2::new(x as f32 + 0.5, y as f32 + 0.5),
                                    0.0,
                                ),
                                block.clone(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                if shapes.is_empty() {
                    break;
                }
                let col = parry2d::shape::Compound::new(shapes);
                if let Some(toi) = parry2d::query::time_of_impact(
                    &parry2d::na::Isometry2::new(pos, 0.0),
                    &parry2d::na::Vector2::from(game.characters[i].velocity),
                    &character_shape,
                    &Default::default(),
                    &Default::default(),
                    &col,
                    dt,
                    false,
                )
                .unwrap()
                {
                    // log::info!("toi: {:?}", toi);
                    // toi.status == parry2d::query::TOIStatus::Converged;
                    game.characters[i].position[0] +=
                        game.characters[i].velocity[0] * toi.toi * 0.99;
                    game.characters[i].position[1] +=
                        game.characters[i].velocity[1] * toi.toi * 0.99;
                    let refl = toi.normal2.scale(
                        1.5 * parry2d::na::Vector2::from(game.characters[i].velocity).norm(),
                    );
                    game.characters[i].velocity[0] += refl.x;
                    game.characters[i].velocity[1] += refl.y;
                    dt -= toi.toi;
                    continue;
                }
                break;
            }

            game.characters[i].position[0] += game.characters[i].velocity[0] * dt;
            game.characters[i].position[1] += game.characters[i].velocity[1] * dt;

            let chunks_around: Vec<_> = game
                .field()
                .chunks_around(chunk_id)
                .into_iter()
                .map(|c| (c.0, c.1.map(|c| c.id)))
                .collect();
            for j in 0..game.characters.len() {
                if i == j {
                    continue;
                }

                let Some((rel, _)) = chunks_around.iter().find(|c| {
                    c.1.map(|id| id == game.characters[j].chunk_id)
                        .unwrap_or_default()
                }) else {
                    continue;
                };

                let pos2 = parry2d::na::Vector2::from([
                    game.characters[j].position[0] + rel[0] as f32 * CHUNK_SIZE as f32,
                    game.characters[j].position[1] + rel[1] as f32 * CHUNK_SIZE as f32,
                ]);
                if let Ok(dist) = parry2d::query::details::distance(
                    &parry2d::na::Isometry2::new(pos, 0.0),
                    &character_shape,
                    &parry2d::na::Isometry2::new(pos2, 0.0),
                    &character_shape,
                ) {
                    if dist == 0.0 && pos != pos2 {
                        let d = (pos - pos2).normalize() * 0.25;
                        game.characters[i].position[0] += d.x;
                        game.characters[i].position[1] += d.y;
                        game.characters[i].velocity[0] *= -0.5;
                        game.characters[i].velocity[1] *= -0.5;
                    }
                }
            }

            // Move to the next chunk
            {
                if game.characters[i].position[0] < 0.0 {
                    if let Some(chunk_id) = game
                        .field()
                        .chunk(game.characters[i].chunk_id)
                        .and_then(|c| ChunkId::new(c.related[Direction::Left.to_number()]))
                    {
                        game.characters[i].chunk_id = chunk_id;
                        game.characters[i].position[0] += CHUNK_SIZE as f32;
                    } else {
                        game.characters[i].position[0] = 0.0;
                        game.characters[i].velocity[0] = 0.0;
                    }
                }
                if game.characters[i].position[0] >= CHUNK_SIZE as f32 {
                    if let Some(chunk_id) = game
                        .field()
                        .chunk(game.characters[i].chunk_id)
                        .and_then(|c| ChunkId::new(c.related[Direction::Right.to_number()]))
                    {
                        game.characters[i].chunk_id = chunk_id;
                        game.characters[i].position[0] -= CHUNK_SIZE as f32;
                    } else {
                        game.characters[i].position[0] = CHUNK_SIZE as f32 - 0.1;
                        game.characters[i].velocity[0] = 0.0;
                    }
                }
                if game.characters[i].position[1] < 0.0 {
                    if let Some(chunk_id) = game
                        .field()
                        .chunk(game.characters[i].chunk_id)
                        .and_then(|c| ChunkId::new(c.related[Direction::Top.to_number()]))
                    {
                        game.characters[i].chunk_id = chunk_id;
                        game.characters[i].position[1] += CHUNK_SIZE as f32;
                    } else {
                        game.characters[i].position[1] = 0.0;
                        game.characters[i].velocity[1] = 0.0;
                    }
                }
                if game.characters[i].position[1] >= CHUNK_SIZE as f32 {
                    if let Some(chunk_id) = game
                        .field()
                        .chunk(game.characters[i].chunk_id)
                        .and_then(|c| ChunkId::new(c.related[Direction::Bottom.to_number()]))
                    {
                        game.characters[i].chunk_id = chunk_id;
                        game.characters[i].position[1] -= CHUNK_SIZE as f32;
                    } else {
                        game.characters[i].position[1] = CHUNK_SIZE as f32 - 0.1;
                        game.characters[i].velocity[1] = 0.0;
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
                chunk_id: character.chunk_id,
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

pub fn system_chunk_retriever() -> impl FnMut(&mut Game, &Input, &mut Communication) {
    let mut requested = vec![];
    let mut time = 0.5;

    return move |game, input, comm| {
        if time <= 0.0 {
            time = 0.5;
            requested.clear();
        }
        time -= input.dt;

        let mut request = |chunk_id, i| {
            log::info!("Requesting chunk: id = {:?}, direction = {:?}", chunk_id, i);
            comm.push_tcp_event(model::ClientMessage::RequestChunk {
                id: chunk_id,
                direction: Direction::from_number(i),
            });
        };

        let Some(character) = game.characters.iter().find(|c| c.id() == game.player_id) else {
            return;
        };
        let chunk_id = character.chunk_id;
        if let Some(chunk) = game.field().chunk(chunk_id) {
            for i in 0..4 {
                if requested.contains(&(chunk_id, i)) {
                    continue;
                }

                if let Some(chunk) =
                    ChunkId::new(chunk.related[i]).and_then(|id| game.field().chunk(id))
                {
                    let i = Direction::from_number(i).turn_left().to_number();

                    if requested.contains(&(chunk_id, i)) {
                        continue;
                    }

                    if let Some(_) =
                        ChunkId::new(chunk.related[i]).and_then(|id| game.field().chunk(id))
                    {
                        continue;
                    }
                    request(chunk.id, i);
                    requested.push((chunk_id, i));
                    continue;
                }
                request(chunk_id, i);
                requested.push((chunk_id, i));
            }
        }
    };
}
