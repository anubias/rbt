use crate::{
    actor::ActorContext,
    utils::{Action, Player},
};

#[derive(Default)]
pub struct Es {}

impl Es {}

impl Player for Es {
    fn act(&mut self, context: &ActorContext) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "ES".to_string()
    }
}
