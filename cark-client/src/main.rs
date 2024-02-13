use cark_client::{connection::Connection, game::Game, systems::system_player_move, udp::Udp};
use cark_common::model::{ClientMessage, ClientUMessageBody};
use piston_window::prelude::*;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // ip a show dev eth0
    let ip = option_env!("CARK_SERVER_IP").unwrap_or("127.0.0.1");
    let server_addr_tcp = format!("{}:8080", ip);
    let server_addr_udp = format!("{}:8081", ip);

    let mut window: PistonWindow = WindowSettings::new("CARK", [800, 600])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(4, 2)
        .for_folder("cark-client")
        .unwrap()
        .join("assets");
    let mut glyphs = window.load_font(assets.join("FiraSans-Bold.ttf")).unwrap();

    let mut touch_visualizer = touch_visualizer::TouchVisualizer::new();

    let mut game = Game::new();
    let mut systems: Vec<
        Box<dyn FnMut(&mut Game, &piston_window::Event, &mut dyn FnMut(ClientMessage))>,
    > = vec![Box::new(system_player_move())];

    let mut udp = Udp::new(&server_addr_udp).unwrap();
    let mut connection = Connection::new(&server_addr_tcp).unwrap();

    connection.push_event(ClientMessage::Join(cark_common::model::Join {
        name: game.character[0].name().to_owned(),
    }));

    // test
    udp.push_event(ClientUMessageBody::Position {
        position: [1.0, 0.0],
        velocity: [0.0, 0.0],
    });

    while let Some(event) = window.next() {
        touch_visualizer.event(window.size(), &event);

        for system in &mut systems {
            system(&mut game, &event, &mut |message| {
                connection.push_event(message)
            });
        }

        connection.process(&mut game).or_else(map_err).unwrap();
        udp.process().or_else(map_err).unwrap();

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

#[test]
fn test() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let server_addr_udp = "127.0.0.1:8081";

    Udp::new(server_addr_udp).unwrap();
}
