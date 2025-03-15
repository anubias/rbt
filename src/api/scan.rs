use super::{map_cell::MapCell, orientation::Orientation, position::SCANNING_DISTANCE};

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
