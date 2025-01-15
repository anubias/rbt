use rand::Rng;

use super::player::*;

pub struct PlayerOne {
    last_action: Action,
}

impl Player for PlayerOne {
    fn act(&mut self, _context: &Context) -> Action {
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
    pub fn new() -> Self {
        Self {
            last_action: Action::default(),
        }
    }

    fn handle_movement(&self) -> Action {
        let default_prob = 0.5;
        let rotate_prob = match self.last_action {
            Action::Idle => default_prob,
            Action::Move(_) => default_prob,
            Action::Fire => 0.5,
            Action::Rotate(_) => default_prob / 2.,
            Action::Scan(_) => default_prob,
        };

        let rotate = rand::thread_rng().gen_bool(rotate_prob);
        if rotate {
            let rotate_clockwise = rand::thread_rng().gen_bool(0.5);
            if rotate_clockwise {
                Action::Rotate(Rotation::Clockwise)
            } else {
                Action::Rotate(Rotation::CounterClockwise)
            }
        } else {
            let forward = rand::thread_rng().gen_bool(0.8);
            if forward {
                Action::Move(Direction::Forward)
            } else {
                Action::Move(Direction::Backward)
            }
        }
    }

    fn should_move(&self) -> bool {
        // check last scan result etc.
        true
    }
}
