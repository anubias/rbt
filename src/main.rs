mod game;
mod players;

use std::time::Duration;

use game::World;
use players::{
    armholt::Swede, arola::Arola, laurikainen::PlayerOne, player::WorldSize, pop::Aurelian,
    rahtu::Rahtu, rantala::PlayerTeemu, reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi,
    terava::PlAgiAntti,
};

fn main() {
    let mut world = World::new(WorldSize { x: 30, y: 90 });

    let arola = Box::new(Arola::default());
    world.spawn_player(arola);

    let armholt = Box::new(Swede::default());
    world.spawn_player(armholt);

    let laurikainen = Box::new(PlayerOne::default());
    world.spawn_player(laurikainen);

    let pop = Box::new(Aurelian::default());
    world.spawn_player(pop);

    let rahtu = Box::new(Rahtu::default());
    world.spawn_player(rahtu);

    let rantala = Box::new(PlayerTeemu::default());
    world.spawn_player(rantala);

    let reponen = Box::new(Samuli::default());
    world.spawn_player(reponen);

    let salonen = Box::new(Es::default());
    world.spawn_player(salonen);

    let sjarvi = Box::new(Siimesjarvi::default());
    world.spawn_player(sjarvi);

    let terava = Box::new(PlAgiAntti::default());
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
