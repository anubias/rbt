use super::player::*;

pub struct Es {
    iteration: u32,
}

impl Es {
    pub fn new() -> Self {
        Self { iteration: 0 }
    }
}

impl Player for Es {
    fn act(&mut self, context: &Context) -> Action {
        self.iteration += 1;

        if self.iteration == 1 {
            return Action::Scan(ScanType::Omni);
        }

        if let Some(scanned_data) = context.scanned_data() {
            let orientation = context.orientation();

            // The tank is at [7][7] for now
            let center = SCANNING_DISTANCE / 2;
            let mut next_cell_x = center;
            let mut next_cell_y = center;

            match orientation {
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

            let next_cell = scanned_data.data[next_cell_x][next_cell_y];

            match next_cell {
                MapCell::Field => {
                    return Action::Move(Direction::Forward);
                }
                _ => {
                    return Action::Rotate(Rotation::Clockwise);
                }
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
