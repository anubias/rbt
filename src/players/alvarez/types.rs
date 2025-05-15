// silence unused imports
#![allow(unused_imports)]

// Just api and common aliases
pub use crate::{
    api::{
        action::Action,
        aiming::Aiming,
        context::Context,
        direction::Direction,
        map_cell::{MapCell, Terrain, TreeType},
        orientation::Orientation,
        path_finder::{MapReader, PathFinder},
        player::{PlayerId, Player, Details},
        position::Position,
        position::SCANNING_DISTANCE,
        rotation::Rotation,
        scan::ScanResult,
        scan::ScanType,
        world_size::WorldSize,
        world_size::MAX_WORLD_SIZE,
    },
    terminal::Terminal,
};

pub type Distance = f32;
pub type PId = PlayerId;
pub type Turn = usize;


impl Orientation {
    // Existing methods...

    /// Returns all 8 orientations in clockwise order
    pub fn all() -> [Orientation; 8] {
        [
            Orientation::North,
            Orientation::NorthEast,
            Orientation::East,
            Orientation::SouthEast,
            Orientation::South,
            Orientation::SouthWest,
            Orientation::West,
            Orientation::NorthWest,
        ]
    }

    /// Returns the 4 cardinal orientations
    pub fn cardinals() -> [Orientation; 4] {
        [
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ]
    }

    /// Returns the 4 diagonal orientations
    pub fn diagonals() -> [Orientation; 4] {
        [
            Orientation::NorthEast,
            Orientation::SouthEast,
            Orientation::SouthWest,
            Orientation::NorthWest,
        ]
    }
}
