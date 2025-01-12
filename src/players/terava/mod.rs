use super::player::*;

#[derive(Default)]
pub struct PlAgiAntti {
    fired: bool,
}

impl PlAgiAntti {}

impl Player for PlAgiAntti {
    fn act(&mut self, _context: &Context) -> Action {
        if self.fired {
            self.fired = false;
            return Action::Rotate(Rotation::Clockwise);
        } else {
            self.fired = true;
            return Action::Fire;
        }
    }

    fn name(&self) -> String {
        "tantti".to_string()
    }
}
