use super::player::*;

pub struct Arola {}

impl Arola {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for Arola {
    fn act(&mut self, context: Context) -> Action {
        match context.scanned_data() {
            None => Action::Scan(ScanType::Directional(context.orientation().clone())),
            Some(scan_data) => get_moving_action(scan_data, context.orientation()),
        }
    }

    fn name(&self) -> String {
        "Arola".to_string()
    }
}

fn get_moving_action(scan_data: &ScanResult, heading: &Orientation) -> Action {
    let front_cell = match heading {
        Orientation::North => scan_data.data[SCANNING_DISTANCE - 2][SCANNING_DISTANCE / 2],
        Orientation::NorthEast => scan_data.data[SCANNING_DISTANCE - 2][1],
        Orientation::East => scan_data.data[SCANNING_DISTANCE / 2][1],
        Orientation::SouthEast => scan_data.data[1][1],
        Orientation::South => scan_data.data[1][SCANNING_DISTANCE / 2],
        Orientation::SouthWest => scan_data.data[1][SCANNING_DISTANCE - 2],
        Orientation::West => scan_data.data[SCANNING_DISTANCE / 2][SCANNING_DISTANCE - 2],
        Orientation::NorthWest => scan_data.data[SCANNING_DISTANCE - 2][SCANNING_DISTANCE - 2],
    };

    match front_cell {
        MapCell::Player(_, _, _) => Action::Move(Direction::Forward), // Crash to the other player
        MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
        MapCell::Terrain(_) => Action::Rotate(Rotation::Clockwise),
        MapCell::Unknown => Action::Idle,
    }
}
