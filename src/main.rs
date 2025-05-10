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
    let championship_outcome = championship.run(GAME_ROUNDS);

    println!("");
    println!("");
    println!("[RANKING]");
    println!("=========\n");
    println!("RANK  ID  PLAYER");
    println!("----  --  -------------------------");

    let mut ranks = championship_outcome.get_ranks();

    while !ranks.is_empty() {
        let mut player_id = 0;
        let mut rank = f32::MAX;
        ranks.iter().for_each(|(p, r)| {
            if *r <= rank {
                player_id = *p;
                rank = *r;
            }
        });

        let entry = ranks.remove(&player_id);
        if let Some(r) = entry {
            let text = format!(
                "{:02.02}  {:02}  {}",
                r,
                player_id,
                championship_outcome
                    .get_player_name(player_id)
                    .unwrap_or_default()
            );
            println!("{text}");
        }
    }

    println!("");
}
