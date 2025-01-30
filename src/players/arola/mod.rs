use super::player::*;

pub struct Arola {
    state: PlayerState,
}

enum PlayerState {
    Explore,
    Attack,
}

impl Arola {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Explore,
        }
    }
}

impl Player for Arola {
    fn act(&mut self, context: Context) -> Action {
        let (next_state, action) = match self.state {
            PlayerState::Explore => explore(&context),
            PlayerState::Attack => attack(&context),
        };

        self.state = next_state;
        return action;
    }

    fn name(&self) -> String {
        "Arola".to_string()
    }

    fn is_ready(&self) -> bool {
        false
    }
}

fn explore(context: &Context) -> (PlayerState, Action) {
    // Effectively scan every second round and act every second
    match context.scanned_data() {
        None => request_scan(context),
        Some(_) => process_scan_and_act(context),
    }
}

fn request_scan(context: &Context) -> (PlayerState, Action) {
    (
        PlayerState::Explore,
        Action::Scan(ScanType::Mono(context.orientation().clone())),
    )
}

fn process_scan_and_act(context: &Context) -> (PlayerState, Action) {
    if let Some(scan_data) = context.scanned_data() {
        if find_other_players(scan_data) {
            attack(context)
        } else {
            (
                PlayerState::Explore,
                get_moving_action(scan_data, context.orientation()),
            )
        }
    } else {
        panic!("No scan data available")
    }
}

fn get_my_position_in_scan(scan_data: &ScanResult) -> (usize, usize) {
    match &scan_data.scan_type {
        ScanType::Mono(orientation) => match orientation {
            Orientation::North => (SCANNING_DISTANCE - 1, SCANNING_DISTANCE / 2),
            Orientation::NorthEast => (SCANNING_DISTANCE - 1, 0),
            Orientation::East => (SCANNING_DISTANCE / 2, 0),
            Orientation::SouthEast => (0, 0),
            Orientation::South => (0, SCANNING_DISTANCE / 2),
            Orientation::SouthWest => (0, SCANNING_DISTANCE - 1),
            Orientation::West => (SCANNING_DISTANCE / 2, SCANNING_DISTANCE - 1),
            Orientation::NorthWest => (SCANNING_DISTANCE - 1, SCANNING_DISTANCE - 1),
        },
        ScanType::Omni => (SCANNING_DISTANCE / 2, SCANNING_DISTANCE / 2),
    }
}

fn find_other_players(scan_data: &ScanResult) -> bool {
    let my_position = get_my_position_in_scan(scan_data);

    for i in 1..scan_data.data.len() - 1 {
        for j in 1..scan_data.data[i].len() - 1 {
            if matches!(scan_data.data[i][j], MapCell::Player(_, _)) && (i, j) != my_position {
                return true;
            }
        }
    }

    return false;
}

fn attack(context: &Context) -> (PlayerState, Action) {
    if let Some(scan_data) = context.scanned_data() {
        if find_other_players(scan_data) {
            println!("Attack");
            return (
                PlayerState::Attack,
                Action::Scan(ScanType::Mono(context.orientation().clone())),
            );
        }
    }

    return (
        PlayerState::Explore,
        Action::Scan(ScanType::Mono(context.orientation().clone())),
    );
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
        MapCell::Player(_, _) => Action::Fire(Aiming::Cardinal(heading.clone())),
        MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
        MapCell::Terrain(_) => Action::Rotate(Rotation::Clockwise),
        MapCell::Unknown => Action::Idle,
    }
}
