use super::player::{Action, Player};
use crate::game::user::Context;

#[derive(Default)]
pub struct Arola {}

impl Arola {}

impl Player for Arola {
    fn act(&mut self, context: &Context) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Arola".to_string()
    }
}
