use crate::game::Game;
use piston_window::{PressEvent, ReleaseEvent};

pub fn system_player_move() -> impl FnMut(&mut Game, &piston_window::Event) {
    use piston_window::{Button, Key, UpdateEvent};

    let mut dx = 0.0;
    let mut dy = 0.0;

    return move |game: &mut Game, event: &piston_window::Event| {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            match key {
                Key::Up => {
                    dy -= 0.1;
                }
                Key::Down => {
                    dy += 0.1;
                }
                Key::Left => {
                    dx -= 0.1;
                }
                Key::Right => {
                    dx += 0.1;
                }
                _ => {}
            }
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            match key {
                Key::Up => {
                    dy += 0.1;
                }
                Key::Down => {
                    dy -= 0.1;
                }
                Key::Left => {
                    dx += 0.1;
                }
                Key::Right => {
                    dx -= 0.1;
                }
                _ => {}
            }
        }

        if let Some(_args) = event.update_args() {
            game.character[0].position[0] += dx;
            game.character[0].position[1] += dy;
        }
    };
}
