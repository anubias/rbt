use crate::{
    api::{
        player::{Avatar, Player},
        world_size::WorldSize,
    },
    engine::{outcome::GameOutcome, world::World},
    terminal::{Terminal, CHAMPIONSHIP_MODE},
};

use crossterm::event::{poll, read, Event, KeyCode};

use std::time::Duration;

const ENABLE_SHELL_ANIMATION: bool = false;
const GAME_TICK_DURATION_MSEC: u64 = 10;

pub const DEAD_AVATAR: Avatar = '💀';
const DEFAULT_AVATAR: Avatar = '👶';
const AVATARS: [Avatar; 18] = [
    '🙂', '😈', '👽', '🤡', '🤖', '🎃', '🐵', '🐶', '🐱', '🦁', '🐺', '🐻', '🐼', '🦊', '🐷', '🐰',
    '🐭', '🐸',
];

pub struct Game {
    world: Box<World>,
}

impl Game {
    pub fn new(world_size: WorldSize) -> Self {
        Self {
            world: Box::new(World::new(
                ENABLE_SHELL_ANIMATION,
                GAME_TICK_DURATION_MSEC,
                world_size,
            )),
        }
    }

    pub fn start(&mut self, game_id: u32) -> (bool, GameOutcome) {
        Terminal::enter_raw_mode();

        let mut terminal = Terminal::new();
        terminal.clear_screen();
        terminal.println(self.world.to_string());

        let mut pause = false;
        let mut next = false;
        let mut quit = false;
        let mut animation = ENABLE_SHELL_ANIMATION;
        let mut tick_ms = GAME_TICK_DURATION_MSEC;

        let mut game_outcome = GameOutcome::new(game_id, self.world.map());

        while !self.world.is_game_over() {
            if !CHAMPIONSHIP_MODE {
                if let Ok(true) = poll(Duration::from_millis(0)) {
                    if let Ok(event) = read() {
                        if event == Event::Key(KeyCode::Esc.into()) {
                            quit = true;
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
            }

            let turn_outcome = self.world.new_turn(&mut terminal);
            game_outcome.add_turn_outcome(turn_outcome);

            if next {
                pause = true;
                next = false;
            }
        }

        terminal.move_caret_to_origin();
        terminal.clear_below();

        terminal.println("[Final game state]\n");
        terminal.println(&self.world);

        self.world.reward_survivors();

        let mut players = self.world.get_ready_players();
        players.sort_by(|&a, &b| a.context().score().cmp(&b.context().score()));
        players.reverse();

        let mut rank = 0;
        let mut score = u16::MAX;
        for tank in players {
            if score > tank.context().score() {
                rank += 1;
                score = tank.context().score();
            }

            game_outcome.add_player_rank(tank.context().player_details().id, rank);
        }

        (quit, game_outcome)
    }

    pub fn spawn_players(&mut self, players: Vec<Box<dyn Player>>) {
        for (rank, player) in players.into_iter().enumerate() {
            self.world.spawn_player(player, avatar(rank + 1));
        }
    }
}

pub fn avatar(player_id: usize) -> Avatar {
    let index = player_id - 1;
    if index < AVATARS.len() {
        AVATARS[index]
    } else {
        DEFAULT_AVATAR
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
    fn test_animation_is_off() {
        assert!(!ENABLE_SHELL_ANIMATION);
    }

    #[test]
    fn test_default_avatar() {
        assert_eq!(DEFAULT_AVATAR, avatar(1000));
    }
}
