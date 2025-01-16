mod players;
mod world;

use std::time::Duration;

use players::{
    armholt::Swede, arola::Arola, laurikainen::PlayerOne, player::WorldSize, pop::Aurelian,
    rahtu::Rahtu, rantala::PlayerTeemu, reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi,
    terava::PlAgiAntti,
};
use world::World;

fn main() {
    let mut world = World::new(WorldSize { x: 60, y: 30 });

    let arola = Box::new(Arola::new());
    world.spawn_player(arola);

    let armholt = Box::new(Swede::new());
    world.spawn_player(armholt);

    let laurikainen = Box::new(PlayerOne::new());
    world.spawn_player(laurikainen);

    let pop = Box::new(Aurelian::new());
    world.spawn_player(pop);

    let rahtu = Box::new(Rahtu::new());
    world.spawn_player(rahtu);

    let rantala = Box::new(PlayerTeemu::new());
    world.spawn_player(rantala);

    let reponen = Box::new(Samuli::new());
    world.spawn_player(reponen);

    let salonen = Box::new(Es::new());
    world.spawn_player(salonen);

    let sjarvi = Box::new(Siimesjarvi::new());
    world.spawn_player(sjarvi);

    let terava = Box::new(PlAgiAntti::new());
    world.spawn_player(terava);

    game_loop(world)
}

fn game_loop(mut world: World) -> ! {
    loop {
        println!("{world}");
        std::thread::sleep(Duration::from_millis(1000));

        world.new_turn();
    }
}
