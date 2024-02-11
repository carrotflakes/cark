use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    env_logger::init();

    let addr = std::env::var("ADDR").unwrap_or("0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    log::info!("Listening on: {:?}", listener.local_addr()?);

    let mut connections = vec![];
    let mut global = cark_server::Global::new();
    let mut incoming_events = vec![];
    let mut outgoing_events = vec![];

    loop {
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            };
            connections.push(cark_server::connection::Connection::new(stream)?);
        }

        for connection in &mut connections {
            connection
                .process(|e| incoming_events.push(e), &outgoing_events)
                .or_else(map_err)?;
        }

        connections.retain(|connection| !connection.is_closed());

        outgoing_events.clear();
        global.process(&mut incoming_events, |e| outgoing_events.push(e));
    }
}

fn map_err(e: std::io::Error) -> Result<(), std::io::Error> {
    if e.kind() == std::io::ErrorKind::WouldBlock {
        Ok(())
    } else {
        Err(e)
    }
}
