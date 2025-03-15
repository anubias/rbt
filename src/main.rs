#![deny(unsafe_code)]

mod api;
mod engine;
mod players;
mod terminal;

use crossterm::event::{poll, read, Event, KeyCode};
use std::time::Duration;

use api::{
    player::{Avatar, Player},
    world_size::WorldSize,
};
use engine::world::World;
use players::{
    alvarez::Luis, armholt::Swede, arola::Arola, laurikainen::PlayerOne, moykkynen::Joonas,
    niemisto::Niemisto, pop::Aurelian, rahtu::Rahtu, rantala::PlayerTeemu, reponen::Samuli,
    salonen::Es, siimesjarvi::Siimesjarvi, terava::PlAgiAntti,
};
use terminal::Terminal;

pub const DEAD_AVATAR: Avatar = '💀';
const DEFAULT_AVATAR: Avatar = '👶';
const AVATARS: [Avatar; 18] = [
    '🙂', '😈', '👽', '🤡', '🤖', '🎃', '🐵', '🐶', '🐱', '🦁', '🐺', '🐻', '🐼', '🦊', '🐷', '🐰',
    '🐭', '🐸',
];

pub const DEBUG_MODE: bool = true;
const ENABLE_SHELL_ANIMATION: bool = false;
const GAME_TICK_DURATION_MSEC: u64 = 10;

fn main() {
    let mut game = Game::new();

    spawn_players(&mut game);
    game.main_loop();
}

fn spawn_players(game: &mut Game) {
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
            world: Box::new(World::new(
                GAME_TICK_DURATION_MSEC,
                WorldSize { x: 60, y: 30 },
            )),
        }
    }

    fn spawn_single_player(&mut self, player: Box<dyn Player>) {
        self.player_count += 1;

        self.world.spawn_player(player, avatar(self.player_count));
    }

    fn main_loop(&mut self) {
        Terminal::enter_raw_mode();

        let mut terminal = Terminal::new();
        terminal.clear_screen();
        terminal.println(self.world.to_string());

        let mut pause = false;
        let mut next = false;
        let mut animation = ENABLE_SHELL_ANIMATION;
        let mut tick_ms = GAME_TICK_DURATION_MSEC;

        while !self.world.is_game_over() {
            if let Ok(true) = poll(Duration::from_millis(0)) {
                if let Ok(event) = read() {
                    if event == Event::Key(KeyCode::Esc.into()) {
                        break;
                    } else if event == Event::Key(KeyCode::Up.into()) {
                        tick_ms = tick_ms.saturating_add(1);
                        self.world.update_tick(tick_ms);
                    } else if event == Event::Key(KeyCode::Down.into()) {
                        tick_ms = tick_ms.saturating_sub(1);
                        self.world.update_tick(tick_ms);
                    } else if event == Event::Key(KeyCode::Char('a').into())
                        || event == Event::Key(KeyCode::Char('A').into())
                    {
                        animation = !animation;
                        self.world.update_animation(animation);
                    } else if event == Event::Key(KeyCode::Char('n').into())
                        || event == Event::Key(KeyCode::Char('N').into())
                    {
                        pause = false;
                        next = true;
                    } else if event == Event::Key(KeyCode::Char('p').into())
                        || event == Event::Key(KeyCode::Char('P').into())
                    {
                        pause = !pause;
                    }
                }
            }

            if pause {
                continue;
            }

            self.world.new_turn(&mut terminal);

            if next {
                pause = true;
                next = false;
            }
        }

        terminal.move_caret_to_origin();
        if self.world.is_game_over() {
            terminal.println("[Game ended]");
        } else {
            terminal.println("[Game interrupted]");
        }

        terminal.println("[Final game state]\n");
        terminal.println(&self.world);

        self.world.reward_survivors();

        let mut players = self.world.get_ready_players();
        players.sort_by(|&a, &b| a.context().score().cmp(&b.context().score()));
        players.reverse();

        terminal.println("[RANKING]");
        terminal.println("=========\n");
        terminal.println("RANK  SCORE  PLAYER");
        terminal.println("----  -----  -------------------------");
        for (place, player) in players.iter().enumerate() {
            let text = format!(
                " {:02}    {:03}   {}",
                place + 1,
                player.context().score(),
                player.player().name()
            );
            terminal.println(text);
        }

        terminal.println("");
        terminal.clear_below();
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        Terminal::exit_raw_mode();
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
    fn test_animation_is_off() {
        assert!(!ENABLE_SHELL_ANIMATION);
    }

    #[test]
    fn test_default_avatar() {
        assert_eq!(DEFAULT_AVATAR, avatar(1000));
    }
}
