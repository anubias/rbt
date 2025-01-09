use crate::utils::{Player, Position};

pub struct Aurelian {
    health: u8,
    name: String,
    position: Position,
    ready: bool,
    score: u32,
}

impl Aurelian {
    pub fn new(position: Position) -> Self {
        Aurelian {
            health: 100,
            name: "Aurelian".to_string(),
            position,
            ready: false,
            score: 0,
        }
    }
}

impl Player for Aurelian {
    fn get_health(&self) -> u8 {
        self.health
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn get_score(&self) -> u32 {
        self.score
    }
}
