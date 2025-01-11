use crate::{
    actor::ActorContext,
    utils::{Action, Player},
};

#[derive(Default)]
pub struct Arola {}

impl Arola {}

impl Player for Arola {
    fn act(&mut self, context: &ActorContext) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Arola".to_string()
    }
}
