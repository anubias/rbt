use super::player::{Action, Player};
use crate::game::{types::Direction, user::Context};

#[derive(Default)]
pub struct Aurelian {}

impl Aurelian {}

impl Player for Aurelian {
    fn act(&mut self, context: &Context) -> Action {
        println!("{}: context:{}", self.name(), context);
        Action::Move(Direction::Forward)
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
