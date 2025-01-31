use super::super::player::*;

const SCAN_TOP: usize = 0;
const SCAN_LEFT: usize = 0;
const SCAN_CENTER: usize = SCANNING_DISTANCE / 2;
const SCAN_BOTTOM: usize = SCANNING_DISTANCE - 1;
const SCAN_RIGHT: usize = SCANNING_DISTANCE - 1;

impl ScanResult {
    pub fn get_my_position(&self) -> Position {
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

    pub fn find_other_players(&self, my_world_position: &Position) -> Vec<(PlayerId, Position)> {
        let mut players = Vec::new();
        let my_position = self.get_my_position();
        let scan_position = self.get_world_position(my_world_position);

        for y in 0..self.data.len() {
            for x in 0..self.data[y].len() {
                if let MapCell::Player(player_id, _) = self.data[y][x] {
                    if (x, y) != (my_position.x, my_position.y) {
                        let position_x = x as isize + scan_position.0;
                        let position_y = y as isize + scan_position.1;
                        assert!(position_x >= 0 && position_y >= 0);
                        players.push((
                            player_id,
                            Position {
                                x: position_x as usize,
                                y: position_y as usize,
                            },
                        ));
                    }
                }
            }
        }

        return players;
    }

    pub fn get_world_position(&self, player_world_position: &Position) -> (isize, isize) {
        let player_scan_position = self.get_my_position();
        let scan_world_x = player_world_position.x as isize - player_scan_position.x as isize;
        let scan_world_y = player_world_position.y as isize - player_scan_position.y as isize;

        return (scan_world_x, scan_world_y);
    }
}
