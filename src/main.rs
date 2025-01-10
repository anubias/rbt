mod actor;
mod utils;
mod world;

// mod arola;
mod pop;
// mod rahtu;
// mod reponen;
// mod salonen;
// mod siimesjarvi;

use std::time::Duration;

use world::{World, WorldSize};

// use arola::Arola;
use pop::Aurelian;
// use rahtu::Rahtu;
// use reponen::Samuli;
// use salonen::Es;
// use siimesjarvi::Siimesjarvi;
// use utils::{Context, Player};

fn main() {
    let mut world = World::new(WorldSize { x: 30, y: 90 });

    // let arola = Arola::new();
    let pop_1 = Aurelian::new();
    let pop_2 = Aurelian::new();
    // let rahtu = Rahtu::new();
    // let reponen = Samuli::new();
    // let salonen = Es::new();
    // let sjarvi = Siimesjarvi::new();

    let mut actors = Vec::new();
    actors.push(world.spawn_actor(&pop_1));
    actors.push(world.spawn_actor(&pop_2));

    loop {
        for actor in &mut actors {
            if actor.ready_for_action() {
                actor.act();
            }
        }

        std::thread::sleep(Duration::from_millis(500));
        println!("{world}");
    }
}
