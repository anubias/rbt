use super::player::*;

#[derive(Clone)]
enum SensorData {
    NotScanned,
    Empty,
    Blocked,
}

pub struct Rahtu {
    sensor_data: Vec<Vec<SensorData>>,
}

impl Rahtu {
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

        Self { sensor_data: v }
    }
    fn update_map(&mut self, context: &Context) {
        if let Some(scanned_data) = context.scanned_data() {
            match scanned_data.scan_type {
                ScanType::Directional(_) => todo!(),
                ScanType::Omni => {
                    for x in 0..SCANNING_DISTANCE {
                        if x < SCANNING_DISTANCE / 2 || x + SCANNING_DISTANCE / 2 > MAX_WORLD_SIZE {
                            continue;
                        }
                        let global_x = x - SCANNING_DISTANCE / 2;
                        for y in 0..SCANNING_DISTANCE {
                            if y < SCANNING_DISTANCE / 2
                                || y + SCANNING_DISTANCE / 2 > MAX_WORLD_SIZE
                            {
                                continue;
                            }
                            let global_y = y - SCANNING_DISTANCE / 2;
                            if global_x == context.position().x && global_y == context.position().y
                            {
                                continue;
                            }
                            match scanned_data.data[x][y] {
                                MapCell::Terrain(Terrain::Field) => {
                                    self.sensor_data[global_x][global_y] = SensorData::Empty
                                }
                                MapCell::Player(_, _) => {
                                    self.sensor_data[global_x][global_y] = SensorData::NotScanned
                                }

                                _ => self.sensor_data[global_x][global_y] = SensorData::Blocked,
                            }
                        }
                    }
                }
            }
        }
    }
    fn get_map_tile(&self, x: isize, y: isize) -> SensorData {
        if x < 0 || x > MAX_WORLD_SIZE as isize {
            return SensorData::Blocked;
        }
        if y < 0 || y > MAX_WORLD_SIZE as isize {
            return SensorData::Blocked;
        }
        return self.sensor_data[x as usize][y as usize].clone();
    }
    fn get_map_tile_in_front(&mut self, context: &Context) -> SensorData {
        match context.orientation() {
            Orientation::North => self.get_map_tile(
                context.position().x as isize,
                context.position().y as isize - 1,
            ),
            Orientation::East => self.get_map_tile(
                context.position().x as isize,
                context.position().y as isize - 1,
            ),
            Orientation::South => self.get_map_tile(
                context.position().x as isize,
                context.position().y as isize - 1,
            ),
            Orientation::West => self.get_map_tile(
                context.position().x as isize,
                context.position().y as isize - 1,
            ),
            _ => SensorData::Blocked,
        }
    }

    fn get_next_action(&mut self, context: &Context) -> Action {
        return match context.orientation() {
            Orientation::North => match self.get_map_tile_in_front(context) {
                SensorData::NotScanned => Action::Scan(ScanType::Omni),
                SensorData::Empty => Action::Move(Direction::Forward),
                SensorData::Blocked => Action::Rotate(Rotation::CounterClockwise),
            },
            Orientation::NorthEast => Action::Rotate(Rotation::CounterClockwise),
            Orientation::East => match self.get_map_tile_in_front(context) {
                SensorData::NotScanned => Action::Scan(ScanType::Omni),
                SensorData::Empty => Action::Move(Direction::Forward),
                SensorData::Blocked => Action::Rotate(Rotation::CounterClockwise),
            },
            Orientation::SouthEast => Action::Rotate(Rotation::CounterClockwise),
            Orientation::South => match self.get_map_tile_in_front(context) {
                SensorData::NotScanned => Action::Scan(ScanType::Omni),
                SensorData::Empty => Action::Move(Direction::Forward),
                SensorData::Blocked => Action::Rotate(Rotation::CounterClockwise),
            },
            Orientation::SouthWest => Action::Rotate(Rotation::CounterClockwise),
            Orientation::West => match self.get_map_tile_in_front(context) {
                SensorData::NotScanned => Action::Scan(ScanType::Omni),
                SensorData::Empty => Action::Move(Direction::Forward),
                SensorData::Blocked => Action::Rotate(Rotation::CounterClockwise),
            },
            Orientation::NorthWest => Action::Rotate(Rotation::CounterClockwise),
        };
    }
}

impl Player for Rahtu {
    fn act(&mut self, context: &Context) -> Action {
        self.update_map(context);
        // println(context);
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Rahtu".to_string()
    }
}
