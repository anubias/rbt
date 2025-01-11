use super::player::{Action, Player};
use crate::game::{types::Direction, user::Context};

#[derive(Default)]
pub struct Aurelian {}

impl Aurelian {}

impl Player for Aurelian {
    fn act(&mut self, context: &Context) -> Action {
        println!(
            "{}: position :{}, orientation: {}",
            self.name(),
            context.position,
            context.orientation
        );

        Action::Move(Direction::Forward)
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
