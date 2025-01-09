use crate::utils::Player;


pub struct Samuli {
    name : String,
    score : u32,
    health : u8,
    is_ready : bool,
}


impl Samuli {
    pub fn new() -> Self {
        Samuli {
            name : "Samuli".to_string(),
            score : 0,
            health : 100,
            is_ready : false,
        }
    }
}


impl Player for Samuli {
    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_score(&self) -> u32 {
        self.score
    }

    fn get_health(&self) -> u8 {
        self.health
    }
}
