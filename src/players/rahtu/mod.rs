use super::player::*;

pub struct Rahtu {}

impl Rahtu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for Rahtu {
    fn act(&mut self, context: &Context) -> Action {
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Rahtu".to_string()
    }
}
