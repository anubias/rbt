#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rotation {
    #[default]
    Clockwise,
    CounterClockwise,
}

impl Rotation {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::api::orientation::Orientation;

    use super::*;

    #[test]
    fn test_orientation_quick_turn() {
        let a = Orientation::North;
        let b = Orientation::SouthWest;

        let steps = a.quick_turn(&b);
        assert_eq!((Rotation::CounterClockwise, 3), (steps.0, steps.1));

        let a = Orientation::North;
        let b = Orientation::SouthEast;

        let steps = a.quick_turn(&b);
        assert_eq!((Rotation::Clockwise, 3), (steps.0, steps.1));

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
