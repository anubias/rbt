use crate::actor::ActorContext;
use crate::utils::{Action, Player};

#[derive(Default)]
pub struct Aurelian {}

impl Aurelian {}

impl Player for Aurelian {
    fn act(&mut self, context: &ActorContext) -> Action {
        println!("My position is: {}", context.position);
        Action::Fire
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
