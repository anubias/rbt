use crate::{actor::ActorContext, world::WorldSize};

pub trait Player {
    /// This is the player's turn to fight
    fn act(&self, context: &ActorContext) -> Action;

    /// Returns the player's name
    fn name(&self) -> String {
        "Unnamed".to_string()
    }

    /// This indicates whether the player is ready to battle
    fn is_ready(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn follow(&self, orientation: &Orientation, world_size: &WorldSize) -> Option<Self> {
        let (mut x, mut y) = (self.x as isize, self.y as isize);

        match orientation {
            Orientation::North | Orientation::NorthWest | Orientation::NorthEast => x = x - 1,
            Orientation::South | Orientation::SouthWest | Orientation::SouthEast => x = x + 1,
            _ => {}
        }
        match orientation {
            Orientation::East | Orientation::NorthEast | Orientation::SouthEast => y = y + 1,
            Orientation::West | Orientation::NorthWest | Orientation::SouthWest => y = y - 1,
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
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[x={}, y={}]", self.x, self.y)
    }
}

pub enum Action {
    Fire,
    _Move(Direction),
    _Rotate(Rotation),
    _Scan(_ScanType),
}

pub enum Direction {
    _Forward,
    _Backward,
}

#[derive(Clone)]
pub enum Orientation {
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

    pub fn rotate_clockwise(&self) -> Self {
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

    pub fn rotate_counter_clockwise(&self) -> Self {
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
            7 => Self::NorthWest,
            _ => Self::North,
        }
    }
}

pub enum Rotation {
    _Clockwise,
    _CounterClockwise,
}

pub enum _ScanType {
    _Omni,
    _Directional,
}
