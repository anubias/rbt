use crate::utils::Player;
use crate::utils::Position;

pub struct Rahtu
{
    score: u32,
    health: u8
}

impl Rahtu {
    pub fn new(position: Position) -> Self {
        Rahtu {
            score: u32::MAX,
            health: 100,
        }
    }
}

impl Player for Rahtu{
    fn is_ready(&self) -> bool {
        false
    }

    /// Returns the player's name
    fn get_name(&self) -> String {
        "Rahtu".to_string()
    }

    ///
    fn get_score(&self) -> u32 {
        self.score
    }

    fn get_health(&self) -> u8 {
        self.health
    }
}
