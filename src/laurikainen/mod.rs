use crate::{
    actor::ActorContext,
    utils::{Action, Direction, Player, Rotation},
};
use rand::Rng;

#[derive(Default)]
pub struct PlayerOne {
    last_action: Action,
}

impl Player for PlayerOne {
    fn act(&mut self, _context: &ActorContext) -> Action {
        // handle scanning and firing

        if self.should_move() {
            self.handle_movement()
        } else {
            Action::Idle
        }
    }

    fn is_ready(&self) -> bool {
        // was born ready
        true
    }

    fn name(&self) -> String {
        String::from("PlayerOne")
    }
}

impl PlayerOne {
    fn handle_movement(&self) -> Action {
        let default_prob = 0.5;
        let rotate_prob = match self.last_action {
            Action::Move(_) => default_prob,
            Action::Fire => 0.5,
            Action::Rotate(_) => default_prob / 2.,
            Action::_Scan(_) => default_prob,
            Action::Idle => default_prob,
        };

        let rotate = rand::thread_rng().gen_bool(rotate_prob);
        if rotate {
            let rotate_clockwise = rand::thread_rng().gen_bool(0.5);
            if rotate_clockwise {
                Action::Rotate(Rotation::_Clockwise)
            } else {
                Action::Rotate(Rotation::_CounterClockwise)
            }
        } else {
            let forward = rand::thread_rng().gen_bool(0.8);
            if forward {
                Action::Move(Direction::_Forward)
            } else {
                Action::Move(Direction::_Backward)
            }
        }
    }

    fn should_move(&self) -> bool {
        // check last scan result etc.
        true
    }
}
