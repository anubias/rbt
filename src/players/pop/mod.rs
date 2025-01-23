use super::player::*;

pub struct Aurelian {}

// Public functions
impl Aurelian {
    pub fn new() -> Self {
        Self {}
    }
}

// Private functions
impl Aurelian {}

impl Player for Aurelian {
    fn act(&mut self, _context: Context) -> Action {
        Action::default()
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }
}
