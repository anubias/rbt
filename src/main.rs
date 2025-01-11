mod game;
mod players;

use std::time::Duration;

use game::world::{World, WorldSize};
use players::{
    armholt::Swede, arola::Arola, laurikainen::PlayerOne, pop::Aurelian, rahtu::Rahtu,
    rantala::PlayerTeemu, reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi,
    terava::PlAgiAntti,
};

fn main() {
    let mut world = World::new(WorldSize { x: 30, y: 90 });

    let mut arola = Arola::default();
    world.spawn_user(&mut arola);

    let mut armholt = Swede::default();
    world.spawn_user(&mut armholt);

    let mut laurikainen = PlayerOne::default();
    world.spawn_user(&mut laurikainen);

    let mut pop_1 = Aurelian::default();
    world.spawn_user(&mut pop_1);
    // let mut pop_2 = Aurelian::default();
    // world.spawn_user(&mut pop_2);

    let mut rahtu = Rahtu::default();
    world.spawn_user(&mut rahtu);

    let mut rantala = PlayerTeemu::default();
    world.spawn_user(&mut rantala);

    let mut reponen = Samuli::default();
    world.spawn_user(&mut reponen);

    let mut salonen = Es::default();
    world.spawn_user(&mut salonen);

    let mut sjarvi = Siimesjarvi::default();
    world.spawn_user(&mut sjarvi);

    let mut terava = PlAgiAntti::default();
    world.spawn_user(&mut terava);

    game_loop(world)
}

fn game_loop(mut world: World) -> ! {
    loop {
        println!("{world}");
        std::thread::sleep(Duration::from_millis(1000));

        world.new_turn();
    }
}
