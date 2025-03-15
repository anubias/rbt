use super::rotation::Rotation;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Orientation {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Orientation {
    pub fn get_cardinal_direction_count() -> usize {
        8
    }

    pub fn rotated_clockwise(&self) -> Self {
        match self {
            Self::North => Self::NorthEast,
            Self::NorthEast => Self::East,
            Self::East => Self::SouthEast,
            Self::SouthEast => Self::South,
            Self::South => Self::SouthWest,
            Self::SouthWest => Self::West,
            Self::West => Self::NorthWest,
            Self::NorthWest => Self::North,
        }
    }

    pub fn rotated_counter_clockwise(&self) -> Self {
        match self {
            Self::North => Self::NorthWest,
            Self::NorthWest => Self::West,
            Self::West => Self::SouthWest,
            Self::SouthWest => Self::South,
            Self::South => Self::SouthEast,
            Self::SouthEast => Self::East,
            Self::East => Self::NorthEast,
            Self::NorthEast => Self::North,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::East => Self::West,
            Self::SouthEast => Self::NorthWest,
            Self::South => Self::North,
            Self::SouthWest => Self::NorthEast,
            Self::West => Self::East,
            Self::NorthWest => Self::SouthEast,
        }
    }

    /// Computes the fastest way to turn towards a new `Orientation`.
    ///
    /// The result is a tuple containing the Rotation and the number of
    /// steps to repeat the same rotation in order to change orientation
    /// to the desired one.
    pub fn quick_turn(&self, other: &Self) -> (Rotation, usize) {
        let mut my_index: usize = self.into();
        let their_index: usize = other.into();

        if my_index < their_index {
            my_index += Orientation::get_cardinal_direction_count();
        }

        let mut delta = my_index - their_index;
        let rotation = match delta {
            ..4 => Rotation::CounterClockwise,
            4.. => Rotation::Clockwise,
        };
        if delta > 4 {
            delta = Orientation::get_cardinal_direction_count() - delta;
        }

        (rotation, delta)
    }
}

impl From<usize> for Orientation {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::North,
            1 => Self::NorthEast,
            2 => Self::East,
            3 => Self::SouthEast,
            4 => Self::South,
            5 => Self::SouthWest,
            6 => Self::West,
            7.. => Self::NorthWest,
        }
    }
}

impl From<&Orientation> for usize {
    fn from(value: &Orientation) -> Self {
        match value {
            Orientation::North => 0,
            Orientation::NorthEast => 1,
            Orientation::East => 2,
            Orientation::SouthEast => 3,
            Orientation::South => 4,
            Orientation::SouthWest => 5,
            Orientation::West => 6,
            Orientation::NorthWest => 7,
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::North => " N ",
            Self::NorthEast => "N-E",
            Self::East => " E ",
            Self::SouthEast => "S-E",
            Self::South => " S ",
            Self::SouthWest => "S-W",
            Self::West => " W ",
            Self::NorthWest => "N-W",
        };

        write!(f, "{text}")
    }
}
