use crate::utils::Player;

pub struct Arola {
    name: String,
    health: u8,
    score: u32
}

impl Arola {
    pub fn new() -> Arola{
        Arola {
            name: "Arola".to_string(),
            health: 100,
            score: 0
        }
    }
}

impl Player for Arola {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_health(&self) -> u8 {
        self.health
    }

    fn get_score(&self) -> u32 {
        self.score
    }
}
