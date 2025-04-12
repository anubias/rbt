#![deny(unsafe_code)]

mod api;
mod engine;
mod players;
mod terminal;

use api::world_size::WorldSize;
use engine::championship::{Championship, League};

const GAME_ROUNDS: u32 = 1;
const WORLD_SIZE: WorldSize = WorldSize { x: 60, y: 45 };

fn main() {
    let mut championship = Championship::new(League::Academy, WORLD_SIZE);

    championship.run(GAME_ROUNDS);
}
