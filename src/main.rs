#![deny(unsafe_code)]

mod api;
mod engine;
mod players;
mod terminal;

use engine::championship::{Championship, League};

const GAME_ROUNDS: u32 = 1;

// Useful during development phase, eventually it should be turned off
pub const DEBUG_MODE: bool = true;

fn main() {
    let mut championship = Championship::new(League::Academy);
    championship.run(GAME_ROUNDS);
}
