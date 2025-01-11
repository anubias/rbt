use crate::{
    actor::ActorContext,
    utils::{Action, Player},
};

#[derive(Default)]
pub struct Rahtu {}

impl Rahtu {}

impl Player for Rahtu {
    fn act(&mut self, context: &ActorContext) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Rahtu".to_string()
    }
}
