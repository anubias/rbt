use super::player::*;
mod map;
mod scan_helper;
use map::Map;

enum PlayerState {
    Explore,
    Attack,
}

pub struct Arola {
    state: PlayerState,
    map: Map,
}

impl Arola {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Explore,
            map: Map::new(),
        }
    }
}

impl Player for Arola {
    fn act(&mut self, context: Context) -> Action {
        let (next_state, action) = match self.state {
            PlayerState::Explore => self.explore(&context),
            PlayerState::Attack => self.attack(&context),
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

impl Arola {
    fn explore(&mut self, context: &Context) -> (PlayerState, Action) {
        // Effectively scan every second round and move/rotate every second
        match context.scanned_data() {
            None => request_scan(context.orientation()),
            Some(scan_result) => self.process_explore_scan(scan_result, context),
        }
    }

    fn attack(&mut self, context: &Context) -> (PlayerState, Action) {
        if let Some(scan_result) = context.scanned_data() {
            let other_players =
                scan_result.find_other_players(context.player_id(), context.position());
            if !other_players.is_empty() {
                return (
                    PlayerState::Attack,
                    Action::Fire(Aiming::Positional(other_players[0].1.clone())),
                );
            }
        }

        return (
            PlayerState::Explore,
            Action::Scan(ScanType::Mono(context.orientation().clone())),
        );
    }

    fn process_explore_scan(
        &mut self,
        scan_result: &ScanResult,
        context: &Context,
    ) -> (PlayerState, Action) {
        self.map.collect_data(scan_result, context.position());

        if !scan_result
            .find_other_players(context.player_id(), context.position())
            .is_empty()
        {
            self.attack(context)
        } else {
            (PlayerState::Explore, self.get_moving_action(context))
        }
    }

    fn get_moving_action(&self, context: &Context) -> Action {
        let front_position = context
            .position()
            .follow(context.orientation(), context.world_size());

        let front_cell = match front_position {
            Some(position) => self.map.get_cell(&position),
            None => MapCell::Unknown,
        };

        match front_cell {
            MapCell::Player(_, _) => Action::Fire(Aiming::Cardinal(context.orientation().clone())),
            MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
            MapCell::Terrain(_) => Action::Rotate(Rotation::Clockwise),
            MapCell::Unknown => Action::Idle,
            _ => Action::default(),
        }
    }
}

fn request_scan(orientation: &Orientation) -> (PlayerState, Action) {
    (
        PlayerState::Explore,
        Action::Scan(ScanType::Mono(orientation.clone())),
    )
}
