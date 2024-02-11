use cark_client::{game::Game, systems::system_player_move};
use piston_window::prelude::*;

fn main() {
        let addr = "127.0.0.1:8080";

    let mut window: PistonWindow = WindowSettings::new("CARK", [800, 600])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(5, 2)
        .for_folder("assets")
        .unwrap();
    let mut glyphs = window.load_font(assets.join("FiraSans-Bold.ttf")).unwrap();

    let mut touch_visualizer = touch_visualizer::TouchVisualizer::new();

    let mut game = Game::new();
    let mut systems: Vec<Box<dyn FnMut(&mut Game, &piston_window::Event)>> =
        vec![Box::new(system_player_move())];

    let mut connection = cark_client::connection::Connection::new(addr).unwrap();

    while let Some(event) = window.next() {
        touch_visualizer.event(window.size(), &event);

        for system in &mut systems {
            system(&mut game, &event);
        }

        connection.process(&mut game).or_else(map_err).unwrap();

        window.draw_2d(&event, |ctx, g, device| {
            piston_window::clear([1.0; 4], g);

            cark_client::draw(&mut glyphs, ctx, g, &mut game);

            glyphs.factory.encoder.flush(device);
        });
    }
}

fn map_err(e: std::io::Error) -> Result<(), std::io::Error> {
    if e.kind() == std::io::ErrorKind::WouldBlock {
        Ok(())
    } else {
        Err(e)
    }
}
