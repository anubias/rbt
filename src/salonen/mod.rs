use crate::utils::Player;

pub struct Es {
    score: u32,
    health: u8,
    ready: bool,
    name: String,
}

impl Es {
    pub fn new() -> Self {
        Es {
            score: 999,
            health: 100,
            ready: false,
            name: String::from("ES"),
        }
    }
}

impl Player for Es {
    fn is_ready(&self) -> bool {
        self.ready
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }

    fn get_score(&self) -> u32 {
        self.score
    }

    fn get_health(&self) -> u8 {
        self.health
    }
}
