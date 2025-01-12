use super::player::{Action, Context, Player};

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
