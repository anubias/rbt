use crate::actor::ActorContext;
use crate::utils::{Action, Direction, Orientation, Player, Rotation};

#[derive(Default)]
pub struct Swede {


}
impl Swede {}

impl Player for Swede {
    fn act(&mut self, context: &ActorContext) -> Action {
        
        match context.position {
            _ => {}
        }
        
        Action::Move(Direction::_Forward)
    }

    fn name(&self) -> String {
        "The Swede".to_string()
    }
}
