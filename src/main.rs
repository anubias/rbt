#![deny(unsafe_code)]

mod display_printer;
mod players;
mod world;

use std::{thread, time::Duration};

use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use display_printer::DisplayPrinter;
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

fn main() -> std::io::Result<()> {
    let mut game = Game::new();

    game.setup()?;

    spawn_players(&mut game);
    game.main_loop();

    game.teardown()
}

fn spawn_players(game: &mut Game) {
    DisplayPrinter::println_str("Spawning players...");

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

    DisplayPrinter::println_str("Players spawned.");
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

    fn setup(&self) -> std::io::Result<()> {
        enable_raw_mode()
    }

    fn teardown(&self) -> std::io::Result<()> {
        disable_raw_mode()
    }

    fn spawn_single_player(&mut self, player: Box<dyn Player>) {
        self.player_count += 1;

        self.world.spawn_player(player, avatar(self.player_count));
    }

    fn main_loop(&mut self) {
        DisplayPrinter::clear();
        DisplayPrinter::println(self.world.to_string());

        let mut pause = false;
        while !self.world.is_game_over() {
            if let Ok(true) = poll(Duration::from_millis(1)) {
                if let Ok(event) = read() {
                    if event == Event::Key(KeyCode::Char('q').into())
                        || event == Event::Key(KeyCode::Char('Q').into())
                    {
                        break;
                    } else if event == Event::Key(KeyCode::Char('p').into())
                        || event == Event::Key(KeyCode::Char('P').into())
                    {
                        pause = !pause;
                    }
                }
            }

            if pause {
                thread::sleep(Duration::from_micros(TICK_DURATION_MSEC));
                continue;
            }

            self.world.new_turn();

            DisplayPrinter::clear();
            DisplayPrinter::println(self.world.to_string());
        }

        if self.world.is_game_over() {
            DisplayPrinter::println_str("[Game ended]\n");
        } else {
            DisplayPrinter::println_str("[Game interrupted]\n");
        }

        self.world.reward_survivors();

        let mut players = self.world.get_ready_players();
        players.sort_by(|&a, &b| a.context().score().cmp(&b.context().score()));
        players.reverse();

        DisplayPrinter::println_str("[RANKING]");
        DisplayPrinter::println_str("=========\n");
        DisplayPrinter::println_str("RANK  SCORE  PLAYER");
        DisplayPrinter::println_str("----  -----  -------------------------");
        for (place, player) in players.iter().enumerate() {
            let text = format!(
                " {:02}    {:03}   {}",
                place + 1,
                player.context().score(),
                player.player().name()
            );
            DisplayPrinter::println(text);
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

    #[test]
    fn test_no_stack_overflow_during_world_generation() {
        for _ in 0..100 {
            // repeat world generation many times, hopefully catching stack overflows
            let _ = Game::new();
        }
    }
}
