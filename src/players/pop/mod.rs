use crate::actor::ActorContext;
use crate::utils::{Action, Direction, Player};

#[derive(Default)]
pub struct Aurelian {}

impl Aurelian {}

impl Player for Aurelian {
    fn act(&mut self, context: &ActorContext) -> Action {
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
