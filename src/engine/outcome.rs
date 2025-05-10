use std::collections::HashMap;

use crate::api::{
    action::Action, map_cell::MapCell, player::PlayerId, position::Position,
    world_size::MAX_WORLD_SIZE,
};

#[derive(Debug)]
pub struct ChampionshipOutcome {
    players: Vec<PlayerEntry>,
    game_results: Vec<GameOutcome>,
    ranks: HashMap<PlayerId, f32>,
}

impl ChampionshipOutcome {
    pub fn new() -> Self {
        ChampionshipOutcome {
            players: Vec::new(),
            game_results: Vec::new(),
            ranks: HashMap::new(),
        }
    }

    pub fn register_player(&mut self, id: PlayerId, name: String) {
        self.players.push(PlayerEntry { id, name });
    }

    pub fn add_game_result(&mut self, game_result: GameOutcome) {
        self.game_results.push(game_result);
    }

    pub fn compute_ranks(&mut self) {
        for game in &self.game_results {
            for (player_id, rank) in &game.ranks {
                let computed_rank = self.ranks.entry(*player_id).or_default();
                *computed_rank += *rank as f32;
            }
        }

        for rank in self.ranks.values_mut() {
            *rank /= self.game_results.len() as f32;
        }
    }

    pub fn get_ranks(&self) -> HashMap<PlayerId, f32> {
        self.ranks.clone()
    }

    pub fn get_player_name(&self, id: PlayerId) -> Option<String> {
        self.players
            .iter()
            .find(|&entry| entry.id == id)
            .map(|entry| entry.name.clone())
    }
}

#[derive(Debug)]
pub struct PlayerEntry {
    id: PlayerId,
    name: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct GameOutcome {
    game_id: u32,
    original_map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
    turns: Vec<TurnOutcome>,
    ranks: HashMap<PlayerId, u8>,
}

impl GameOutcome {
    pub fn new(
        game_id: u32,
        original_map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
    ) -> Self {
        GameOutcome {
            game_id,
            original_map,
            turns: Vec::new(),
            ranks: HashMap::new(),
        }
    }

    pub fn add_turn_outcome(&mut self, turn: TurnOutcome) {
        self.turns.push(turn);
    }

    pub fn add_player_rank(&mut self, id: PlayerId, rank: u8) {
        self.ranks.insert(id, rank);
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TurnOutcome {
    number: usize,
    players: HashMap<PlayerId, PlayerOutcome>,
}

impl TurnOutcome {
    pub fn new(number: usize) -> Self {
        TurnOutcome {
            number,
            players: HashMap::new(),
        }
    }

    pub fn add_player_outcome(&mut self, id: PlayerId, outcome: PlayerOutcome) {
        self.players.insert(id, outcome);
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PlayerOutcome {
    action: Action,
    health: u8,
    position: Position,
    score: u16,
}

impl PlayerOutcome {
    pub fn new(action: Action, health: u8, position: Position, score: u16) -> Self {
        PlayerOutcome {
            action: action.into(),
            health,
            position: position.into(),
            score,
        }
    }
}
