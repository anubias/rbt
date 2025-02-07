use super::super::player::*;
use super::Arola;
use super::PlayerState;

impl Arola {
    pub(super) fn explore(&mut self, context: &Context) -> (PlayerState, Action) {
        // Effectively scan every second round and move/rotate every second
        match context.scanned_data() {
            None => self.request_scan(),
            Some(scan_result) => self.process_explore_scan(scan_result, context),
        }
    }

    fn request_scan(&mut self) -> (PlayerState, Action) {
        // Scan in radar style: NE => SE => SW => NW => NE => ...
        // TODO: If at the edge of the world, some of the scan directions would not make sense
        let scan_direction = match self.previous_scan_direction {
            Orientation::NorthEast => Orientation::SouthEast,
            Orientation::SouthEast => Orientation::SouthWest,
            Orientation::SouthWest => Orientation::NorthWest,
            Orientation::NorthWest => Orientation::NorthEast,
            _ => Orientation::NorthEast,
        };

        self.previous_scan_direction = scan_direction.clone();

        return (
            PlayerState::Explore,
            Action::Scan(ScanType::Mono(scan_direction)),
        );
    }

    fn process_explore_scan(
        &mut self,
        scan_result: &ScanResult,
        context: &Context,
    ) -> (PlayerState, Action) {
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
            MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
            MapCell::Terrain(_) => Action::Rotate(Rotation::Clockwise),
            _ => Action::Idle,
        }
    }
}
