/// Specifies the maximum horizontal or vertical size of the game map
pub const MAX_WORLD_SIZE: usize = 64;

/// Specifies the size of the scanning data array. It should always be an odd number.
pub const SCANNING_DISTANCE: usize = (MAX_WORLD_SIZE / DIV) - (MAX_WORLD_SIZE / DIV + 1) % 2;
const DIV: usize = 4;

/// Public trait that players need to implement, in order for the game engine to be able to interact with the player
pub trait Player {
    /// This is the player's turn to fight
    fn act(&mut self, context: &Context) -> Action;

    /// Returns the player's name
    fn name(&self) -> String;

    /// This indicates whether the player is ready to battle
    fn is_ready(&self) -> bool {
        false
    }
}

/// Represents the context that the game engine is sharing with the player logic with
/// every interaction.
pub struct Context {
    player_id: u8,
    health: u8,
    mobile: bool,
    position: Position,
    orientation: Orientation,
    scan: Option<ScanResult>,
    world_size: WorldSize,
}

impl Context {
    pub fn new(player_id: u8, position: Position, world_size: WorldSize) -> Self {
        Self {
            player_id,
            health: 100,
            mobile: true,
            position,
            orientation: Orientation::default(),
            scan: None,
            world_size,
        }
    }

    pub fn player_id(&self) -> u8 {
        self.player_id
    }

    pub fn health(&self) -> u8 {
        self.health
    }

    pub fn is_mobile(&self) -> bool {
        self.mobile
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn orientation(&self) -> &Orientation {
        &self.orientation
    }

    pub fn world_size(&self) -> &WorldSize {
        &self.world_size
    }

    pub fn rotate(&mut self, rotation: &Rotation) {
        self.orientation = match rotation {
            Rotation::Clockwise => self.orientation.rotate_clockwise(),
            Rotation::CounterClockwise => self.orientation.rotate_counter_clockwise(),
        }
    }

    pub fn damage(&mut self, damage: u8) {
        self.health -= self.health.min(damage);
    }

    pub fn relocate(&mut self, new_position: &Position, walk_on: Terrain) -> bool {
        self.position = new_position.clone();

        match walk_on {
            Terrain::Lake => self.health = 0,
            Terrain::Swamp => self.mobile = false,
            _ => {}
        }

        true
    }

    pub fn scanned_data(&self) -> &Option<ScanResult> {
        &self.scan
    }

    pub fn set_scanned_data(&mut self, scan: Option<ScanResult>) {
        self.scan = scan;
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scan) = &self.scan {
            write!(
                f,
                "{{health={}, position={}, orientation=\"{}\", scanned_data={}}}",
                self.health, self.position, self.orientation, scan
            )
        } else {
            write!(
                f,
                "{{health={}, position={}, orientation=\"{}\"}}",
                self.health, self.position, self.orientation,
            )
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum MapCell {
    Player(u8, Terrain),
    Terrain(Terrain),
    #[default]
    Unknown,
}

impl std::fmt::Display for MapCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Player(_, _) => write!(f, "🪖"), // 🪖
            Self::Terrain(t) => write!(f, "{t}"),
            Self::Unknown => write!(f, "⬛"), // "", ⬛
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
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
            Self::Swamp => write!(f, "⬜"),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum TreeType {
    #[default]
    Deciduous,
    Evergreen,
}

#[derive(Default)]
pub enum Action {
    #[default]
    Idle,
    Fire,
    Move(Direction),
    Rotate(Rotation),
    Scan(ScanType),
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

    pub fn manhattan_distance(&self, other: &Position) -> (isize, isize) {
        let dx = self.x as isize - other.x as isize;
        let dy = self.y as isize - other.y as isize;

        (dx, dy)
    }

    pub fn pythagorean_distance(&self, other: &Position) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;

        (dx * dx + dy * dy).sqrt()
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[x={}, y={}]", self.x, self.y)
    }
}

pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Default, PartialEq, Eq, Hash)]
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

    pub fn steps_to(&self, other: &Self) -> (Rotation, usize) {
        let my_index: isize = self.into();
        let their_index: isize = other.into();

        let delta = my_index - their_index;
        let rotation = match delta {
            ..-4 => Rotation::CounterClockwise,
            -4..=0 => Rotation::Clockwise,
            1..=4 => Rotation::CounterClockwise,
            5.. => Rotation::Clockwise,
        };

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

impl From<&Orientation> for isize {
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

pub enum Rotation {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone)]
pub enum ScanType {
    Directional(Orientation),
    Omni,
}

impl std::fmt::Display for ScanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Directional(o) => write!(f, "directional({o})"),
            Self::Omni => write!(f, "omni"),
        }
    }
}

#[derive(Clone)]
pub struct ScanResult {
    pub scan_type: ScanType,
    pub data: Box<[[MapCell; SCANNING_DISTANCE]; SCANNING_DISTANCE]>,
}

impl std::fmt::Display for ScanResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{\nscan_type:{},\ndata:", self.scan_type)?;
        for i in 0..SCANNING_DISTANCE {
            write!(f, "\n")?;
            for j in 0..SCANNING_DISTANCE {
                write!(f, "{}", self.data[i][j])?;
            }
        }
        write!(f, "\n}}")
    }
}

#[derive(Clone)]
pub struct WorldSize {
    pub x: usize,
    pub y: usize,
}

impl std::fmt::Display for WorldSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
