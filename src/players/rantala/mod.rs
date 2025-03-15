use crate::api::{action::Action, context::Context, player::Player};

pub struct PlayerTeemu {}

impl PlayerTeemu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for PlayerTeemu {
    fn act(&mut self, context: Context) -> Action {
        //Scan environmen
        //Move forward until hitting obstacle or other player

        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Teemu".to_string()
    }
}
