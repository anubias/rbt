use super::player::*;

pub struct Es {}

impl Es {
    pub fn new() -> Self {
        Self {}
    }
}

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
