use crate::api::{action::Action, context::Context, player::Player};

pub struct Swede {}

impl Swede {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for Swede {
    fn act(&mut self, context: Context) -> Action {
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "The Swede".to_string()
    }
}
