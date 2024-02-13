use cark_client::{
    connection::Connection,
    game::Game,
    systems::{self, BoxedSystemFn},
    udp::Udp,
};
use cark_common::model::{ClientMessage, ServerMessage};
use piston_window::prelude::*;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // ip a show dev eth0
    let mut ip = "127.0.0.1".to_owned();
    if let Ok(f) = std::fs::File::open("cark.txt") {
        let mut reader = std::io::BufReader::new(f);
        let mut s = String::new();
        std::io::Read::read_to_string(&mut reader, &mut s).unwrap();
        ip = s.trim().to_owned();
    }
    log::info!("CARK_SERVER_IP: {:?}", ip);

    let server_addr_tcp = format!("{}:8080", ip);
    let server_addr_udp = format!("{}:8081", ip);

    let mut window: PistonWindow = WindowSettings::new("CARK", [800 / 2, 600 / 2])
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
    let mut systems: Vec<BoxedSystemFn> = vec![
        Box::new(systems::system_player_move()),
        Box::new(systems::system_player_action_push()),
    ];

    let mut udp = Udp::new(&server_addr_udp).unwrap();
    let mut connection = Connection::new(&server_addr_tcp).unwrap();

    let name = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % 1000)
        .to_string();
    connection.push_event(ClientMessage::Join(cark_common::model::Join { name }));

    while let Some(event) = window.next() {
        touch_visualizer.event(window.size(), &event);

        for system in &mut systems {
            system(
                &mut game,
                &event,
                &mut |message| connection.push_event(message),
                &mut |message| udp.push_event(message),
            );
        }
        // TODO: systemの自殺

        let mut incoming_events = vec![];
        let mut handler = |message| incoming_events.push(message);

        connection.process(&mut handler).or_else(map_err).unwrap();
        udp.process(&mut handler).or_else(map_err).unwrap();

        for event in incoming_events {
            // XXX
            if let ServerMessage::Joined(joined) = &event {
                udp.send_init(joined.user_id).or_else(map_err).unwrap();
            }

            handle_event(
                event,
                &mut game,
                &mut |m| connection.push_event(m),
                &mut |m| udp.push_event(m),
            );
        }

        window.draw_2d(&event, |ctx, g, device| {
            piston_window::clear([1.0; 4], g);

            cark_client::draw(&mut glyphs, ctx, g, &mut game);

            glyphs.factory.encoder.flush(device);
        });
    }
}

fn handle_event(
    event: ServerMessage,
    game: &mut Game,
    mut push_tcp_event: impl FnMut(ClientMessage),
    mut push_udp_event: impl FnMut(ClientMessage),
) {
    match event {
        ServerMessage::Joined(joined) => {
            game.set_field(cark_client::game::Field::from_data(
                joined.field.width,
                joined.field.height,
                joined.field.data,
            ));
            game.characters = joined
                .characters
                .into_iter()
                .map(|c| cark_client::game::Character::new(c.id, c.name, c.position))
                .collect();
            game.player_id = joined.user_id;
        }
        ServerMessage::UpdateField(update_field) => {
            let mut field = game.field().to_owned();
            field.data_mut()[update_field.position[1] as usize * game.field().width() as usize
                + update_field.position[0] as usize] = update_field.value;
            game.set_field(field);
            // TODO: how to update the field?
        }
        ServerMessage::Position {
            user_id,
            position,
            velocity,
        } => {
            if user_id == game.player_id {
                return;
            }
            if let Some(character) = game.characters.iter_mut().find(|c| c.id() == user_id) {
                character.position = position;
                character.velocity = velocity;
            }
        }
        ServerMessage::PlayerJoined { id, name, position } => {
            if game.characters.iter().any(|c| c.id() == id) {
                return;
            }
            game.characters
                .push(cark_client::game::Character::new(id, name, position));
        }
        ServerMessage::PlayerLeft { user_id } => {
            game.characters.retain(|c| c.id() != user_id);
        }
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
