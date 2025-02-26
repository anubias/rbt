use rand::Rng;

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
        let mut rng = rand::thread_rng();
        loop {
            let position = Position {
                x: rng.gen_range(1..world_size.x - 1),
                y: rng.gen_range(1..world_size.y - 1),
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
        if let Some(path) = &path {
            let front_position = context
                .position()
                .follow(&context.player_details().orientation, context.world_size());

            let back_position = context.position().follow(
                &context.player_details().orientation.opposite(),
                context.world_size(),
            );

            if let Some(front_position) = front_position {
                if let Some(back_position) = back_position {
                    if front_position == path[0] {
                        return Some(Action::Move(Direction::Forward));
                    } else if back_position == path[0] {
                        return Some(Action::Move(Direction::Backward));
                    } else {
                        let target_orientation = context.position().get_orientation_to(&path[0]);
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
        }

        return None;
    }
}
