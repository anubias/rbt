use rand::Rng;

use crate::api::{
    action::Action,
    context::Context,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    position::Position,
    scan::{ScanResult, ScanType},
    world_size::WorldSize,
};

use super::Arola;
use super::PlayerState;

impl Arola {
    pub(super) fn explore(&mut self, context: &Context) -> (PlayerState, Action) {
        // Effectively scan every second round and move/rotate every second
        match context.scanned_data() {
            None => self.request_scan(context.position(), context.world_size()),
            Some(scan_result) => self.process_explore_scan(scan_result, context),
        }
    }

    fn request_scan(
        &mut self,
        position: &Position,
        world_size: &WorldSize,
    ) -> (PlayerState, Action) {
        // Scan in radar style: NE => SE => SW => NW => NE => ...
        // But skip non-meaningful scan directions at the edge of the world
        fn accept_scan_direction(
            scan_direction: &Orientation,
            position: &Position,
            world_size: &WorldSize,
        ) -> bool {
            let limit_min = 1;
            let limit_x_max = world_size.x - 2;
            let limit_y_max = world_size.y - 2;

            match scan_direction {
                Orientation::NorthEast => position.x < limit_x_max && position.y > limit_min,
                Orientation::SouthEast => position.x < limit_x_max && position.y < limit_y_max,
                Orientation::SouthWest => position.x > limit_min && position.y < limit_y_max,
                Orientation::NorthWest => position.x > limit_min && position.y > limit_min,
                _ => false,
            }
        }

        loop {
            self.scan_direction = self.scan_direction.rotated_clockwise();
            if accept_scan_direction(&self.scan_direction, position, world_size) {
                return (
                    PlayerState::Explore,
                    Action::Scan(ScanType::Mono(self.scan_direction)),
                );
            }
        }
    }

    fn process_explore_scan(
        &mut self,
        scan_result: &ScanResult,
        context: &Context,
    ) -> (PlayerState, Action) {
        if !scan_result
            .find_other_players(context.player_details().id, context.position())
            .is_empty()
        {
            self.attack(context)
        } else {
            (PlayerState::Explore, self.get_moving_action(context))
        }
    }

    fn get_moving_action(&mut self, context: &Context) -> Action {
        if self.target_position.is_none() {
            self.target_position = Some(self.generate_random_position(context.world_size()));
        }

        let path = self.target_position.as_ref().and_then(|target_position| {
            self.map.find_path(
                context.position().clone(),
                context.player_details().orientation.clone(),
                target_position,
            )
        });

        if let Some(action) = self.move_along_path(context, &path) {
            return action;
        } else {
            // No valid moving action => clear target position to try again in future
            self.target_position = None;
            return Action::Idle;
        }
    }

    fn generate_random_position(&self, world_size: &WorldSize) -> Position {
        let mut rng = rand::rng();
        loop {
            let position = Position {
                x: rng.random_range(1..world_size.x - 1),
                y: rng.random_range(1..world_size.y - 1),
            };

            if matches!(
                self.map.get_cell(&position),
                MapCell::Terrain(Terrain::Field) | MapCell::Unallocated
            ) {
                return position;
            }
        }
    }

    fn move_along_path(&self, context: &Context, path: &Option<Vec<Position>>) -> Option<Action> {
        if path.as_ref().is_none_or(|path| path.is_empty()) {
            return None;
        }

        let target_position = path.as_ref().unwrap().first().unwrap();

        let front_position = context
            .position()
            .follow(&context.player_details().orientation, context.world_size())
            .expect("Should always be in position where front position exists");

        let back_position = context
            .position()
            .follow(
                &context.player_details().orientation.opposite(),
                context.world_size(),
            )
            .expect("Should always be in position where back position exists");

        let try_move = |position: &Position,
                        orientation: Orientation,
                        direction: Direction|
         -> Option<Action> {
            match self.map.get_cell(position) {
                // Cell is unknown, request for scan
                MapCell::Unallocated => Some(Action::Scan(ScanType::Mono(orientation))),
                // Cell is ok to enter, move there
                MapCell::Terrain(Terrain::Field) => Some(Action::Move(direction)),
                // Cell is not ok to enter, this should not happen (bug in path finding)
                _ => panic!("Can't enter cell {position}"),
            }
        };

        if &front_position == target_position {
            return try_move(
                &front_position,
                context.player_details().orientation,
                Direction::Forward,
            );
        } else if &back_position == target_position {
            return try_move(
                &back_position,
                context.player_details().orientation.opposite(),
                Direction::Backward,
            );
        } else {
            let target_orientation = context.position().get_orientation_to(target_position);
            return Some(Action::Rotate(
                context
                    .player_details()
                    .orientation
                    .quick_turn_bidirectional(&target_orientation)
                    .0,
            ));
        }
    }
}
