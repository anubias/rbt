use crate::api::{
    action::Action, context::Context, orientation::Orientation, player::Player, position::Position,
    scan::ScanType,
};

mod attack;
mod explore;
mod helpers;
mod map;

enum PlayerState {
    Start,
    Explore,
    Attack,
}

pub struct Arola {
    state: PlayerState,
    map: map::Map,
    scan_direction: Orientation,
    target_position: Option<Position>,
}

impl Arola {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Start,
            map: map::Map::new(),
            scan_direction: Orientation::North,
            target_position: None,
        }
    }
}

impl Player for Arola {
    fn act(&mut self, context: Context) -> Action {
        // Clear screen. Prevents the terminal from jumping, so makes it easier to watch the
        // simulation. Some flickering is there still though.
        // print!("{esc}c", esc = 27 as char);

        // Collect scan data to map
        if let Some(scan_result) = context.scanned_data() {
            self.map.collect_data(scan_result, context.position());
        }

        // Do processing and decide action
        let (next_state, action) = match self.state {
            PlayerState::Start => (PlayerState::Explore, Action::Scan(ScanType::Omni)),
            PlayerState::Explore => self.explore(&context),
            PlayerState::Attack => self.attack(&context),
        };

        // Set new state and return action
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
