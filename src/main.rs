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
    let mut users = Vec::new();

    let mut arola = Arola::default();
    users.push(world.spawn_user(&mut arola));

    let mut armholt = Swede::default();
    users.push(world.spawn_user(&mut armholt));

    let mut laurikainen = PlayerOne::default();
    users.push(world.spawn_user(&mut laurikainen));

    let mut pop_1 = Aurelian::default();
    users.push(world.spawn_user(&mut pop_1));
    let mut pop_2 = Aurelian::default();
    users.push(world.spawn_user(&mut pop_2));

    let mut rahtu = Rahtu::default();
    users.push(world.spawn_user(&mut rahtu));

    let mut rantala = PlayerTeemu::default();
    users.push(world.spawn_user(&mut rantala));

    let mut reponen = Samuli::default();
    users.push(world.spawn_user(&mut reponen));

    let mut salonen = Es::default();
    users.push(world.spawn_user(&mut salonen));

    let mut sjarvi = Siimesjarvi::default();
    users.push(world.spawn_user(&mut sjarvi));

    let mut terava = PlAgiAntti::default();
    users.push(world.spawn_user(&mut terava));

    game_loop(world, users)
}

fn game_loop(world: World, mut users: Vec<game::user::User<'_>>) -> ! {
    loop {
        for user in &mut users {
            if user.ready_for_action() {
                user.act();
            }
        }

        std::thread::sleep(Duration::from_millis(500));
        println!("{world}");
    }
}
