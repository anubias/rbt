use std::collections::HashSet;

use rand::Rng;

use super::player::*;

pub struct Surroundings {
    pub with_players: HashSet<Orientation>,
    pub valid_to_step: HashSet<Orientation>,
    pub with_unknown: HashSet<Orientation>,
}

pub struct PlayerOne {
    last_action: Action,
    last_scan: Option<ScanResult>,
    scanned_map: Option<Box<Vec<Vec<MapCell>>>>,
}

impl Player for PlayerOne {
    fn act(&mut self, context: &Context) -> Action {
        // handle scanning and firing

        let action;
        let scan = context.scanned_data();
        if scan.is_some() {}

        if self.should_move() {
            action = self.handle_movement();
        } else {
            action = Action::Fire;
        }

        self.last_scan = None;
        action
    }

    // fn is_ready(&self) -> bool {
    //     // was born ready
    //     true
    // }

    fn name(&self) -> String {
        String::from("PlayerOne")
    }
}

impl PlayerOne {
    pub fn new() -> Self {
        Self {
            last_action: Action::default(),
            last_scan: None,
            scanned_map: None,
        }
    }

    fn handle_scan(&mut self, context: &Context) -> Option<Action> {
        let mut action: Option<Action> = None;
        if self.scanned_map.is_none() {
            let world_size = context.world_size();
            self.scanned_map = Some(Box::new(vec![
                vec![MapCell::Unknown; world_size.x];
                world_size.y
            ]));
        }

        if let Some(scanned_data) = context.scanned_data() {}

        // check for players
        let surroundings = self.check_surroundings(&context);
        if let Some(first_orientation_with_player) =
            surroundings.with_players.iter().next().cloned()
        {
            action = self.rotate_to(first_orientation_with_player, context);
            if action.is_none() {
                action = Some(Action::Fire)
            }
        }

        // check for okay cells to step on
        if action.is_none() {
            if let Some(first_valid_step) = surroundings.valid_to_step.iter().next().cloned() {
                action = self.rotate_to(first_valid_step, context);
                if action.is_none() {
                    action = Some(Action::Move(Direction::Forward))
                }
            }
        }

        if action.is_none() {
            action = Some(Action::Scan(ScanType::Omni))
        }

        if !surroundings.with_players.is_empty() {
            // get the first one
        }

        action
    }

    fn rotate_to(&self, to: Orientation, context: &Context) -> Option<Action> {
        if *context.orientation() == to {
            // Already facing the direction
            return None;
        }

        // // Calculate the current orientation and target orientation as indices
        // let current = *context.orientation() as isize;
        // let target = to as isize;

        // // Calculate the clockwise and counterclockwise steps
        // let clockwise_steps = (target - current + 8) % 8;
        // let counter_clockwise_steps = (current - target + 8) % 8;

        // // Choose the shorter rotation
        // if clockwise_steps <= counter_clockwise_steps {
        //     Some(Action::Rotate(Rotation::Clockwise))
        // } else {
        //     Some(Action::Rotate(Rotation::CounterClockwise))
        // }

        None
    }

    fn check_surroundings(&mut self, context: &Context) -> Surroundings {
        let mut with_players = HashSet::new();
        let mut valid_to_step = HashSet::new();
        let mut with_unknown = HashSet::new();
        let pos = context.position();
        let world_size = context.world_size();

        if let Some(scanned_map) = self.scanned_map.as_ref() {
            let directions = [
                (-1, 0, Orientation::North),
                (1, 0, Orientation::South),
                (0, -1, Orientation::West),
                (0, 1, Orientation::East),
                (-1, -1, Orientation::NorthWest),
                (-1, 1, Orientation::NorthEast),
                (1, -1, Orientation::SouthWest),
                (1, 1, Orientation::SouthEast),
            ];

            for (dx, dy, orientation) in directions {
                let new_x = pos.x as isize + dx;
                let new_y = pos.y as isize + dy;

                // Ensure within boundaries
                if new_x >= 0
                    && new_x < world_size.x as isize
                    && new_y >= 0
                    && new_y < world_size.y as isize
                {
                    let cell = scanned_map[new_y as usize][new_x as usize];
                    match cell {
                        MapCell::Player(_) => {
                            with_players.insert(orientation);
                        }
                        MapCell::Unknown => {
                            with_unknown.insert(orientation);
                        }
                        _ if self.is_cell_okay_to_step_on(&cell) => {
                            valid_to_step.insert(orientation);
                        }
                        _ => {}
                    }
                }
            }
        }

        Surroundings {
            with_players,
            valid_to_step,
            with_unknown,
        }
    }

    fn is_cell_okay_to_step_on(&self, cell: &MapCell) -> bool {
        match cell {
            MapCell::Field => true,
            MapCell::Swamp => false,
            MapCell::Lake => false,
            MapCell::Mountain => false,
            MapCell::Player(_) => false,
            MapCell::Unknown => false,
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
        rand::thread_rng().gen_bool(0.8)
    }
}
