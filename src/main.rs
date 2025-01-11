mod actor;
mod players;
mod utils;
mod world;

use std::time::Duration;

use players::{
    armholt::Swede, arola::Arola, laurikainen::PlayerOne, pop::Aurelian, rahtu::Rahtu,
    rantala::PlayerTeemu, reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi,
    terava::PlAgiAntti,
};
use world::{World, WorldSize};

fn main() {
    let mut world = World::new(WorldSize { x: 30, y: 90 });

    let mut arola = Arola::default();
    let mut armholt = Swede::default();
    let mut laurikainen = PlayerOne::default();
    let mut pop_1 = Aurelian::default();
    let mut pop_2 = Aurelian::default();
    let mut rahtu = Rahtu::default();
    let mut rantala = PlayerTeemu::default();
    let mut reponen = Samuli::default();
    let mut salonen = Es::default();
    let mut sjarvi = Siimesjarvi::default();
    let mut terava = PlAgiAntti::default();

    let mut actors = Vec::new();
    actors.push(world.spawn_actor(&mut arola));
    actors.push(world.spawn_actor(&mut armholt));
    actors.push(world.spawn_actor(&mut laurikainen));
    actors.push(world.spawn_actor(&mut pop_1));
    actors.push(world.spawn_actor(&mut pop_2));
    actors.push(world.spawn_actor(&mut rahtu));
    actors.push(world.spawn_actor(&mut rantala));
    actors.push(world.spawn_actor(&mut reponen));
    actors.push(world.spawn_actor(&mut salonen));
    actors.push(world.spawn_actor(&mut sjarvi));
    actors.push(world.spawn_actor(&mut terava));

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
