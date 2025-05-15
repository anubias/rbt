use super::tanks::*;
use super::terrain::*;

pub struct WorldModel {
    pub map: MappedTerrain,
    pub tanks: TanksTracker,
}

impl WorldModel {
    pub fn new(map: MappedTerrain, tanks: TanksTracker) -> Self {
        Self { map, tanks }
    }
}
