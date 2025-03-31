#![deny(unsafe_code)]

mod api;
mod engine;
mod players;
mod terminal;

use engine::game::Game;

pub const DEBUG_MODE: bool = true;

fn main() {
    let mut game = Game::new();

    game.spawn_players();
    game.main_loop();
}
