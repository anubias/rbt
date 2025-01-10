use crate::utils::{Player, Action, Rotation};
use crate::actor::ActorContext;

#[derive(Default)]
struct PlAgiAntti {
    fired: bool,
}

impl PlAgiAntti{}

impl Player for PlAgiAntti {
    fn act(&mut self, _context: &ActorContext) -> Action {
        if self.fired
        {
            self.fired = false;
            return Action::Rotate(Rotation::_Clockwise)
        }
        else {
            self.fired = true;
            return Action::Fire;  
        }
    }

    fn name(&self)->String{
        String::from("tantti")
    }
}
