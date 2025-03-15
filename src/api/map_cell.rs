use super::player::Details;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapCell {
    Explosion(Details, Terrain),
    Player(Details, Terrain),
    Shell(Details, Terrain),
    Terrain(Terrain),
    #[default]
    Unallocated,
}

impl std::fmt::Display for MapCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explosion(_, _) => write!(f, "💥"),
            Self::Player(player_details, _) => write!(f, "{}", player_details.avatar),
            Self::Shell(_, _) => write!(f, "🔴"),
            Self::Terrain(t) => write!(f, "{t}"),
            Self::Unallocated => write!(f, "⬛"),
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
