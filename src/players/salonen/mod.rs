use crate::api::{
    action::Action,
    context::Context,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    player::Player,
    position::SCANNING_DISTANCE,
    rotation::Rotation,
    scan::ScanType,
    position::Position,
    scan::ScanResult,
    aiming::Aiming
};

use rand::Rng;

pub struct Es {
    iteration: u32,
    latest_scan_pos: Position,
    latest_scan_data: Option<ScanResult>,
    rotate_direction: Option<Rotation>
}

impl Es {
    pub fn new() -> Self {
        Self {
            iteration: 0,
            latest_scan_pos: Position { x: 0, y: 0 },
            latest_scan_data: None,
            rotate_direction: None,
        }
    }

    fn find_alignment(&self, from: &Position, to: &Position) -> Option<Orientation> {
        let (dx, dy) = from.manhattan_distance(to);

        match (dx, dy) {
            (0, ..0) => Some(Orientation::South),
            (0, 0..) => Some(Orientation::North),
            (..0, 0) => Some(Orientation::East),
            (0.., 0) => Some(Orientation::West),
            _ => {
                if dx.abs() == dy.abs() {
                    if dx < 0 && dy < 0 {
                        Some(Orientation::SouthEast)
                    } else if dx < 0 && dy > 0 {
                        Some(Orientation::NorthEast)
                    } else if dx > 0 && dy < 0 {
                        Some(Orientation::SouthWest)
                    } else {
                        Some(Orientation::NorthWest)
                    }
                } else {
                    None
                }
            }
        }
    }
}

impl Player for Es {
    fn act(&mut self, context: Context) -> Action {
        self.iteration += 1;
        //println!("Iteration {}", self.iteration);
        //println!("Direction {}", context.player_details().orientation);

        if self.iteration == 1 {
            return Action::Scan(ScanType::Omni);
        }

        if let Some(scanned_data) = context.scanned_data() {
            //println!("New scanned data received: {}", scanned_data);
            self.latest_scan_data = Some(scanned_data.clone());
            self.latest_scan_pos = context.position().clone();

            for y in 0..SCANNING_DISTANCE {
                for x in 0..SCANNING_DISTANCE {
                    if let MapCell::Player(_, _) = scanned_data.data[y][x] {
                        let pos = context.position();
                        //println!("Cell: {}, details: {}", scanned_data.data[y][x], det);
                        let enemy_pos = Position { x, y };
                        if pos.could_hit_cardinally(&enemy_pos) {
                            if let Some(orientation) =
                                self.find_alignment(&context.position(), &enemy_pos)
                            {
                                return Action::Fire(Aiming::Cardinal(orientation));
                            }
                        } else if pos.could_hit_positionally(&enemy_pos) {
                            return Action::Fire(Aiming::Positional(enemy_pos));
                        }
                    }
                }
            }
        }

        if let Some(scanned_data) = &self.latest_scan_data {
            //println!("Old scanned data exists");
            let pos = context.position();

            // The tank is at [7][7] for now
            let center = SCANNING_DISTANCE / 2;
            let mut next_cell_x = center + pos.x - self.latest_scan_pos.x;
            let mut next_cell_y = center + pos.y - self.latest_scan_pos.y;

            // Could use follow
            match context.player_details().orientation {
                Orientation::North => {
                    next_cell_y -= 1;
                }
                Orientation::NorthWest => {
                    next_cell_y -= 1;
                    next_cell_x -= 1;
                }
                Orientation::West => {
                    next_cell_x -= 1;
                }
                Orientation::SouthWest => {
                    next_cell_x -= 1;
                    next_cell_y += 1;
                }
                Orientation::South => {
                    next_cell_y += 1;
                }
                Orientation::SouthEast => {
                    next_cell_y += 1;
                    next_cell_x += 1;
                }
                Orientation::East => {
                    next_cell_x += 1;
                }
                Orientation::NorthEast => {
                    next_cell_x += 1;
                    next_cell_y -= 1;
                }
            }

            if next_cell_x > 0 && next_cell_x < scanned_data.data.len() && next_cell_y > 0 && next_cell_y < scanned_data.data[0].len() {
                let next_cell = scanned_data.data[next_cell_y][next_cell_x];
                //println!("Next cell {}, {}: {}", next_cell_x, next_cell_y, next_cell);

                match next_cell {
                    MapCell::Player(_, _) => {
                        self.rotate_direction = None;
                        return Action::Move(Direction::Forward);
                    }
                    MapCell::Terrain(Terrain::Field) => {
                        self.rotate_direction = None;
                        return Action::Move(Direction::Forward);
                    }
                    _ => {
                        if self.rotate_direction.is_none()
                        {
                            let mut rng = rand::rng();
                            let dir = if rng.random_bool(0.5)
                            {
                                Rotation::Clockwise
                            } else {
                                Rotation::CounterClockwise
                            };

                            self.rotate_direction = Some(dir);
                        }
                        return Action::Rotate(self.rotate_direction.as_ref().unwrap_or(&Rotation::Clockwise).clone());
                    }
                }
            }
            else {
                self.rotate_direction = None;
                //println!("Going outside of the scanned area. Doing a rescan");
                return Action::Scan(ScanType::Omni);
            }
        }

        /*match context.position() {
            _ => {}
        }*/

        Action::default()
    }

    fn name(&self) -> String {
        "ES".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
