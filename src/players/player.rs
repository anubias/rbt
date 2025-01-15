/// Specifies the maximum horizontal or vertical size of the game map
pub const MAX_GAME_MAP_SIZE: usize = 64;

/// Specifies the size of the scanning data array. It should always be an odd number.
pub const SCANNING_DISTANCE: usize = (MAX_GAME_MAP_SIZE / 8) - (MAX_GAME_MAP_SIZE / 8 + 1) % 2;

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

    pub fn immobilize(&mut self) {
        self.mobile = false;
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

    #[must_use]
    pub fn relocate(&mut self, position: Position) -> bool {
        self.position = position;
        true
    }

    pub fn reset_scan(&mut self, scan: Option<ScanResult>) {
        self.scan = scan;
    }

    pub fn scanned_data(&self) -> &Option<ScanResult> {
        &self.scan
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scan) = &self.scan {
            write!(
                f,
                "{{health={}, mobile={}, position={}, orientation=\"{}\", scanned_data={}}}",
                self.health, self.mobile, self.position, self.orientation, scan
            )
        } else {
            write!(
                f,
                "{{health={}, mobile={}, position={}, orientation=\"{}\"}}",
                self.health, self.mobile, self.position, self.orientation,
            )
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MapCell {
    Field,
    Lake,
    Mountain,
    Player(u8),
    Swamp,
    Unknown,
}

impl std::fmt::Display for MapCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field => write!(f, "🌱"),
            Self::Lake => write!(f, "🌊"),
            Self::Mountain => write!(f, "🪨"),
            Self::Player(_) => write!(f, "🪖"),
            Self::Swamp => write!(f, "🌲"),
            Self::Unknown => write!(f, ""),
        }
    }
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

#[derive(Clone, Default)]
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
    pub data: [[MapCell; SCANNING_DISTANCE]; SCANNING_DISTANCE],
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
