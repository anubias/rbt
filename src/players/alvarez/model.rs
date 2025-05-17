use super::tanks::*;
use super::terrain::*;

pub struct WorldModel {
    pub map: MappedTerrain,
    pub tanks: TanksTracker,
}
