use super::player::*;

#[derive(Default)]
pub struct Swede {}

impl Swede {}

impl Player for Swede {
    fn act(&mut self, context: &Context) -> Action {
        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "The Swede".to_string()
    }
}
