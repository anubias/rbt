use crate::utils::{Player, Action};

pub struct Siimesjarvi {
    name: String,
    health: u8,
    score: u32,
    is_ready: bool,
}

impl Siimesjarvi {
    pub fn new() -> Self {
        Siimesjarvi {
            name: String::from("Joni Siimesjarvi"),
            health: 100,
            score: 0,
            is_ready: false,
        }
    }
}

impl Player for Siimesjarvi {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_health(&self) -> u8 {
        self.health
    }

    fn get_score(&self) -> u32 {
        self.score
    }

    fn is_ready(&self) -> bool {
        self.is_ready
    }

    fn act() {
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_that_initial_player_values_are_correct() {
        let s = Siimesjarvi::new();
        assert_eq!(100, s.get_health());
        assert_eq!(0, s.get_score());
        assert_eq!(false, s.is_ready());
    }
}
