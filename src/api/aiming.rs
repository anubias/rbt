use super::{orientation::Orientation, position::Position};

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
