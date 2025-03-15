use super::{orientation::Orientation, world_size::WorldSize};

/// Specifies the size of the scanning data array. It should always be an odd number.
pub const SCANNING_DISTANCE: usize = 15;

/// Specifies the maximum range of a cardinal attack
pub const CARDINAL_SHOT_DISTANCE: usize = SCANNING_DISTANCE - 1;

/// Specifies the maximum range of a positional attack
pub const POSITIONAL_SHOT_DISTANCE: usize = CARDINAL_SHOT_DISTANCE / 2;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    /// Follows a "path" from the current Position according to the provided orientation.
    ///
    /// It takes into account the provided world size, guaranteeing that the result will
    /// either be a valid Position within the bounds of the provided world size, or None.
    pub fn follow(&self, orientation: &Orientation, world_size: &WorldSize) -> Option<Self> {
        let (mut x, mut y) = (self.x as isize, self.y as isize);

        match orientation {
            Orientation::North | Orientation::NorthWest | Orientation::NorthEast => y -= 1,
            Orientation::South | Orientation::SouthWest | Orientation::SouthEast => y += 1,
            _ => {}
        }
        match orientation {
            Orientation::East | Orientation::NorthEast | Orientation::SouthEast => x += 1,
            Orientation::West | Orientation::NorthWest | Orientation::SouthWest => x -= 1,
            _ => {}
        }

        if x >= 0 && x < world_size.x as isize && y >= 0 && y < world_size.y as isize {
            Some(Self {
                x: x as usize,
                y: y as usize,
            })
        } else {
            None
        }
    }

    /// Computes the manhattan distance between `self` and the provided `Position`
    ///
    /// The `manhattan distance` is defined as the distance between two points
    /// that it would take to navigate if one could only travel along the
    /// x and y axis (ie. not diagonally). Similar to computing the walking
    /// distance between two points in Manhattan.
    ///
    /// This distance is expressed separately on the X and Y axis. Please note
    /// that the distances are relative, to allow relative positioning.
    pub fn manhattan_distance(&self, other: &Position) -> (isize, isize) {
        let dx = self.x as isize - other.x as isize;
        let dy = self.y as isize - other.y as isize;

        (dx, dy)
    }

    /// Computes the pythagorean (geometric) distance between `self` and the provided `Position`
    pub fn pythagorean_distance(&self, other: &Position) -> f32 {
        let (dx, dy) = self.manhattan_distance(other);
        let pythagora = dx * dx + dy * dy;

        f32::sqrt(pythagora.abs() as f32)
    }

    /// Indicates whether cardinal shoothing from this position towards the
    /// `other` position, would be successful
    pub fn could_hit_cardinally(&self, other: &Position) -> bool {
        let (dx, dy) = self.manhattan_distance(other);

        (dx.abs() + dy.abs() > 0)
            && (dx * dy == 0 || dx.abs() == dy.abs())
            && self.within_distance(other, CARDINAL_SHOT_DISTANCE)
    }

    /// Indicates whether positional shoothing from this position towards the
    /// `other` position, would be successful
    pub fn could_hit_positionally(&self, other: &Position) -> bool {
        let (dx, dy) = self.manhattan_distance(other);

        (dx.abs() + dy.abs() > 0) && self.within_distance(other, POSITIONAL_SHOT_DISTANCE)
    }

    /// Indicates whether the `other` position is within a certain range or not
    pub fn within_distance(&self, other: &Position, range: usize) -> bool {
        let (dx, dy) = self.manhattan_distance(other);

        dx.unsigned_abs() <= range && dy.unsigned_abs() <= range
    }

    /// Provides a list with all the valid adjacent position of self
    pub fn list_adjacent_positions(&self, world_size: &WorldSize) -> Vec<Position> {
        let mut adjacents = Vec::new();

        let mut orientation = Orientation::North;
        loop {
            if let Some(adjacent_position) = self.follow(&orientation, world_size) {
                adjacents.push(adjacent_position);
            }
            orientation = orientation.rotated_clockwise();
            if orientation == Orientation::North {
                break;
            }
        }

        adjacents
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[x:{:02}, y:{:02}]", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let a = Position { x: 2, y: 3 };
        let b = Position { x: 5, y: 7 };

        let manhattan = a.manhattan_distance(&b);

        assert_eq!((-3, -4), manhattan);
    }
}
