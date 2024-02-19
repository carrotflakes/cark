use cark_common::field::CHUNK_SIZE;

pub mod config;

pub fn draw<C, G>(
    glyphs: &mut C,
    ctx: piston_window::Context,
    g: &mut G,
    game: &mut cark_client::game::Game,
) where
    C: piston_window::character::CharacterCache,
    G: piston_window::Graphics<Texture = <C as piston_window::character::CharacterCache>::Texture>,
{
    use piston_window::{ellipse, rectangle, text, Transformed};

    if let Some(my_character) = game.characters.iter().find(|c| c.id() == game.player_id) {
        let cell_size = 10.0;
        let size = 16;
        let rect = [-size, -size, size * 2, size * 2];
        let data = game.field().view(my_character.chunk_id, rect);
        let transform: [[f64; 3]; 2] = ctx.transform.trans(
            -cell_size * my_character.position[0] as f64,
            -cell_size * my_character.position[1] as f64,
        );
        for y in 0..(rect[3] - rect[1]) {
            for x in 0..(rect[2] - rect[0]) {
                let cell = data[(y * (rect[2] - rect[0]) + x) as usize];
                rectangle(
                    match cell {
                        0 => [0.1, 0.1, 0.1, 1.0],
                        1 => [0.0, 0.5, 0.0, 1.0],
                        2 => [1.0, 0.5, 0.0, 1.0],
                        3 => [1.0, 0.0, 0.0, 1.0],
                        _ => [0.9, 0.9, 0.9, 1.0],
                    },
                    [
                        x as f64 * cell_size,
                        y as f64 * cell_size,
                        cell_size,
                        cell_size,
                    ],
                    transform,
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
            ellipse(
                [0.0, 0.0, 1.0, 1.0],
                [-0.5 * cell_size, -0.5 * cell_size, cell_size, cell_size],
                transform,
                g,
            );
            text(
                [0.0, 0.0, 0.0, 0.5],
                12,
                character.name(),
                glyphs,
                transform.trans(0.0, 0.0),
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
