use std::collections::HashSet;

use rand::Rng;

use crate::api::{
    action::Action,
    aiming::Aiming,
    context::Context,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    player::Player,
    position::{Position, SCANNING_DISTANCE},
    rotation::Rotation,
    scan::{ScanResult, ScanType},
};

pub struct Surroundings {
    pub with_players: HashSet<Orientation>,
    pub valid_to_step: HashSet<Orientation>,
    pub with_unknown: HashSet<Orientation>,
}

pub struct PlayerOne {
    last_action: Action,
    last_scan: Option<ScanResult>,
    scanned_map: Option<Box<Vec<Vec<MapCell>>>>,
    counter: u32,
    prev_position: Option<Position>,
}

impl Player for PlayerOne {
    fn act(&mut self, context: Context) -> Action {
        self.counter += 1;
        // handle scanning and firing
        let mut have_an_action = true;
        let pos_x = context.position().x;
        let pos_y = context.position().y;

        let mut action = if let Some(scan_action) = self.handle_scan(context) {
            scan_action
        } else {
            have_an_action = false;
            Action::default()
        };

        if !have_an_action {
            if self.should_move() {
                action = self.handle_movement();
            } else {
                action = Action::Idle;
            }
        }

        self.last_scan = None;
        println!("ACT {}, {action:?}", self.counter);

        //println!("{self}");
        self.prev_position = Some(Position { x: pos_x, y: pos_y });
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
            counter: 0,
            prev_position: None,
        }
    }

    /// Handles the logic for scanning the surroundings.
    ///
    /// This function checks the scanned data for any players in the surroundings.
    /// If it finds any, it rotates towards the first one it found and fires.
    /// If there are no players, it checks for any valid cells to step on.
    /// If it finds any valid cells, it rotates towards the first one it found and moves forward.
    /// If there are no valid cells, it scans the surroundings again.
    /// The function returns an action based on the logic above, or None if no action was found.
    fn handle_scan(&mut self, context: Context) -> Option<Action> {
        let mut action: Option<Action> = None;
        if self.scanned_map.is_none() {
            let world_size = context.world_size();
            self.scanned_map = Some(Box::new(vec![
                vec![MapCell::Unallocated; world_size.x];
                world_size.y
            ]));
        }

        println!(
            "Position: {:?}, current orientation: {:?},",
            context.position(),
            context.player_details().orientation
        );

        if let Some(scanned_data) = context.scanned_data() {
            let dist = SCANNING_DISTANCE as isize;
            let (pos_x, pos_y) = (context.position().x as isize, context.position().y as isize);

            // Calculate where the scanned_data sub-map starts in the world
            let (start_j, start_i) = match &scanned_data.scan_type {
                ScanType::Mono(orientation) => match orientation {
                    Orientation::North => (pos_x - dist / 2, pos_y - dist + 1),
                    Orientation::NorthEast => (pos_x, pos_y - dist + 1),
                    Orientation::East => (pos_x, pos_y - dist / 2),
                    Orientation::SouthEast => (pos_x, pos_y),
                    Orientation::South => (pos_x - dist / 2, pos_y),
                    Orientation::SouthWest => (pos_x - dist + 1, pos_y),
                    Orientation::West => (pos_x - dist + 1, pos_y - dist / 2),
                    Orientation::NorthWest => (pos_x - dist + 1, pos_y - dist + 1),
                },
                ScanType::Omni => (pos_x - dist / 2, pos_y - dist / 2),
            };

            // Fill our internal scanned_map with the scanned_data
            if let Some(scanned_map) = self.scanned_map.as_mut() {
                for i in 0..SCANNING_DISTANCE {
                    for j in 0..SCANNING_DISTANCE {
                        let world_y = start_i + i as isize;
                        let world_x = start_j + j as isize;

                        if world_y >= 0
                            && world_y < scanned_map.len() as isize
                            && world_x >= 0
                            && world_x < scanned_map[0].len() as isize
                        {
                            scanned_map[world_y as usize][world_x as usize] =
                                scanned_data.data[i][j];
                        }
                    }
                }
            }
        }

        if let Some(prev_position) = self.prev_position.as_ref() {
            self.scanned_map.as_mut().unwrap()[prev_position.y as usize]
                [prev_position.x as usize] = MapCell::Terrain(Terrain::Field);
        }

        // set our current position unknown
        //if let Some(scanned_map) = self.scanned_map.as_mut() {
        //    scanned_map[context.position().y as usize][context.position().x as usize] = MapCell::Unknown;
        //}

        // check for players
        let surroundings = self.check_surroundings(&context);
        if let Some(first_orientation_with_player) =
            surroundings.with_players.iter().next().cloned()
        {
            // println!("Player seen at orientation: {first_orientation_with_player:?}. ");
            // action = self.rotate_to(first_orientation_with_player, context);
            if action.is_none() {
                println!("Facing correct direction, FIRE! ");
                action = Some(Action::Fire(Aiming::Cardinal(
                    first_orientation_with_player,
                )))
            }
        }

        if action.is_none() {
            if !surroundings.with_unknown.is_empty() {
                action = Some(Action::Scan(ScanType::Omni))
            }
        }

        // check for okay cells to step on
        if action.is_none() {
            // check if our current orientation is valid to step on
            if surroundings
                .valid_to_step
                .contains(&context.player_details().orientation)
            {
                println!("Facing a direction with a valid step, move forward");
                action = Some(Action::Move(Direction::Forward))
            } else {
                // loop through valid to step and find the closest one
                let mut closest_valid_step: Option<Orientation> = None;
                let mut closest_valid_step_distance: isize = 100;
                for orientation in surroundings.valid_to_step.iter() {
                    let current = context.player_details().orientation.clone() as isize;
                    let target = orientation.clone() as isize;
                    let clockwise_steps = (target - current + 8) % 8;
                    let counter_clockwise_steps = (current - target + 8) % 8;

                    let distance = if clockwise_steps <= counter_clockwise_steps {
                        clockwise_steps
                    } else {
                        counter_clockwise_steps
                    };

                    if distance < closest_valid_step_distance {
                        closest_valid_step = Some(orientation.clone());
                        closest_valid_step_distance = distance;
                    }
                }

                if let Some(first_valid_step) = closest_valid_step {
                    print!("Closest valid step: {first_valid_step:?}. ");
                    action = self.rotate_to(first_valid_step, context);
                    if action.is_none() {
                        println!("No need to rotate, move forward");
                        action = Some(Action::Move(Direction::Forward))
                    } else {
                        println!("Rotate towards it.");
                    }
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

    fn rotate_to(&self, to: Orientation, context: Context) -> Option<Action> {
        if context.player_details().orientation == to {
            // Already facing the direction
            return None;
        }

        // Calculate the current orientation and target orientation as indices
        let current = context.player_details().orientation.clone() as isize;
        let target = to as isize;

        // Calculate the clockwise and counterclockwise steps
        let clockwise_steps = (target - current + 8) % 8;
        let counter_clockwise_steps = (current - target + 8) % 8;

        println!(
            "Rotating from {:?} to {:?}",
            context.player_details().orientation.clone(),
            target
        );

        // Choose the shorter rotation
        if clockwise_steps <= counter_clockwise_steps {
            return Some(Action::Rotate(Rotation::Clockwise));
        } else {
            return Some(Action::Rotate(Rotation::CounterClockwise));
        }
    }

    fn check_surroundings(&mut self, context: &Context) -> Surroundings {
        let mut with_players = HashSet::new();
        let mut valid_to_step = HashSet::new();
        let mut with_unknown = HashSet::new();
        let pos = context.position();
        let world_size = context.world_size();

        if let Some(scanned_map) = self.scanned_map.as_ref() {
            let directions = [
                (0, -1, Orientation::North),
                (0, 1, Orientation::South),
                (-1, 0, Orientation::West),
                (1, 0, Orientation::East),
                (-1, -1, Orientation::NorthWest),
                (1, -1, Orientation::NorthEast),
                (-1, 1, Orientation::SouthWest),
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
                    println!(
                        "{},{}: {} {new_x},{new_y} {}: {orientation:?}",
                        pos.x, pos.y, scanned_map[pos.y as usize][pos.x as usize], cell
                    );
                    match cell {
                        MapCell::Player(_, _) => {
                            with_players.insert(orientation);
                        }
                        MapCell::Unallocated => {
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
            MapCell::Terrain(Terrain::Field) => true,
            MapCell::Terrain(_) => false,
            MapCell::Player(_, _) => false,
            MapCell::Unallocated => false,
            _ => false,
        }
    }

    fn handle_movement(&self) -> Action {
        let default_prob = 0.5;
        let rotate_prob = match self.last_action {
            Action::Idle => default_prob,
            Action::Move(_) => default_prob,
            Action::Fire(_) => 0.5,
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

impl std::fmt::Display for PlayerOne {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scanned_map) = &self.scanned_map {
            for row in scanned_map.iter() {
                let mut line = String::new();
                for cell in row.iter() {
                    line = format!("{line}{}", cell);
                }
                writeln!(f, "{line}")?;
            }
        } else {
            writeln!(f, "No scanned map yet")?;
        }
        Ok(())
    }
}
