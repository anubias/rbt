use super::player::*;

pub struct Siimesjarvi {
    strategy: Strategy,
}

impl Siimesjarvi {
    pub fn new() -> Self {
        Self {
            strategy: Strategy::Advanced(AdvancedStrategy::new())
        }
    }
}

impl Player for Siimesjarvi {
    fn act(&mut self, context: Context) -> Action {
        match &mut self.strategy {
            Strategy::Basic(x) => { x.get_next_action(context) },
            Strategy::Advanced(x) => { 
                x.get_next_action(&context) 
            }
        }
    }

    fn name(&self) -> String {
        "Joni Siimesjärvi 🐙".to_string()
    }
}

#[allow(dead_code)]
enum Strategy {
    Basic(BasicStrategy),
    Advanced(AdvancedStrategy)
}

/// Basic strategy to get started
/// 
/// Blindly fire in all directions in clockwise pattern
#[derive(Default)]
struct BasicStrategy {
    orientation: Orientation
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
    world_map: [MapCell; MAX_WORLD_SIZE*MAX_WORLD_SIZE]
}

struct PlayerInMap {
    player_id: PlayerId,
    x: usize,
    y: usize
}

#[allow(dead_code)]
impl AdvancedStrategy {
    fn new() -> Self {
        Self {
            previous_action: Action::Idle,
            world_map: [MapCell::Unknown; MAX_WORLD_SIZE*MAX_WORLD_SIZE]
        }
    }

    /// Check if any players are in scan result excluding myself
    fn other_players_are_in_scan_result(&self, my_player_id: &PlayerId, scan_result: &ScanResult) -> bool {
        for row in scan_result.data.iter() {
            for map_cell in row.iter() {
                match map_cell {
                    MapCell::Player(player_id, ..) => { 
                        if player_id != my_player_id { return true }
                     }
                    _ => ()
                }
            }
        }
        false
    }

    fn get_players_from_scan_result(&self, scan_result: &ScanResult) -> Vec<PlayerInMap> {
        let mut players: Vec<PlayerInMap> = Vec::new();
        for y in 0..SCANNING_DISTANCE {
            for x in 0..SCANNING_DISTANCE {
                match scan_result.data[x][y] {
                    MapCell::Player(player_id, ..) => { 
                        players.push(PlayerInMap { player_id: player_id.clone(), x: x, y: y })
                     }
                    _ => ()
                }
            }
        }
        players
    }

    fn get_my_coordinates_from_submap(&self, my_player_id: &PlayerId, players: &Vec<PlayerInMap>) -> Option<(isize, isize)> {
        for player in players.iter() {
            if &player.player_id == my_player_id {
                return Option::Some((player.x as isize, player.y as isize))
            }
        }
        Option::None
    }

    fn get_any_other_player_coordinates_from_submap(&self, my_player_id: &PlayerId, players: &Vec<PlayerInMap>) -> Option<(isize, isize)> {
        for player in players.iter() {
            if &player.player_id != my_player_id {
                return Option::Some((player.x as isize, player.y as isize))
            }
        }
        Option::None
    }

    fn calculate_relative_position(&self, my_position:(isize, isize), other_position: (isize, isize)) -> (isize, isize) {
        (other_position.0 - my_position.0, other_position.1 - my_position.1)
    }

    fn calculate_firing_position(&self, my_position: &Position, delta: (isize, isize)) -> Position {
        let new_x = (my_position.x as isize + delta.0) as usize;
        let new_y = (my_position.y as isize + delta.1) as usize;
        Position {x: new_x, y: new_y}
    }


    fn get_firing_position(&self) -> Position {
        // if I have failed to find anything then just shoot my own tank
        Position {x: 0 , y: 0}
    }

    fn get_next_action_when_scan_was_previous(&self, context: &Context) -> Action {
        match context.scanned_data() {
            None => (),
            Some(scan_result) => {
                if let true = self.other_players_are_in_scan_result(context.player_id(), scan_result) { 
                    let players_in_scan_result = self.get_players_from_scan_result(scan_result);

                    // Idle action should not be executed as at this point we should always have other player + our own player in result
                    let my_position = match self.get_my_coordinates_from_submap(context.player_id(), &players_in_scan_result) {
                        Some(x) => x,
                        None => return Action::Idle
                    };
                    let other_player_position = match self.get_any_other_player_coordinates_from_submap(context.player_id(), &players_in_scan_result) {
                        Some(x) => x,
                        None => return Action::Idle
                    };

                    let delta = self.calculate_relative_position(my_position, other_player_position);
                    let pos = self.calculate_firing_position(context.position(), delta);

                    return Action::Fire(Aiming::Positional(pos))
                }
            }
        };
        Action::Idle
    }

    fn get_next_action(&mut self, context: &Context) -> Action {
        let next_action: Action = match self.previous_action {
            Action::Idle => {
                Action::Scan(ScanType::Omni)
            },
            Action::Scan(..) => {
                self.get_next_action_when_scan_was_previous(context)
            },
            Action::Fire(..) => {
                Action::Scan(ScanType::Omni)
            },
            _ => {Action::Idle}
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
        let mut s: BasicStrategy = BasicStrategy { orientation: Orientation::North };
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
    fn advanced_strategy_check_if_players_are_in_scan_result() {
        let s: AdvancedStrategy = AdvancedStrategy::new();

        // Scan result does not contain any players
        let mut scan_result: ScanResult = ScanResult {
            scan_type: ScanType::Omni,
            data: Box::new([[MapCell::Terrain(Terrain::Field); SCANNING_DISTANCE]; SCANNING_DISTANCE])
        };
        let my_player_id = PlayerId::new(avatar(1), 1);
        assert_eq!(false, s.other_players_are_in_scan_result(&my_player_id, &scan_result));

        // Scan result has a player but the player is myself
        scan_result.data[1][1] = MapCell::Player(
            PlayerId::new(avatar(1), 1),
            Terrain::Field
        );
        assert_eq!(false, s.other_players_are_in_scan_result(&my_player_id, &scan_result));

        // Scan result has myself and other player
        scan_result.data[1][2] = MapCell::Player(
            PlayerId::new(avatar(1), 2),
            Terrain::Field
        );
        assert_eq!(true, s.other_players_are_in_scan_result(&my_player_id, &scan_result));
    }

}
