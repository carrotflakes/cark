use cark_server::{tcp::Tcp, udp::Udp};

fn main() -> std::io::Result<()> {
    env_logger::init();

    let addr = std::env::var("ADDR").unwrap_or("0.0.0.0:8080".to_string());

    let mut tcp = Tcp::new(&addr)?;
    let mut udp = Udp::new("0.0.0.0:8081")?;

    log::info!(
        "Listening on: {:?}, {:?}",
        tcp.local_addr()?,
        udp.local_addr()?
    );

    let mut global = cark_server::Global::new();
    let mut incoming_events = vec![];

    loop {
        udp.process(|e| incoming_events.push(e)).or_else(map_err)?;
        tcp.process(|e| incoming_events.push(e))?;

        global.process(
            &mut incoming_events,
            |e| tcp.push_event(e),
            |e| udp.push_event(e),
        );

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn map_err(e: std::io::Error) -> Result<(), std::io::Error> {
    if e.kind() == std::io::ErrorKind::WouldBlock {
        Ok(())
    } else {
        Err(e)
    }
}
