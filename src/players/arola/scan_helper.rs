use super::super::super::DEAD_AVATAR;
use super::super::player::*;

const SCAN_TOP: usize = 0;
const SCAN_LEFT: usize = 0;
const SCAN_CENTER: usize = SCANNING_DISTANCE / 2;
const SCAN_BOTTOM: usize = SCANNING_DISTANCE - 1;
const SCAN_RIGHT: usize = SCANNING_DISTANCE - 1;

impl ScanResult {
    fn get_my_position(&self) -> Position {
        match &self.scan_type {
            ScanType::Mono(orientation) => match orientation {
                Orientation::North => Position {
                    x: SCAN_CENTER,
                    y: SCAN_BOTTOM,
                },
                Orientation::NorthEast => Position {
                    x: SCAN_LEFT,
                    y: SCAN_BOTTOM,
                },
                Orientation::East => Position {
                    x: SCAN_LEFT,
                    y: SCAN_CENTER,
                },
                Orientation::SouthEast => Position {
                    x: SCAN_LEFT,
                    y: SCAN_TOP,
                },
                Orientation::South => Position {
                    x: SCAN_CENTER,
                    y: SCAN_TOP,
                },
                Orientation::SouthWest => Position {
                    x: SCAN_RIGHT,
                    y: SCAN_TOP,
                },
                Orientation::West => Position {
                    x: SCAN_RIGHT,
                    y: SCAN_CENTER,
                },
                Orientation::NorthWest => Position {
                    x: SCAN_RIGHT,
                    y: SCAN_BOTTOM,
                },
            },
            ScanType::Omni => Position {
                y: SCAN_CENTER,
                x: SCAN_CENTER,
            },
        }
    }

    fn get_player_details(map_cell: &MapCell) -> PlayerDetails {
        match map_cell {
            MapCell::Explosion(player_details, _) => player_details.clone(),
            MapCell::Player(player_details, _) => player_details.clone(),
            MapCell::Shell(player_details, _) => player_details.clone(),
            _ => INVALID_PLAYER,
        }
    }

    pub fn find_other_players(
        &self,
        my_id: &PlayerDetails,
        my_world_position: &Position,
    ) -> Vec<(PlayerDetails, Position)> {
        let mut other_players = Vec::new();
        let scan_world_position = self.get_world_position(my_world_position);

        for y in 0..self.data.len() {
            for x in 0..self.data[y].len() {
                let player_details = ScanResult::get_player_details(&self.data[y][x]);

                if player_details != INVALID_PLAYER
                    && &player_details != my_id
                    && player_details.is_alive()
                {
                    let position_x = x as isize + scan_world_position.0;
                    let position_y = y as isize + scan_world_position.1;
                    assert!(position_x >= 0 && position_y >= 0);

                    other_players.push((
                        player_details,
                        Position {
                            x: position_x as usize,
                            y: position_y as usize,
                        },
                    ));
                }
            }
        }

        return other_players;
    }

    pub fn get_world_position(&self, my_world_position: &Position) -> (isize, isize) {
        let my_scan_position = self.get_my_position();
        let scan_world_x = my_world_position.x as isize - my_scan_position.x as isize;
        let scan_world_y = my_world_position.y as isize - my_scan_position.y as isize;

        return (scan_world_x, scan_world_y);
    }
}

impl PlayerDetails {
    pub fn is_alive(&self) -> bool {
        self.avatar != DEAD_AVATAR
    }
}
