use super::{aiming::Aiming, direction::Direction, rotation::Rotation, scan::ScanType};

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
