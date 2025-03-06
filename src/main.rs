#![deny(unsafe_code)]

mod players;
mod world;

use players::{
    alvarez::Luis,
    armholt::Swede,
    arola::Arola,
    laurikainen::PlayerOne,
    moykkynen::Joonas,
    niemisto::Niemisto,
    player::{Player, WorldSize},
    pop::Aurelian,
    rahtu::Rahtu,
    rantala::PlayerTeemu,
    reponen::Samuli,
    salonen::Es,
    siimesjarvi::Siimesjarvi,
    terava::PlAgiAntti,
};
use world::World;

pub const DEAD_AVATAR: char = '💀';
const DEFAULT_AVATAR: char = '👶';
const AVATARS: [char; 18] = [
    '🙂', '😈', '👽', '🤡', '🤖', '🎃', '🐵', '🐶', '🐱', '🦁', '🐺', '🐻', '🐼', '🦊', '🐷', '🐰',
    '🐭', '🐸',
];

const TICK_DURATION_MSEC: u64 = 100;

fn main() {
    let mut game = Game::new();
    spawn_players(&mut game);
    game.main_loop();
}

fn spawn_players(game: &mut Game) {
    println!("Spawning players...");

    game.spawn_single_player(Box::new(Luis::new()));
    game.spawn_single_player(Box::new(Swede::new()));
    game.spawn_single_player(Box::new(Arola::new()));
    game.spawn_single_player(Box::new(PlayerOne::new()));
    game.spawn_single_player(Box::new(Joonas::new()));
    game.spawn_single_player(Box::new(Niemisto::new()));
    game.spawn_single_player(Box::new(Aurelian::new()));
    game.spawn_single_player(Box::new(Rahtu::new()));
    game.spawn_single_player(Box::new(PlayerTeemu::new()));
    game.spawn_single_player(Box::new(Samuli::new()));
    game.spawn_single_player(Box::new(Es::new()));
    game.spawn_single_player(Box::new(Siimesjarvi::new()));
    game.spawn_single_player(Box::new(PlAgiAntti::new()));

    println!("Players spawned.");
}

fn avatar(player_id: usize) -> char {
    let index = player_id - 1;
    if index < AVATARS.len() {
        AVATARS[index]
    } else {
        DEFAULT_AVATAR
    }
}

struct Game {
    player_count: usize,
    world: Box<World>,
}

impl Game {
    fn new() -> Self {
        Self {
            player_count: 0,
            world: Box::new(World::new(TICK_DURATION_MSEC, WorldSize { x: 60, y: 30 })),
        }
    }

    fn spawn_single_player(&mut self, player: Box<dyn Player>) {
        self.player_count += 1;

        self.world.spawn_player(player, avatar(self.player_count));
    }

    fn main_loop(&mut self) {
        println!("{}", self.world);
        while !self.world.is_game_over() {
            self.world.new_turn();
            println!("{}", self.world);
        }

        println!("Game over!");
        self.world.reward_survivors();

        let mut players = self.world.get_ready_players();
        players.sort_by(|&a, &b| a.context().score().cmp(&b.context().score()));
        players.reverse();

        for player in players {
            println!(
                "{:02} -- {}",
                player.context().score(),
                player.player().name()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_player_is_ready() {
        let mut game = Game::new();
        spawn_players(&mut game);

        assert!(game.world.get_ready_players().is_empty());
    }
}
