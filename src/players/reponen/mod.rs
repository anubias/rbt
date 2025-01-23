use super::player::*;

pub struct Samuli {}

impl Samuli {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for Samuli {
    fn act(&mut self, context: Context) -> Action {
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Samuli".to_string()
    }
}
