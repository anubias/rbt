use crate::api::orientation::Orientation;
use crate::api::position::Position;

impl Position {
    pub(super) fn get_orientation_to_pos(&self, target: &Position) -> Orientation {
        match target.manhattan_distance(self) {
            (-1, -1) => Orientation::NorthWest,
            (0, -1) => Orientation::North,
            (-1, 0) => Orientation::West,
            (1, 0) => Orientation::East,
            (0, 1) => Orientation::South,
            (1, -1) => Orientation::NorthEast,
            (-1, 1) => Orientation::SouthWest,
            (1, 1) => Orientation::SouthEast,
            _ => panic!("?"),
        }
    }
}
