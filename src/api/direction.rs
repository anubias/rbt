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
