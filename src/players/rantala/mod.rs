use super::player::*;

#[derive(Default)]
pub struct PlayerTeemu {}

impl Player for PlayerTeemu {
    fn act(&mut self, context: &Context) -> Action {
        //Scan environmen
        //Move forward until hitting obstacle or other player

        match context.position {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Teemu".to_string()
    }
}
