use super::player::{Action, Player};
use crate::game::user::Context;

#[derive(Default)]
pub struct Rahtu {}

impl Rahtu {}

impl Player for Rahtu {
    fn act(&mut self, context: &Context) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Rahtu".to_string()
    }
}
