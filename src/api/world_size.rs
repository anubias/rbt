/// Specifies the maximum horizontal or vertical size of the game map
pub const MAX_WORLD_SIZE: usize = 128;

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
