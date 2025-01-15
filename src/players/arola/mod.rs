use super::player::*;

pub struct Arola {}

impl Arola {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for Arola {
    fn act(&mut self, context: &Context) -> Action {
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Arola".to_string()
    }
}
