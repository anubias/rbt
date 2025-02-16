use super::player::*;

pub struct Siimesjarvi {
    strategy: Strategy,
}

const MIDDLE_COORDINATE: usize = SCANNING_DISTANCE / 2;

impl Siimesjarvi {
    pub fn new() -> Self {
        Self {
            strategy: Strategy::Advanced(AdvancedStrategy::new()),
        }
    }
}

impl Player for Siimesjarvi {
    fn act(&mut self, context: Context) -> Action {
        match &mut self.strategy {
            Strategy::Basic(x) => x.get_next_action(context),
            Strategy::Advanced(x) => x.get_next_action(&context),
        }
    }

    fn name(&self) -> String {
        "Joni Siimesjärvi 🐙".to_string()
    }
}

#[allow(dead_code)]
enum Strategy {
    Basic(BasicStrategy),
    Advanced(AdvancedStrategy),
}

/// Basic strategy to get started
///
/// Blindly fire in all directions in clockwise pattern
#[derive(Default)]
struct BasicStrategy {
    orientation: Orientation,
}

impl BasicStrategy {
    /// Provides Orientation that should be used next
    ///
    /// Updates the next orientation so repeatedly calling this will eventually
    /// give all orientations
    fn get_next_orientation(&mut self) -> Orientation {
        match self.orientation {
            Orientation::North => {
                self.orientation = Orientation::NorthEast;
                Orientation::North
            }
            Orientation::NorthEast => {
                self.orientation = Orientation::East;
                Orientation::NorthEast
            }
            Orientation::East => {
                self.orientation = Orientation::SouthEast;
                Orientation::East
            }
            Orientation::SouthEast => {
                self.orientation = Orientation::South;
                Orientation::SouthEast
            }
            Orientation::South => {
                self.orientation = Orientation::SouthWest;
                Orientation::South
            }
            Orientation::SouthWest => {
                self.orientation = Orientation::West;
                Orientation::SouthWest
            }
            Orientation::West => {
                self.orientation = Orientation::NorthWest;
                Orientation::West
            }
            Orientation::NorthWest => {
                self.orientation = Orientation::North;
                Orientation::NorthWest
            }
        }
    }

    pub fn get_next_action(&mut self, _context: Context) -> Action {
        Action::Fire(Aiming::Cardinal(self.get_next_orientation()))
    }
}

#[allow(dead_code)]
struct AdvancedStrategy {
    previous_action: Action,
    world_map: [MapCell; MAX_WORLD_SIZE * MAX_WORLD_SIZE],
}

struct PlayerInMap {
    player_details: PlayerDetails,
    x: usize,
    y: usize,
}

type PointPair = ((isize, isize), (isize, isize));

#[allow(dead_code)]
impl AdvancedStrategy {
    fn new() -> Self {
        Self {
            previous_action: Action::Idle,
            world_map: [MapCell::Unallocated; MAX_WORLD_SIZE * MAX_WORLD_SIZE],
        }
    }

    fn are_other_players_in_scan_result(
        &self,
        my_player_id: PlayerId,
        scan_result: &ScanResult,
    ) -> bool {
        for row in scan_result.data.iter() {
            for map_cell in row.iter() {
                match map_cell {
                    MapCell::Player(player_details, ..) => {
                        if player_details.id != my_player_id && player_details.alive {
                            return true;
                        }
                    }
                    _ => (),
                }
            }
        }
        false
    }

    fn get_players_from_scan_result(&self, scan_result: &ScanResult) -> Vec<PlayerInMap> {
        let mut players: Vec<PlayerInMap> = Vec::new();
        for y in 0..SCANNING_DISTANCE {
            for x in 0..SCANNING_DISTANCE {
                // Coordinates are reversed as the array indexes
                match scan_result.data[y][x] {
                    MapCell::Player(player_details, ..) => { 
                        players.push(PlayerInMap { player_details: player_details.clone(), x: x, y: y })
                     }
                    _ => ()
                }
            }
        }
        players
    }

    fn get_my_coordinates_from_submap(
        &self,
        my_player_id: PlayerId,
        players: &Vec<PlayerInMap>,
    ) -> Option<(isize, isize)> {
        for player in players.iter() {
            if player.player_details.id == my_player_id {
                return Option::Some((player.x as isize, player.y as isize));
            }
        }
        Option::None
    }

    fn get_any_other_player_coordinates_from_submap(
        &self,
        my_player_id: PlayerId,
        players: &Vec<PlayerInMap>,
    ) -> Option<(isize, isize)> {
        for player in players.iter() {
            if player.player_details.id != my_player_id && player.player_details.alive {
                return Option::Some((player.x as isize, player.y as isize));
            }
        }
        Option::None
    }

    fn calculate_relative_position(
        &self,
        my_position: (isize, isize),
        other_position: (isize, isize),
    ) -> (isize, isize) {
        (
            other_position.0 - my_position.0,
            other_position.1 - my_position.1,
        )
    }

    fn calculate_firing_position(&self, my_position: &Position, delta: (isize, isize)) -> Position {
        let new_x = (my_position.x as isize + delta.0) as usize;
        let new_y = (my_position.y as isize + delta.1) as usize;
        Position { x: new_x, y: new_y }
    }

    fn handle_scan_result_with_other_players(&self, scan_result: &ScanResult, context: &Context) -> Option<Action> {
        let players_in_scan_result = self.get_players_from_scan_result(scan_result);

        // Idle action should not be returned from here
        // As at this point we should always have other player + our own player in result
        let my_position = self.get_my_coordinates_from_submap(
            context.player_details().id,
            &players_in_scan_result,
        )?;

        let other_player_position = self.get_any_other_player_coordinates_from_submap(
            context.player_details().id,
            &players_in_scan_result,
        )?;

        let delta = self.calculate_relative_position(my_position, other_player_position);
        let pos = self.calculate_firing_position(context.position(), delta);

        Option::Some(Action::Fire(Aiming::Positional(pos)))
    }

    fn handle_scan_result_without_other_players(&self, scan_result: &ScanResult, context: &Context) -> Action {

        let potential_moves = match &context.player_details().orientation {
            Orientation::North => ((0, -1), (0, 1)),
            Orientation::NorthEast => ((1, -1), (-1, 1)),
            Orientation::East => ((1, 0), (-1, 0)),
            Orientation::SouthEast => ((1, 1), (-1, -1)),
            Orientation::South => ((0, 1), (0, -1)),
            Orientation::SouthWest => ((-1, 1), (1, -1)),
            Orientation::West => ((-1, 0), (1, 0)),
            Orientation::NorthWest => ((-1, -1), (1, 1)),
        };

        let forward_x = MIDDLE_COORDINATE as isize + potential_moves.0.0;
        let forward_y = MIDDLE_COORDINATE as isize + potential_moves.0.1;
        if self.is_submap_cell_safe(scan_result, forward_x as usize, forward_y as usize) {
            return Action::Move(Direction::Forward)
        } 

        let backward_x = MIDDLE_COORDINATE as isize + potential_moves.1.0;
        let backward_y = MIDDLE_COORDINATE as isize + potential_moves.1.1;
        if self.is_submap_cell_safe(scan_result, backward_x as usize, backward_y as usize) {
            Action::Move(Direction::Backward)
        } else {
            Action::Rotate(Rotation::Clockwise)
        }
    }

    fn is_submap_cell_safe(&self, scan_result: &ScanResult, x: usize, y: usize) -> bool {
        match scan_result.data[y][x] {
            MapCell::Terrain(Terrain::Lake) => false,
            MapCell::Terrain(Terrain::Swamp) => false,
            MapCell::Terrain(Terrain::Forest(..)) => false,
            _ => true
        }
    }

    fn get_next_action_when_scan_was_previous(&self, context: &Context) -> Action {
        match context.scanned_data() {
            None => Action::Scan(ScanType::Omni),
            Some(scan_result) if self.are_other_players_in_scan_result(context.player_details().id, scan_result) => {
                self.handle_scan_result_with_other_players(scan_result, context).unwrap_or(Action::Idle)
            }
            Some(scan_result) => self.handle_scan_result_without_other_players(scan_result, context),
        }
    }

    fn get_next_action(&mut self, context: &Context) -> Action {
        let next_action: Action = match self.previous_action {
            Action::Scan(..) => self.get_next_action_when_scan_was_previous(context),
            _ => Action::Scan(ScanType::Omni),
        };
        self.previous_action = next_action.clone();
        next_action
    }
}

#[cfg(test)]
mod tests {

    use crate::avatar;

    use super::*;

    #[test]
    fn check_initial_player_values() {
        let s = Siimesjarvi::new();
        assert_eq!(false, s.is_ready());
    }

    #[test]
    fn basic_strategy_provides_next_orientation_clockwise() {
        let mut s: BasicStrategy = BasicStrategy {
            orientation: Orientation::North,
        };
        assert_eq!(Orientation::North, s.get_next_orientation());
        assert_eq!(Orientation::NorthEast, s.get_next_orientation());
        assert_eq!(Orientation::East, s.get_next_orientation());
        assert_eq!(Orientation::SouthEast, s.get_next_orientation());
        assert_eq!(Orientation::South, s.get_next_orientation());
        assert_eq!(Orientation::SouthWest, s.get_next_orientation());
        assert_eq!(Orientation::West, s.get_next_orientation());
        assert_eq!(Orientation::NorthWest, s.get_next_orientation());
        assert_eq!(Orientation::North, s.get_next_orientation());
    }

    #[test]
    fn advanced_strategy_check_players_in_scan_result() {
        let s: AdvancedStrategy = AdvancedStrategy::new();

        let mut scan_result: ScanResult = ScanResult {
            scan_type: ScanType::Omni,
            data: Box::new(
                [[MapCell::Terrain(Terrain::Field); SCANNING_DISTANCE]; SCANNING_DISTANCE],
            ),
        };
        let my_player_details = PlayerDetails::new(avatar(1), 1);
        // Should be false when completely empty
        assert_eq!(
            false,
            s.are_other_players_in_scan_result(my_player_details.id, &scan_result)
        );

        // Should be false when scan result has a player but the player is myself
        scan_result.data[1][1] = MapCell::Player(my_player_details, Terrain::Field);
        assert_eq!(
            false,
            s.are_other_players_in_scan_result(my_player_details.id, &scan_result)
        );

        // Should be false when scan result has dead player
        let dead_player = PlayerDetails {avatar: avatar(1), alive: false, id: 2, orientation: Orientation::North};
        scan_result.data[1][2] = MapCell::Player(dead_player, Terrain::Field);
        assert_eq!(
            false,
            s.are_other_players_in_scan_result(my_player_details.id, &scan_result)
        );

        // Should be true if there is alive player other than myself
        scan_result.data[1][3] = MapCell::Player(PlayerDetails::new(avatar(1), 2), Terrain::Field);
        assert_eq!(
            true,
            s.are_other_players_in_scan_result(my_player_details.id, &scan_result)
        );
    }
}
