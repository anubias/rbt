//! Provides the interfaces necessary to implement a working Rusty Battle Tank
//!
//! Every player needs to create their own data structure(s), of which one must
//! implement the `Player` trait. That trait provides the mechanism necessary
//! for the RBT game engine to interact with the player, and vice-versa.
//!
//! This module also provides a host of useful ancilary data structures, like
//! `Context`, `MapCell`, `Terrain`, `Position`, `Rotation`, `Direction`,
//! `Orientation`, etc. The players should check the (free) functionality
//! offered by these data structures, and avoid re-implementing already
//! provided behavior.

use crate::DEAD_AVATAR;

/// Specifies the maximum horizontal or vertical size of the game map
pub const MAX_WORLD_SIZE: usize = 64;

/// Specifies the size of the scanning data array. It should always be an odd number.
pub const SCANNING_DISTANCE: usize = (MAX_WORLD_SIZE / DIV) - (MAX_WORLD_SIZE / DIV + 1) % 2;
const DIV: usize = 4;

/// Specifies the maximum range of a cardinal attack
pub const CARDINAL_SHOT_DISTANCE: usize = SCANNING_DISTANCE - 1;

/// Specifies the maximum range of a positional attack
pub const POSITIONAL_SHOT_DISTANCE: usize = CARDINAL_SHOT_DISTANCE / 2 - 1;

/// Public trait that players need to implement, in order for the game engine
/// to be able to interact with the player.
pub trait Player {
    /// Implement this method if and only if you need to perform expensive and
    /// potentially failing initialization.
    ///
    /// The return value should indicate the initialization success.
    fn initialized(&mut self) -> bool {
        true
    }

    /// This is the player's turn to fight.
    ///
    /// The changes performed by the game engine are provided in the `context`.
    fn act(&mut self, context: Context) -> Action;

    /// Returns the player's name
    fn name(&self) -> String;

    /// This indicates whether the player is ready for battle or not.
    fn is_ready(&self) -> bool {
        false
    }
}

pub const INVALID_PLAYER_ID: PlayerId = PlayerId { avatar: ' ', id: 0 };

/// Defines the player id type
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerId {
    pub avatar: char,
    pub id: usize,
}

impl PlayerId {
    pub fn new(avatar: char, id: usize) -> Self {
        if id == 0 {
            panic!("Invalid player id=0 used!");
        }
        Self { avatar, id }
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{id: {}, avatar: {}}}", self.id, self.avatar)
    }
}

/// Represents the context that the game engine is sharing with the player logic with
/// every interaction.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    health: u8,
    mobile: bool,
    previous_action: Action,
    orientation: Orientation,
    player_id: PlayerId,
    position: Position,
    scan: Option<ScanResult>,
    world_size: WorldSize,
}

impl Context {
    pub fn new(player_id: PlayerId, position: Position, world_size: WorldSize) -> Self {
        Self {
            health: 100,
            mobile: true,
            previous_action: Action::default(),
            orientation: Orientation::default(),
            player_id,
            position,
            scan: None,
            world_size,
        }
    }

    pub fn damage(&mut self, damage: u8) {
        self.health = self.health.saturating_sub(damage);
        if self.health == 0 {
            self.player_id.avatar = DEAD_AVATAR;
        }
    }

    pub fn health(&self) -> u8 {
        self.health
    }

    pub fn is_mobile(&self) -> bool {
        self.mobile
    }

    pub fn previous_action(&self) -> &Action {
        &self.previous_action
    }

    pub fn orientation(&self) -> &Orientation {
        &self.orientation
    }

    pub fn player_id(&self) -> &PlayerId {
        &self.player_id
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn relocate(&mut self, new_position: &Position, walk_on: Terrain) {
        self.position = new_position.clone();

        match walk_on {
            Terrain::Lake => self.health = 0,
            Terrain::Swamp => self.mobile = false,
            _ => {}
        }
    }

    pub fn rotate(&mut self, rotation: &Rotation) {
        self.orientation = match rotation {
            Rotation::Clockwise => self.orientation.rotated_clockwise(),
            Rotation::CounterClockwise => self.orientation.rotated_counter_clockwise(),
        }
    }

    pub fn scanned_data(&self) -> &Option<ScanResult> {
        &self.scan
    }

    pub fn set_previous_action(&mut self, action: Action) {
        self.previous_action = action
    }

    pub fn set_scanned_data(&mut self, scan: Option<ScanResult>) {
        self.scan = scan;
    }

    pub fn world_size(&self) -> &WorldSize {
        &self.world_size
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if let Some(_) = &self.scan {
            format!(
                "{{\n   player_id: {},\n   health: {},\n   mobile: {},\n   previous_action: \"{}\",   orientation: \"{}\",\n   position: {},\n   scanned_data: present\n}}",
                self.player_id, self.health, self.mobile, self.previous_action, self.orientation, self.position
            )
        } else {
            format!(
                "{{\n   player_id: {},\n   health: {},\n   mobile: {},\n   previous_action: \"{}\",   orientation: \"{}\",\n   position: {},\n   scanned_data: absent\n}}",
                self.player_id, self.health, self.mobile, self.previous_action, self.orientation, self.position,
            )
        };
        write!(f, "{text}")
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapCell {
    Explosion(PlayerId, Terrain),
    Player(PlayerId, Terrain),
    Shell(PlayerId, Terrain),
    Terrain(Terrain),
    #[default]
    Unknown,
}

impl std::fmt::Display for MapCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explosion(_, _) => write!(f, "💥"),
            Self::Player(player_id, _) => write!(f, "{}", player_id.avatar),
            Self::Shell(_, _) => write!(f, "🔴"),
            Self::Terrain(t) => write!(f, "{t}"),
            Self::Unknown => write!(f, "⬛"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Terrain {
    #[default]
    Field,
    Lake,
    Forest(TreeType),
    Swamp,
}

impl std::fmt::Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field => write!(f, "🟩"),
            Self::Lake => write!(f, "🟦"),
            Self::Forest(TreeType::Deciduous) => write!(f, "🌳"),
            Self::Forest(TreeType::Evergreen) => write!(f, "🌲"),
            Self::Swamp => write!(f, "🟫"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeType {
    #[default]
    Deciduous,
    Evergreen,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    #[default]
    Idle,
    Fire(Aiming),
    Move(Direction),
    Rotate(Rotation),
    Scan(ScanType),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Action::Idle => "Idle".to_string(),
            Action::Fire(a) => format!("Fire({a})"),
            Action::Move(d) => format!("Move({d})"),
            Action::Rotate(r) => format!("Rotate({r})"),
            Action::Scan(s) => format!("Scan({s})"),
        };

        write!(f, "{text}")
    }
}

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
            Orientation::North | Orientation::NorthWest | Orientation::NorthEast => y = y - 1,
            Orientation::South | Orientation::SouthWest | Orientation::SouthEast => y = y + 1,
            _ => {}
        }
        match orientation {
            Orientation::East | Orientation::NorthEast | Orientation::SouthEast => x = x + 1,
            Orientation::West | Orientation::NorthWest | Orientation::SouthWest => x = x - 1,
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

    /// Computes the (x,y) delta between self and the provided `Position`
    ///
    /// In other words, this computes the delta between two Positions
    /// on the X and Y axes separately.
    ///
    /// Please note that the distances may be negative.
    pub fn delta(&self, other: &Position) -> (isize, isize) {
        let dx = self.x as isize - other.x as isize;
        let dy = self.y as isize - other.y as isize;

        (dx, dy)
    }

    /// Computes the manhattan distance between self and the provided `Position`
    ///
    /// The `manhattan distance` is defined as the distance between two points
    /// that it would take to navigate if one could only travel along the
    /// x and y axis (ie. not diagonally). Similar to computing the walking
    /// distance between two points in Manhattan.
    pub fn manhattan_distance(&self, other: &Position) -> usize {
        let (dx, dy) = self.delta(other);

        (dx.abs() + dy.abs()) as usize
    }

    /// Computes the pythagorean distance between self and the provided `Position`.
    pub fn pythagorean_distance(&self, other: &Position) -> f32 {
        let (dx, dy) = self.delta(other);

        ((dx * dx + dy * dy) as f32).sqrt()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[x: {}, y: {}]", self.x, self.y)
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    #[default]
    Forward,
    Backward,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Direction::Forward => "Forward",
            Direction::Backward => "Backward",
        };
        write!(f, "{text}")
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
            my_index += Orientation::get_cardinal_direction_count() as usize;
        }

        let mut delta = my_index - their_index;
        let rotation = match delta {
            ..4 => Rotation::CounterClockwise,
            4.. => Rotation::Clockwise,
        };
        if delta > 4 {
            delta = delta % 4;
        }

        (rotation, delta as usize)
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
            Self::North => "north",
            Self::NorthEast => "north-east",
            Self::East => "east",
            Self::SouthEast => "south-east",
            Self::South => "south",
            Self::SouthWest => "south-west",
            Self::West => "west",
            Self::NorthWest => "north-west",
        };

        write!(f, "{text}")
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rotation {
    #[default]
    Clockwise,
    CounterClockwise,
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Rotation::Clockwise => "Clockwise",
            Rotation::CounterClockwise => "Counter-Clockwise",
        };
        write!(f, "{text}")
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Aiming {
    Cardinal(Orientation),
    Positional(Position),
}

impl Default for Aiming {
    fn default() -> Self {
        Aiming::Cardinal(Orientation::default())
    }
}

impl std::fmt::Display for Aiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Aiming::Cardinal(o) => format!("Cardinal({})", o),
            Aiming::Positional(p) => format!("Positional({})", p),
        };
        write!(f, "{}", text)
    }
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanType {
    Mono(Orientation),
    #[default]
    Omni,
}

impl std::fmt::Display for ScanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Mono(o) => format!("Mono({o})"),
            Self::Omni => "Omni".to_string(),
        };
        write!(f, "{text}")
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScanResult {
    pub scan_type: ScanType,
    pub data: Box<[[MapCell; SCANNING_DISTANCE]; SCANNING_DISTANCE]>,
}

impl std::fmt::Display for ScanResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{scan_type: {},   data:", self.scan_type)?;
        for i in 0..SCANNING_DISTANCE {
            write!(f, "\n      ")?;
            for j in 0..SCANNING_DISTANCE {
                write!(f, "{}", self.data[i][j])?;
            }
        }
        write!(f, "\n}}")
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldSize {
    pub x: usize,
    pub y: usize,
}

impl std::fmt::Display for WorldSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(width={}, height={})", self.x, self.y)
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
        let pythagorean = a.pythagorean_distance(&b);

        assert_eq!(3 + 4, manhattan);
        assert_eq!(5.0, pythagorean);
    }

    #[test]
    fn test_orientation_quick_turn() {
        let a = Orientation::North;
        let b = Orientation::SouthWest;

        let steps = a.quick_turn(&b);
        assert_eq!((Rotation::CounterClockwise, 3), (steps.0, steps.1));

        let a = Orientation::West;
        let b = Orientation::North;

        let steps = a.quick_turn(&b);
        assert_eq!((Rotation::Clockwise, 2), (steps.0, steps.1));

        let a = Orientation::North;
        let b = Orientation::South;

        let steps = a.quick_turn(&b);
        assert_eq!((Rotation::Clockwise, 4), (steps.0, steps.1));

        let a = Orientation::West;
        let b = Orientation::West;

        let steps = a.quick_turn(&b);
        assert_eq!((Rotation::CounterClockwise, 0), (steps.0, steps.1));
    }
}
