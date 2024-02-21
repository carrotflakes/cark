pub mod client;
pub mod communication;
pub mod game;
pub mod systems;
pub mod tcp_connection;
pub mod udp;

const KEY_COUNT: usize = 6;

pub struct Input {
    pub key_down: [bool; KEY_COUNT],
    pub key_up: [bool; KEY_COUNT],
    pub dt: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            key_down: [false; KEY_COUNT],
            key_up: [false; KEY_COUNT],
            dt: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.key_down = [false; KEY_COUNT];
        self.key_up = [false; KEY_COUNT];
    }
}
