mod model;
mod scanner;
mod strategy;
mod tanks;
mod terrain;
mod types;

//use super::player::*;
use tanks::*;

use strategy::*;
use terrain::*;
use types::*;

pub struct Luis {
    id: PId,
    abs_pos: Position,
    strategy: Option<StrategyManager>,
}

impl Player for Luis {
    fn act(&mut self, context: Context) -> Action {
        self.ensure_strategy_initializtion(&context);

        if let Some(strategy) = &mut self.strategy {
            strategy.process(&context);
            return strategy.decide_action(&context);
        }
        println!("oi oi oi, nothing selected, why?");
        Action::Idle
    }

    fn name(&self) -> String {
        "Shin-chan".to_string()
    }

    fn initialized(&mut self) -> bool {
        true
    }

    fn is_ready(&self) -> bool {
        true
    }
}

// Private functions
impl Luis {
    pub fn new() -> Self {
        Self {
            id: 0,
            abs_pos: Position { x: 0, y: 0 },
            strategy: None,
        }
    }

    fn ensure_strategy_initializtion(&mut self, context: &Context) {
        if self.strategy.is_none() {
            let player_id = context.player_details().id;
            let position = context.position().clone();

            let map = MappedTerrain::new(context.world_size().clone());
            let tanks = TanksTracker::new(player_id);

            self.strategy = Some(StrategyManager::new(
                model::WorldModel { map: map, tanks: tanks }));
            self.id = player_id;
            self.abs_pos = position;
        }
    }

}
