use crate::api::world_size::MAX_WORLD_SIZE;

#[derive(Clone, Debug, PartialEq)]
pub enum SensorData {
    NotScanned,
    Empty,
    Blocked,
}

#[derive(Clone)]
pub struct Track {
    pub x: usize,
    pub y: usize,
    pub timestamp: i32,
}

#[derive(Clone)]
pub struct Map {
    pub sensor_data: Vec<Vec<SensorData>>,
    pub tracks: Vec<Track>,
    pub initial_scan_done: bool,
    pub actions_since_last_scan_northwest: i64,
    pub actions_since_last_scan_northeast: i64,
    pub actions_since_last_scan_southeast: i64,
    pub actions_since_last_scan_southwest: i64,
}

#[derive(Clone)]
pub struct Data {
    pub map: Map,
}

impl Data {
    pub fn new() -> Self {
        let mut v = Vec::new();
        // initialize
        for _ in 0..MAX_WORLD_SIZE {
            let mut line = Vec::new();
            for _ in 0..MAX_WORLD_SIZE {
                line.push(SensorData::NotScanned);
            }
            v.push(line);
        }

        Self {
            map: Map::new(),
        }
    }
}
