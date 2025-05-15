use std::collections::HashMap;
use std::collections::HashSet;

use rand::Rng;

use crate::api::{
    action::Action,
    aiming::Aiming,
    context::Context,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    player::{Details, Player},
    position::{Position, SCANNING_DISTANCE},
    rotation::Rotation,
    scan::{ScanResult, ScanType},
};

const PLAYER_LIFETIME: u8 = 3;
const SCAN_EVERY_NTH_STEP: u8 = 1;
const DEBUG_PRINTS: bool = false;
const CORNER_OFFSET_X: usize = 6;
const CORNER_OFFSET_Y: usize = 4;
const CORNER_CAMPING_LIMIT: u32 = 100;

pub struct DetectedPlayer {
    position: Position,
    details: Details,
    lifetime: u8,
}

pub struct Surroundings {
    pub valid_to_step: HashSet<Orientation>,
    pub with_unknown: HashSet<Orientation>,
}

pub struct PlayerOne {
    last_action: Action,
    last_scan: Option<ScanResult>,
    scan_counter: u8,
    scan_wall_counter: u8,
    own_map: Option<Box<Vec<Vec<MapCell>>>>,
    counter: u32,
    prev_position: Option<Position>,
    detected_players: HashMap<Details, DetectedPlayer>,
    turns_without_seeing_a_player: u32,
}

impl Player for PlayerOne {
    fn act(&mut self, context: Context) -> Action {
        self.counter += 1;
        // handle scanning and firing
        let mut have_an_action = true;
        let pos_x = context.position().x;
        let pos_y = context.position().y;

        let mut action = if let Some(scan_action) = self.do_something(context) {
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
        if DEBUG_PRINTS {
            println!("ACT {}, {action:?}", self.counter);
        }
        //println!("{self}");
        self.prev_position = Some(Position { x: pos_x, y: pos_y });

        self.update_lifetimes();
        action
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
            last_scan: None,
            own_map: None,
            counter: 0,
            prev_position: None,
            detected_players: HashMap::new(),
            scan_counter: SCAN_EVERY_NTH_STEP,
            scan_wall_counter: 0,
            turns_without_seeing_a_player: 0,
        }
    }

    fn update_world_and_players_from_scanned_data(&mut self, context: &Context) {
        if let Some(scanned_data) = context.scanned_data() {
            let dist = SCANNING_DISTANCE as isize;
            let (my_world_pos_x, my_world_pos_y) =
                (context.position().x as isize, context.position().y as isize);

            // Calculate where the scanned_data sub-map starts in the world
            let (start_j, start_i) = match &scanned_data.scan_type {
                ScanType::Mono(orientation) => match orientation {
                    Orientation::North => (my_world_pos_x - dist / 2, my_world_pos_y - dist + 1),
                    Orientation::NorthEast => (my_world_pos_x, my_world_pos_y - dist + 1),
                    Orientation::East => (my_world_pos_x, my_world_pos_y - dist / 2),
                    Orientation::SouthEast => (my_world_pos_x, my_world_pos_y),
                    Orientation::South => (my_world_pos_x - dist / 2, my_world_pos_y),
                    Orientation::SouthWest => (my_world_pos_x - dist + 1, my_world_pos_y),
                    Orientation::West => (my_world_pos_x - dist + 1, my_world_pos_y - dist / 2),
                    Orientation::NorthWest => {
                        (my_world_pos_x - dist + 1, my_world_pos_y - dist + 1)
                    }
                },
                ScanType::Omni => (my_world_pos_x - dist / 2, my_world_pos_y - dist / 2),
            };

            // Fill our internal scanned_map with the scanned_data
            self.turns_without_seeing_a_player += 1;
            if let Some(scanned_map) = self.own_map.as_mut() {
                for i in 0..SCANNING_DISTANCE {
                    for j in 0..SCANNING_DISTANCE {
                        let world_y = start_i + i as isize;
                        let world_x = start_j + j as isize;

                        let mut cell = MapCell::Unallocated;

                        // check if we found a player
                        if let MapCell::Player(details, terrain) = scanned_data.data[i][j] {
                            if details.alive
                                && (world_x != my_world_pos_x && world_y != my_world_pos_y)
                            {
                                self.turns_without_seeing_a_player = 0;
                                if DEBUG_PRINTS {
                                    println!("Found a player at: ({world_x}, {world_y})");
                                }
                                cell = MapCell::Terrain(terrain);

                                // Add or update the detected player in the bookkeeping
                                let detected_player = DetectedPlayer {
                                    position: Position {
                                        x: world_x as usize,
                                        y: world_y as usize,
                                    },
                                    details: details,
                                    lifetime: PLAYER_LIFETIME,
                                };
                                self.detected_players.insert(details, detected_player);
                            } else if DEBUG_PRINTS {
                                if !details.alive {
                                    println!("Found a dead player at: ({world_x}, {world_y})");
                                }
                            }
                        }

                        if world_y >= 0
                            && world_y < scanned_map.len() as isize
                            && world_x >= 0
                            && world_x < scanned_map[0].len() as isize
                        {
                            if cell == MapCell::Unallocated {
                                cell = scanned_data.data[i][j];
                            }
                            scanned_map[world_y as usize][world_x as usize] = cell;
                        }
                    }
                }
            }
        }
    }

    fn act_on_players(&mut self, context: &Context) -> Option<Action> {
        let mut action: Option<Action> = None;
        if let Some(detected_player) = self.get_newest_player_at_firing_distance(&context) {
            let dx = detected_player.position.x as isize - context.position().x as isize;
            let dy = detected_player.position.y as isize - context.position().y as isize;

            if DEBUG_PRINTS {
                println!("Detected player at dx {}, dy {}", dx, dy);
            }
            if dx == 0 || dy == 0 || dx == dy {
                let orientation = if dx == 0 {
                    if dy > 0 {
                        Orientation::South
                    } else {
                        Orientation::North
                    }
                } else if dy == 0 {
                    if dx > 0 {
                        Orientation::East
                    } else {
                        Orientation::West
                    }
                } else if dx > 0 {
                    if dy > 0 {
                        Orientation::SouthEast
                    } else {
                        Orientation::NorthEast
                    }
                } else {
                    if dy > 0 {
                        Orientation::SouthWest
                    } else {
                        Orientation::NorthWest
                    }
                };
                action = Some(Action::Fire(Aiming::Cardinal(orientation)));
            } else if context.position().could_hit_positionally(&detected_player.position) {
                action = Some(Action::Fire(Aiming::Positional(
                    detected_player.position.clone(),
                )));
            }
        }
        action
    }

    fn act_on_corner(&mut self, context: &Context) -> Option<Action> {
        let mut action: Option<Action> = None;
        if self.in_corner(context) {
            // figure out scan direction when in corner
            let my_x = context.position().x;
            let my_y = context.position().y;

            if my_x == 1 {
                if my_y == 1 {
                    action = Some(Action::Scan(ScanType::Mono(Orientation::SouthEast)));
                } else {
                    action = Some(Action::Scan(ScanType::Mono(Orientation::NorthEast)));
                }
            } else if my_y == 1 {
                action = Some(Action::Scan(ScanType::Mono(Orientation::SouthWest)));
            } else {
                action = Some(Action::Scan(ScanType::Mono(Orientation::NorthWest)));
            }
        }
        action
    }

    fn act_on_scanning(&mut self, context: &Context) -> Option<Action> {
        let mut action: Option<Action> = None;
        let mut scanned_wall = true;
        if self.in_west_wall(context) {
            // Alternate between NorthEast and SouthEast
            if self.scan_wall_counter == 0 {
                action = Some(Action::Scan(ScanType::Mono(Orientation::NorthEast)));
            } else {
                action = Some(Action::Scan(ScanType::Mono(Orientation::SouthEast)));
            }
        } else if self.in_east_wall(context) {
            // Alternate between NorthWest and SouthWest
            if self.scan_wall_counter == 0 {
                action = Some(Action::Scan(ScanType::Mono(Orientation::NorthWest)));
            } else {
                action = Some(Action::Scan(ScanType::Mono(Orientation::SouthWest)));
            }
        } else if self.in_north_wall(context) {
            // Alternate between SouthEast and SouthWest
            if self.scan_wall_counter == 0 {
                action = Some(Action::Scan(ScanType::Mono(Orientation::SouthEast)));
            } else {
                action = Some(Action::Scan(ScanType::Mono(Orientation::SouthWest)));
            }
        } else if self.in_south_wall(context) {
            // Alternate between NorthEast and NorthWest
            if self.scan_wall_counter == 0 {
                action = Some(Action::Scan(ScanType::Mono(Orientation::NorthEast)));
            } else {
                action = Some(Action::Scan(ScanType::Mono(Orientation::NorthWest)));
            }
        } else {
            action = Some(Action::Scan(ScanType::Omni));
            scanned_wall = false;
        }
        let mut reset_counter = true;
        if scanned_wall {
            self.scan_wall_counter += 1;
            if self.scan_wall_counter == 2 {
                self.scan_wall_counter = 0;
            } else {
                // next round scan the other direction
                reset_counter = false;
            }
        }
        if reset_counter {
            self.scan_counter = SCAN_EVERY_NTH_STEP;
        }
        action
    }

    /// Handles the logic for scanning the surroundings.
    ///
    /// This function checks the scanned data for any players in the surroundings.
    /// If it finds any, it rotates towards the first one it found and fires.
    /// If there are no players, it checks for any valid cells to step on.
    /// If it finds any valid cells, it rotates towards the first one it found and moves forward.
    /// If there are no valid cells, it scans the surroundings again.
    /// The function returns an action based on the logic above, or None if no action was found.
    fn do_something(&mut self, context: Context) -> Option<Action> {
        let mut action: Option<Action> = None;
        if self.own_map.is_none() {
            let world_size = context.world_size();
            self.own_map = Some(Box::new(vec![
                vec![MapCell::Unallocated; world_size.x];
                world_size.y
            ]));
        }

        if DEBUG_PRINTS {
            println!(
                "Position: {:?}, current orientation: {:?},",
                context.position(),
                context.player_details().orientation
            );
        }

        self.update_world_and_players_from_scanned_data(&context);

        // reset my previous position to field
        if let Some(prev_position) = self.prev_position.as_ref() {
            self.own_map.as_mut().unwrap()[prev_position.y as usize][prev_position.x as usize] =
                MapCell::Terrain(Terrain::Field);
        }

        if action.is_none() {
            action = self.act_on_players(&context);
        }

        if action.is_none() && self.turns_without_seeing_a_player < CORNER_CAMPING_LIMIT {
            action = self.act_on_corner(&context);
        }

        if self.scan_counter == 0 {
            if action.is_none() {
                action = self.act_on_scanning(&context);
            }
        } else {
            self.scan_counter -= 1;
        }
        // check surroundings and move
        if action.is_none() {
            let surroundings = self.check_surroundings(&context);

            if !surroundings.with_unknown.is_empty() {
                action = Some(Action::Scan(ScanType::Omni))
            }

            // check for okay cells to step on
            if action.is_none() {
                // check if our current orientation is valid to step on
                if surroundings
                    .valid_to_step
                    .contains(&context.player_details().orientation)
                {
                    if DEBUG_PRINTS {
                        println!("Facing a direction with a valid step, move forward");
                    }
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
                        if DEBUG_PRINTS {
                            print!("Closest valid step: {first_valid_step:?}. ");
                        }
                        action = self.rotate_to(first_valid_step, context);
                        if action.is_none() {
                            if DEBUG_PRINTS {
                                println!("No need to rotate, move forward");
                            }
                            action = Some(Action::Move(Direction::Forward))
                        } else {
                            if DEBUG_PRINTS {
                                println!("Rotate towards it.");
                            }
                        }
                    }
                }
            }
        }

        if action.is_none() {
            action = Some(Action::Scan(ScanType::Omni))
        }

        action
    }

    pub fn get_newest_player_at_firing_distance(
        &self,
        context: &Context,
    ) -> Option<&DetectedPlayer> {
        // check for players using self.detected_players
        let mut newest_player_at_firing_distance: Option<(Details, &DetectedPlayer)> = None;
        let mut newest_player_lifetime: u8 = 0;
        let max_distance = match self.in_corner(context) {
            true => SCANNING_DISTANCE,
            false => SCANNING_DISTANCE / 2,
        };
        for (player_details, detected_player) in self.detected_players.iter() {
            let y_distance =
                (detected_player.position.y as isize - context.position().y as isize).abs();
            let x_distance =
                (detected_player.position.x as isize - context.position().x as isize).abs();
            if DEBUG_PRINTS {
                println!(
                    "Checking player {},{} at distance: {y_distance}, {x_distance}. Max distance: {max_distance}",
                    detected_player.position.x,
                    detected_player.position.y
                );
            }
            if context.position().could_hit_positionally(&detected_player.position) || context.position().could_hit_cardinally(&detected_player.position) {
                if player_details.alive && detected_player.lifetime > newest_player_lifetime {
                    newest_player_lifetime = detected_player.lifetime;
                    newest_player_at_firing_distance =
                        Some((player_details.clone(), detected_player));
                }
            }
        }

        if let Some((player_details, detected_player)) = newest_player_at_firing_distance {
            if DEBUG_PRINTS {
                println!("Found a player at firing distance: {player_details:?}. ");
            }
            return Some(detected_player);
        }

        None
    }

    pub fn update_lifetimes(&mut self) {
        let mut removed = false;
        self.detected_players.retain(|_, player| {
            if player.details.alive && player.lifetime > 0 {
                player.lifetime -= 1;
                player.lifetime > 0
            } else {
                removed = true;
                false
            }
        });
        if removed {
            self.scan_counter = 0;
        }
    }

    pub fn in_west_wall(&self, context: &Context) -> bool {
        context.position().x == 1
    }

    pub fn in_east_wall(&self, context: &Context) -> bool {
        context.position().x == context.world_size().x - CORNER_OFFSET_X
    }

    pub fn in_north_wall(&self, context: &Context) -> bool {
        context.position().y == 1
    }

    pub fn in_south_wall(&self, context: &Context) -> bool {
        context.position().y == context.world_size().y / 2 - CORNER_OFFSET_Y
    }

    pub fn in_corner(&self, context: &Context) -> bool {
        (self.in_west_wall(context) && self.in_north_wall(context))
            || (self.in_east_wall(context) && self.in_south_wall(context))
            || (self.in_west_wall(context) && self.in_south_wall(context))
            || (self.in_east_wall(context) && self.in_north_wall(context))
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

        if DEBUG_PRINTS {
            println!(
                "Rotating from {:?} to {:?}",
                context.player_details().orientation.clone(),
                target
            );
        }

        // Choose the shorter rotation
        if clockwise_steps <= counter_clockwise_steps {
            return Some(Action::Rotate(Rotation::Clockwise));
        } else {
            return Some(Action::Rotate(Rotation::CounterClockwise));
        }
    }

    fn check_surroundings(&mut self, context: &Context) -> Surroundings {
        let mut valid_to_step = HashSet::new();
        let mut with_unknown = HashSet::new();
        let pos = context.position();
        let world_size = context.world_size();

        if let Some(scanned_map) = self.own_map.as_ref() {
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
                    if DEBUG_PRINTS {
                        // println!(
                        //    "{},{}: {} {new_x},{new_y} {}: {orientation:?}",
                        //    pos.x, pos.y, scanned_map[pos.y as usize][pos.x as usize], cell
                        //);
                    }
                    match cell {
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
        if let Some(scanned_map) = &self.own_map {
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
