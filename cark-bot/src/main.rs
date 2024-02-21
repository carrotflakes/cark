fn main() {
    init_logger();

    let config = cark_bot::config::load_config();
    log::info!("{:?}", &config);

    let communication = cark_client::communication::Communication::new(
        &config.server_tcp_addr,
        &config.server_udp_addr,
    )
    .unwrap();
    let mut client = cark_client::client::Client::new(communication, "NPC".to_string());
    let mut input = cark_client::Input::new();

    let mut system = cark_bot::system_bot();

    let ups = 60.0f32;

    loop {
        input.dt = 1.0 / ups;

        system(&client.game, &mut input);
        client.process(&input);

        input.reset();

        std::thread::sleep(std::time::Duration::from_millis((1000.0 / ups) as u64));
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
