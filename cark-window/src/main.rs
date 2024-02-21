use cark_window::audio::{render_to_buffer, start_audio};
use piston_window::{prelude::*, Image};

fn main() {
    init_logger();

    let config = cark_window::config::load_config();
    log::info!("{:?}", &config);

    let mut window: PistonWindow = WindowSettings::new("CARK", [640, 480])
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();
    window.events.set_ups(60);

    let assets = find_folder::Search::ParentsThenKids(4, 2)
        .for_folder("cark-window")
        .unwrap()
        .join("assets");
    let mut glyphs = window.load_font(assets.join("FiraSans-Bold.ttf")).unwrap();

    let ref mut texture_context = window.create_texture_context();
    let tex_tiles = Texture::from_path(
        texture_context,
        &assets.join("tile.png"),
        Flip::None,
        &TextureSettings::new().mag(Filter::Nearest),
    )
    .unwrap();

    let mut touch_visualizer = touch_visualizer::TouchVisualizer::new();

    let name = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % 1000)
        .to_string();

    let communication = cark_client::communication::Communication::new(
        &config.server_tcp_addr,
        &config.server_udp_addr,
    )
    .unwrap();
    let mut client = cark_client::client::Client::new(communication, name);
    let mut input = cark_client::Input::new();

    let audio_res = start_audio();
    match &audio_res {
        Ok(a) => {
            let buffer: Vec<f32> = {
                let file = assets.join("cark00.mid");

                let data = std::fs::read(&file).unwrap();
                let events = ezmid::parse(&data);

                render_to_buffer(a.sample_rate as f32, events)
            };

            let channels = a.channels;
            let mut i = 0;
            *a.callback.lock().unwrap() = Box::new(move |len| {
                let mut buf = vec![0.0; len];
                for b in buf.chunks_mut(channels) {
                    for b in b.iter_mut() {
                        *b = buffer[i];
                    }
                    i = (i + 1) % buffer.len();
                }
                buf
            });
        }
        Err(e) => {
            log::error!("Failed to start audio: {:?}", e);
        }
    }
    let audio_res = audio_res.ok();

    let image = Image::new();

    while let Some(event) = window.next() {
        touch_visualizer.event(window.size(), &event);

        if let Some(Button::Keyboard(key)) = event.press_args() {
            if let Some(k) = match key {
                Key::W => Some(0),
                Key::S => Some(1),
                Key::A => Some(2),
                Key::D => Some(3),
                Key::Space => Some(4),
                Key::Escape => Some(5),
                _ => None,
            } {
                input.key_down[k] = true;
            }
        }
        if let Some(Button::Keyboard(key)) = event.release_args() {
            if let Some(k) = match key {
                Key::W => Some(0),
                Key::S => Some(1),
                Key::A => Some(2),
                Key::D => Some(3),
                Key::Space => Some(4),
                Key::Escape => Some(5),
                _ => None,
            } {
                input.key_up[k] = true;
            }
        }
        if let Some(args) = event.update_args() {
            input.dt = args.dt as f32;

            client.process(&input);

            input.reset();
        }

        window.draw_2d(&event, |ctx, g, device| {
            piston_window::clear([1.0; 4], g);

            cark_window::draw(&mut glyphs, &image, &tex_tiles, ctx, g, &mut client.game);

            glyphs.factory.encoder.flush(device);
        });
    }
}

fn init_logger() {
    use simplelog::*;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            std::fs::File::create("cark.log").unwrap(),
        ),
    ])
    .unwrap();

    log_panics::init();
}
