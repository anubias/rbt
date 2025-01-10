use crate::{
    actor::ActorContext,
    utils::{Action, Player},
};

struct PlayerTeemu {

}

impl Player for PlayerTeemu {
    fn act(&mut self, context: &ActorContext) -> Action {
        //Scan environmen
        //Move forward until hitting obstacle or other player
        if context.orientation == Orientation.North)
        {

        }
        

        

        return Action::Move(crate::utils::Direction::_Forward);
    }

    fn name(&self) -> String {
        return "Teemu".to_string();
    }
}
