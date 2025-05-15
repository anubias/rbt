use crate::api::{
    map_cell::MapCell,
    orientation::Orientation,
    player::{Details, PlayerId, INVALID_PLAYER},
    position::{Position, SCANNING_DISTANCE},
    rotation::Rotation,
    scan::{ScanResult, ScanType},
};

impl Position {
    pub(super) fn get_orientation_to(&self, neighbor: &Position) -> Orientation {
        match neighbor.manhattan_distance(self) {
            (1, 0) => Orientation::East,
            (1, 1) => Orientation::SouthEast,
            (0, 1) => Orientation::South,
            (-1, 1) => Orientation::SouthWest,
            (-1, 0) => Orientation::West,
            (-1, -1) => Orientation::NorthWest,
            (0, -1) => Orientation::North,
            (1, -1) => Orientation::NorthEast,
            _ => panic!("Not a neighbor position {self} => {neighbor}"),
        }
    }
}

impl Orientation {
    pub(super) fn quick_turn_bidirectional(&self, other: &Self) -> (Rotation, usize) {
        let result = self.quick_turn(other);
        if result.1 <= 2 {
            result
        } else {
            (result.0.opposite(), 4 - result.1)
        }
    }
}

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

    fn get_player_details(map_cell: &MapCell) -> Details {
        match map_cell {
            MapCell::Explosion(player_details, _) => player_details.clone(),
            MapCell::Player(player_details, _) => player_details.clone(),
            MapCell::Shell(player_details, _) => player_details.clone(),
            _ => INVALID_PLAYER,
        }
    }

    pub(super) fn find_other_players(
        &self,
        my_id: PlayerId,
        my_world_position: &Position,
    ) -> Vec<(Details, Position)> {
        let mut other_players = Vec::new();
        let scan_world_position = self.get_world_position(my_world_position);

        for y in 0..self.data.len() {
            for x in 0..self.data[y].len() {
                let player_details = ScanResult::get_player_details(&self.data[y][x]);

                if player_details != INVALID_PLAYER
                    && player_details.id != my_id
                    && player_details.alive
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

    pub(super) fn get_world_position(&self, my_world_position: &Position) -> (isize, isize) {
        let my_scan_position = self.get_my_position();
        let scan_world_x = my_world_position.x as isize - my_scan_position.x as isize;
        let scan_world_y = my_world_position.y as isize - my_scan_position.y as isize;

        return (scan_world_x, scan_world_y);
    }
}
