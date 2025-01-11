use crate::{
    actor::ActorContext,
    utils::{Action, Player},
};

#[derive(Default)]
pub struct Swede {}

impl Swede {}

impl Player for Swede {
    fn act(&mut self, context: &ActorContext) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "The Swede".to_string()
    }
}
