use std::collections::HashMap;

use crate::api::{
    action::Action, map_cell::MapCell, player::PlayerId, position::Position,
    world_size::MAX_WORLD_SIZE,
};

#[derive(Debug)]
pub struct ChampionshipOutcome {
    players: Vec<PlayerDetails>,
    game_results: Vec<GameOutcome>,
}

impl ChampionshipOutcome {
    pub fn new() -> Self {
        ChampionshipOutcome {
            players: Vec::new(),
            game_results: Vec::new(),
        }
    }

    pub fn add_player(&mut self, id: PlayerId, name: String) {
        self.players.push(PlayerDetails { id, name });
    }

    pub fn add_game_result(&mut self, game_result: GameOutcome) {
        self.game_results.push(game_result);
    }
}

#[derive(Debug)]
pub struct PlayerDetails {
    id: PlayerId,
    name: String,
}

#[derive(Debug)]
pub struct GameOutcome {
    game_id: u32,
    original_map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
    turns: Vec<TurnOutcome>,
    scores: HashMap<PlayerId, u16>,
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
            scores: HashMap::new(),
        }
    }

    pub fn add_turn_outcome(&mut self, turn: TurnOutcome) {
        self.turns.push(turn);
    }

    pub fn add_player_score(&mut self, id: PlayerId, score: u16) {
        self.scores.insert(id, score);
    }
}

#[derive(Debug)]
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
