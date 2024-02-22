pub mod audio;
pub mod config;

use cark_common::field::CHUNK_SIZE;

pub fn draw<C, G>(
    glyphs: &mut C,
    image: &piston_window::Image,
    tex_tiles: &G::Texture,
    ctx: piston_window::Context,
    g: &mut G,
    game: &mut cark_client::game::Game,
) where
    C: piston_window::character::CharacterCache,
    G: piston_window::Graphics<Texture = <C as piston_window::character::CharacterCache>::Texture>,
{
    use piston_window::{ellipse, text, Transformed};

    if let Some(my_character) = game.characters.iter().find(|c| c.id() == game.player_id) {
        let chip_size = 8.0;
        let cell_size = 24.0;
        let size = 16;
        let rect = [-size, -size, size * 2, size * 2];
        let data = game.field().view(my_character.chunk_id, rect);
        let transform = ctx.transform.trans(
            -cell_size * (my_character.position[0] as f64 + size as f64)
                + ctx.get_view_size()[0] / 2.0,
            -cell_size * (my_character.position[1] as f64 + size as f64)
                + ctx.get_view_size()[1] / 2.0,
        );
        for y in 0..(rect[3] - rect[1]) {
            for x in 0..(rect[2] - rect[0]) {
                let cell = data[(y * (rect[2] - rect[0]) + x) as usize];
                // rectangle(
                //     match cell {
                //         0 => [0.1, 0.1, 0.1, 1.0],
                //         1 => [0.0, 0.5, 0.0, 1.0],
                //         2 => [1.0, 0.5, 0.0, 1.0],
                //         3 => [1.0, 0.0, 0.0, 1.0],
                //         _ => [0.9, 0.9, 0.9, 1.0],
                //     },
                //     [
                //         x as f64 * cell_size,
                //         y as f64 * cell_size,
                //         cell_size,
                //         cell_size,
                //     ],
                //     transform,
                //     g,
                // );

                image
                    .src_rect([chip_size * cell as f64, 0.0, chip_size, chip_size])
                    .draw(
                        tex_tiles,
                        &Default::default(),
                        transform
                            .trans(x as f64 * cell_size, y as f64 * cell_size)
                            .scale(cell_size / chip_size * 1.01, cell_size / chip_size * 1.01),
                        g,
                    );
            }
        }

        let chunks_around = game.field().chunks_around(my_character.chunk_id);
        for character in &game.characters {
            let Some((rel, _)) = chunks_around
                .iter()
                .find(|c| c.1.map(|c| c.id == character.chunk_id).unwrap_or_default())
            else {
                continue;
            };

            let transform = transform.trans(
                rel[0] as f64 * cell_size * CHUNK_SIZE as f64
                    + (character.position[0] as f64 - rect[0] as f64) * cell_size,
                rel[1] as f64 * cell_size * CHUNK_SIZE as f64
                    + (character.position[1] as f64 - rect[1] as f64) * cell_size,
            );
            // ellipse(
            //     [0.0, 0.0, 1.0, 1.0],
            //     [-0.5 * cell_size, -0.5 * cell_size, cell_size, cell_size],
            //     transform,
            //     g,
            // );
            ellipse(
                [0.0, 0.0, 0.0, 0.25],
                [
                    -0.5 * cell_size,
                    0.5 * cell_size,
                    cell_size,
                    cell_size * 0.25,
                ],
                transform,
                g,
            );
            image
                .src_rect([chip_size * 0.0, chip_size * 1.0, chip_size, chip_size])
                .draw(
                    tex_tiles,
                    &Default::default(),
                    transform
                        .trans(-0.5 * cell_size, -0.5 * cell_size)
                        .scale(cell_size / chip_size * 1.01, cell_size / chip_size * 1.01),
                    g,
                );
            text(
                [0.0, 0.0, 0.0, 0.5],
                12,
                character.name(),
                glyphs,
                transform.trans(0.0, -cell_size),
                g,
            )
            .unwrap();
        }
    };

    text(
        [0.0, 0.0, 0.0, 1.0],
        12,
        &format!("ups: {:?}", game.ups),
        glyphs,
        ctx.transform.trans(1.0, 13.0),
        g,
    )
    .unwrap();
}

pub fn system_step_se(
) -> impl FnMut(&Option<audio::AudioSystem>, &mut cark_client::game::Game, &cark_client::Input) {
    let buf_se_step = audio::generate_pop();

    let mut step_counts = std::collections::HashMap::<u64, f32>::new();

    move |audio_sys, game, input| {
        for chara in &game.characters {
            let is_player = chara.id() == game.player_id;
            let step_count = step_counts.entry(chara.id()).or_insert(1.0);

            let d = (chara.velocity[0].powi(2) + chara.velocity[1].powi(2)).sqrt();
            if d > 0.1 {
                *step_count -= d * input.dt * 0.5;
                if *step_count < 0.0 {
                    if let Some(audio_sys) = &audio_sys {
                        audio_sys.items.lock().unwrap().push(
                            audio::AudioItem::new_se(buf_se_step.clone())
                                .volume(16.0f32.recip() * if is_player { 1.0 } else { 0.5 })
                                .pitch(
                                    0.9 + ((chara.position[0] * 5.0 + chara.position[1] * 6.0)
                                        % 1.0)
                                        * 0.2,
                                ),
                        );
                    }
                    *step_count += 1.0;
                }
            } else {
                *step_count = 1.0;
            }
        }
    }
}
