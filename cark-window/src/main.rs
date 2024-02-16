use piston_window::prelude::*;

fn main() {
    init_logger();

    let config = cark_window::config::load_config();
    log::info!("{:?}", &config);

    let mut window: PistonWindow = WindowSettings::new("CARK", [800 / 2, 600 / 2])
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();
    window.events.set_ups(60);

    let assets = find_folder::Search::ParentsThenKids(4, 2)
        .for_folder("cark-client")
        .unwrap()
        .join("assets");
    let mut glyphs = window.load_font(assets.join("FiraSans-Bold.ttf")).unwrap();

    let mut touch_visualizer = touch_visualizer::TouchVisualizer::new();

    let communication = cark_client::communication::Communication::new(
        &config.server_tcp_addr,
        &config.server_udp_addr,
    )
    .unwrap();
    let mut client = cark_client::client::Client::new(communication);
    let mut input = cark_client::Input::new();

    while let Some(event) = window.next() {
        touch_visualizer.event(window.size(), &event);

        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::W => {
                    input.key_down[0] = true;
                }
                Key::S => {
                    input.key_down[1] = true;
                }
                Key::A => {
                    input.key_down[2] = true;
                }
                Key::D => {
                    input.key_down[3] = true;
                }
                Key::Space => {
                    input.key_down[4] = true;
                }
                _ => {}
            }
        }
        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::W => {
                    input.key_up[0] = true;
                }
                Key::S => {
                    input.key_up[1] = true;
                }
                Key::A => {
                    input.key_up[2] = true;
                }
                Key::D => {
                    input.key_up[3] = true;
                }
                Key::Space => {
                    input.key_up[4] = true;
                }
                _ => {}
            }
        }
        if let Some(args) = event.update_args() {
            input.dt = args.dt as f32;

            client.process(&input);

            input.key_down = [false; 5];
            input.key_up = [false; 5];
        }

        window.draw_2d(&event, |ctx, g, device| {
            piston_window::clear([1.0; 4], g);

            cark_window::draw(&mut glyphs, ctx, g, &mut client.game);

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
