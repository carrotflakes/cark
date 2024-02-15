pub mod config;
pub mod game;
pub mod systems;
pub mod tcp_connection;
pub mod udp;

pub fn draw<C, G>(glyphs: &mut C, ctx: piston_window::Context, g: &mut G, game: &mut game::Game)
where
    C: piston_window::character::CharacterCache,
    G: piston_window::Graphics<Texture = <C as piston_window::character::CharacterCache>::Texture>,
{
    use piston_window::{ellipse, rectangle, text, Transformed};

    text(
        [0.0, 0.0, 0.0, 1.0],
        12,
        &format!("ups: {:?}", game.ups),
        glyphs,
        ctx.transform.trans(1.0, 13.0),
        g,
    )
    .unwrap();

    let width = game.field().width();
    let height = game.field().height();
    let data = game.field().data();
    let transform = ctx.transform.trans(20.0, 20.0);
    for x in 0..width {
        for y in 0..height {
            let cell = data[(y * width + x) as usize];
            rectangle(
                if cell == 0 {
                    [1.0, 0.0, 0.0, 1.0]
                } else {
                    [1.0, 1.0, 0.0, 1.0]
                },
                [x as f64 * 10.0, y as f64 * 10.0, 10.0, 10.0],
                transform,
                g,
            );
        }
    }

    for character in &game.characters {
        let transform = transform.trans(
            character.position[0] as f64 * 10.0,
            character.position[1] as f64 * 10.0,
        );
        ellipse([0.0, 0.0, 1.0, 1.0], [-5.0, -5.0, 10.0, 10.0], transform, g);
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
}
