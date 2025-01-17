use super::player::*;

#[derive(Default)]
pub struct Luis{
    last_action: Action,
    map: Vec<Vec<u32>>,
}

impl Luis {
    pub fn new() -> Self {
        Self {
            last_action: Action::default(),
            map: Vec::<Vec<u32>>::new()
        }
    }
}

impl Player for Luis {
    fn act(&mut self, context: &Context) -> Action {
        // Just scanned
        if let Some(scan_data) = context.scanned_data() {

            // Cache map data (just if uncomplete?)


            let orientation = context.orientation();
            let center = SCANNING_DISTANCE / 2;
            let mut next_cell_x = center;
            let mut next_cell_y = center;

            match orientation {
                Orientation::North => next_cell_y -= 1,
                Orientation::NorthWest => {
                    next_cell_y -= 1;
                    next_cell_x -= 1;
                }
                Orientation::West => next_cell_x -= 1,
                Orientation::SouthWest => {
                    next_cell_x -= 1;
                    next_cell_y += 1;
                }
                Orientation::South => next_cell_y += 1,
                Orientation::SouthEast => {
                    next_cell_y += 1;
                    next_cell_x += 1;
                }
                Orientation::East => next_cell_x += 1,
                Orientation::NorthEast => {
                    next_cell_x += 1;
                    next_cell_y -= 1;
                }
            }

            let next_cell = scan_data.data[next_cell_x][next_cell_y];

            match next_cell {
                MapCell::Field => Action::Move(Direction::Forward),
                MapCell::Lake | MapCell::Mountain | MapCell::Swamp => Action::Rotate(Rotation::Clockwise),
                MapCell::Player(_) => Action::Fire,
                MapCell::Unknown => Action::Scan(ScanType::Omni),
            }
        } else {
            Action::Scan(ScanType::Omni)
        }
    }

    fn name(&self) -> String {
        "LPlayer".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
