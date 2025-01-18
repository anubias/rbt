mod players;
mod world;

use std::time::Duration;

use players::{
    alvarez::Luis, armholt::Swede, arola::Arola, laurikainen::PlayerOne, moykkynen::Joonas,
    niemisto::Niemisto, player::WorldSize, pop::Aurelian, rahtu::Rahtu, rantala::PlayerTeemu,
    reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi, terava::PlAgiAntti,
};
use world::World;

fn main() {
    let mut world = World::new(WorldSize { x: 60, y: 30 });

    spawn_players(&mut world);
    game_loop(world)
}

fn spawn_players(world: &mut Box<World>) {
    println!("Spawning players...");

    let alvarez = Box::new(Luis::new());
    world.spawn_player(alvarez);

    let armholt = Box::new(Swede::new());
    world.spawn_player(armholt);

    let arola = Box::new(Arola::new());
    world.spawn_player(arola);

    let laurikainen = Box::new(PlayerOne::new());
    world.spawn_player(laurikainen);

    let moykkynen = Box::new(Joonas::new());
    world.spawn_player(moykkynen);

    let niemisto = Box::new(Niemisto::new());
    world.spawn_player(niemisto);

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

    println!("Players spawned.");
}

fn game_loop(mut world: Box<World>) -> ! {
    loop {
        println!("{world}");
        std::thread::sleep(Duration::from_millis(100));

        world.new_turn();
    }
}
