use super::player::{Action, Player};
use crate::game::user::Context;

#[derive(Default)]
pub struct Samuli {}

impl Samuli {}

impl Player for Samuli {
    fn act(&mut self, context: &Context) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Samuli".to_string()
    }
}
