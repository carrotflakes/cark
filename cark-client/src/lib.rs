pub mod client;
pub mod communication;
pub mod game;
pub mod systems;
pub mod tcp_connection;
pub mod udp;

pub struct Input {
    pub key_down: [bool; 5],
    pub key_up: [bool; 5],
    pub dt: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            key_down: [false; 5],
            key_up: [false; 5],
            dt: 0.0,
        }
    }
}
