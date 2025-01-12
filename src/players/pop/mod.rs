use super::player::*;

#[derive(Default)]
pub struct Aurelian {}

impl Aurelian {}

impl Player for Aurelian {
    fn act(&mut self, context: &Context) -> Action {
        println!("{}: context:{}", self.name(), context);
        if context.is_mobile() {
            Action::Move(Direction::Forward)
        } else {
            Action::Move(Direction::Backward)
        }
        // Action::Rotate(Rotation::Clockwise)
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
