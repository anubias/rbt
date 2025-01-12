use super::player::*;

#[derive(Default)]
pub struct Es {}

impl Es {}

impl Player for Es {
    fn act(&mut self, context: &Context) -> Action {
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "ES".to_string()
    }
}
