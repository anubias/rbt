use super::player::*;

const DEBUG_PRINTS: bool = false;

#[derive(Clone, Debug)]
enum SensorData {
    NotScanned,
    Empty,
    Blocked,
}

struct Track {
    x: usize,
    y: usize,
    timestamp: i32,
}

pub struct Rahtu {
    sensor_data: Vec<Vec<SensorData>>,
    actions_since_last_scan: i64,
    tracks: Vec<Track>,
    current_time: u64,
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

        Self { sensor_data: v, actions_since_last_scan: 0, tracks: Vec::new(), current_time: 0 }
    }
    fn update_map(&mut self, context: &Context) {
        if let Some(scanned_data) = context.scanned_data() {
            self.actions_since_last_scan = 0;
            match scanned_data.scan_type {
                ScanType::Mono(_) => todo!(),
                ScanType::Omni => {
                    for scan_x in 0 .. SCANNING_DISTANCE {
                        if context.position().x + scan_x < SCANNING_DISTANCE / 2
                        {
                            continue;
                        }

                        let global_x = context.position().x + scan_x - SCANNING_DISTANCE / 2;
                        for scan_y in 0 .. SCANNING_DISTANCE {
                            if context.position().y + scan_y < SCANNING_DISTANCE / 2
                            {
                                continue;
                            }
                            let global_y = context.position().y + scan_y - SCANNING_DISTANCE / 2;
                            self.store_map_tile(context, global_x, global_y, scanned_data.data[scan_y][scan_x])
                        }
                    }
                }
            }
            if DEBUG_PRINTS {
                self.draw_map();
            }
        }
    }

    fn store_track(&mut self, context: &Context, global_x: usize, global_y: usize, id: &PlayerId) {
        if *id != *context.player_id()
        {
            self.tracks.push(Track {timestamp: self.current_time as i32, x: global_x, y: global_y});
        }
    }

    fn draw_map(&self)
    {
        let mut y_num = 0;
        for y in 0..self.sensor_data.len()  as isize {
            let mut line = y_num.to_string();
            y_num += 1;
            for x in 0..self.sensor_data[0].len() as isize
            {
                line.push_str(&get_cell_char(&self.get_map_tile(x, y)));
            }
            println!("{line}");
        }
    }

    fn store_map_tile(&mut self, context: &Context, global_x: usize, global_y: usize, data: MapCell)
    {
        if global_x >= MAX_WORLD_SIZE || global_y >= MAX_WORLD_SIZE
        {
            return;
        }
        match data {
            MapCell::Terrain(Terrain::Field) => {
                self.sensor_data[global_x][global_y] = SensorData::Empty;
            }
            MapCell::Player(id, terrain) => {
                if terrain != Terrain::Field
                {
                    self.sensor_data[global_x][global_y] = SensorData::Empty;
                }
                else {
                    self.sensor_data[global_x][global_y] = SensorData::Blocked;
                }
                self.store_track(context, global_x, global_y, &id);
            }
            _ => { self.sensor_data[global_x][global_y] = SensorData::Blocked; }
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
                context.position().y as isize - 1
            ),
            Orientation::East => self.get_map_tile(
                context.position().x as isize + 1,
                context.position().y as isize
            ),
            Orientation::South => self.get_map_tile(
                context.position().x as isize,
                context.position().y as isize + 1
            ),
            Orientation::West => self.get_map_tile(
                context.position().x as isize - 1,
                context.position().y as isize
            ),
            _ => SensorData::Blocked,
        }
    }

    fn can_see_unexplored_terrain_in_direction(&mut self, direction: &Orientation, context: &Context) -> bool
    {
        let mut x_offset = 0;
        let mut y_offset = 0;
        match direction {
            Orientation::North => y_offset = -1,
            Orientation::East => x_offset = 1,
            Orientation::South => y_offset = 1,
            Orientation::West => x_offset = -1,
            _ => return false
        }
        let mut x = context.position().x as isize;
        let mut y = context.position().y as isize;
        let mut tile = self.get_map_tile(x + x_offset, y + y_offset);
        while x > 0 && x < MAX_WORLD_SIZE as isize && y > 0 && y < MAX_WORLD_SIZE as isize {
            match tile {
                SensorData::NotScanned => break,
                SensorData::Empty => (),
                SensorData::Blocked => break,
            }
            x += x_offset;
            y += y_offset;
            tile = self.get_map_tile(x, y);
        }
        match tile {
            SensorData::NotScanned => true,
            _ => false,
        }
    }

    fn get_direction_with_unexplored_terrain(&mut self, context: &Context) -> Option<Orientation>
    {

        if self.can_see_unexplored_terrain_in_direction(context.orientation(), context) {
            return Some(context.orientation().clone());
        }
        if self.can_see_unexplored_terrain_in_direction(&Orientation::North, context) {
            return Some(Orientation::North);
        }
        if self.can_see_unexplored_terrain_in_direction(&Orientation::East, context) {
            return Some(Orientation::East);
        }
        if self.can_see_unexplored_terrain_in_direction(&Orientation::South, context) {
            return Some(Orientation::South);
        }
        if self.can_see_unexplored_terrain_in_direction(&Orientation::West, context) {
            return Some(Orientation::West);
        }
        return None;
    }

    fn check_tracks(&mut self) -> Option<Action> {
        let mut found_recent_track = false;
        for track in &self.tracks {
            if track.timestamp == self.current_time as i32 - 1 {
                // Current track -> fire on it!
                return Some(Action::Fire(Aiming::Positional(Position{ x: track.x, y: track.y})));
            }
            else if track.timestamp > self.current_time as i32 - 4 {
                // Recent track - check if it's still there to fire on..
                found_recent_track = true;

            }
        }
        // TODO: This should probably somehow track the tracks (so it won't keep looking after it has killed something)
        if found_recent_track {
            return Some(Action::Scan(ScanType::Omni));
        }
        return None;
    }

    fn get_next_action(&mut self, context: &Context) -> Action {
        if self.actions_since_last_scan > 5 {
            return Action::Scan(ScanType::Omni);
        }
        self.actions_since_last_scan += 1;

        if let Some(action) = self.check_tracks() {
            return action;
        }

        return match context.orientation() {
            Orientation::NorthEast => Action::Rotate(Rotation::CounterClockwise),
            Orientation::SouthEast => Action::Rotate(Rotation::CounterClockwise),
            Orientation::SouthWest => Action::Rotate(Rotation::CounterClockwise),
            Orientation::NorthWest => Action::Rotate(Rotation::CounterClockwise),
            _ => match self.get_direction_with_unexplored_terrain(context) {
                Some(direction) => {
                    if direction == context.orientation().clone()
                    {
                        match self.get_map_tile_in_front(context) {
                            SensorData::NotScanned => Action::Scan(ScanType::Omni),
                            SensorData::Empty => Action::Move(Direction::Forward),
                            SensorData::Blocked => Action::Rotate(Rotation::CounterClockwise),
                        }
                    }
                    else {
                        // Yes, this should really figure out the correct direction to rotate, but it'd also mean fixing the diagonals above..
                        Action::Rotate(Rotation::CounterClockwise)
                    }
                },
                None => {
                    match self.get_map_tile_in_front(context) {
                        SensorData::NotScanned => Action::Scan(ScanType::Omni),
                        SensorData::Empty => Action::Move(Direction::Forward),
                        SensorData::Blocked => Action::Rotate(Rotation::CounterClockwise),
                    }
                },
            }
        };
    }
}

impl Player for Rahtu {
    fn act(&mut self, context: Context) -> Action {
        self.current_time += 1;
        self.update_map(&context);
        if DEBUG_PRINTS {
            println!("{context}");
        }
        let action = self.get_next_action(&context);

        if DEBUG_PRINTS {
            dbg!(&action);
        }
        action
    }

    fn name(&self) -> String {
        "Rahtu".to_string()
    }
    fn is_ready(&self) -> bool {
        false
    }
}

fn get_cell_char(cell_data: &SensorData) -> String
{
    match cell_data {
        SensorData::NotScanned => return "-".to_string(),
        SensorData::Empty => return "_".to_string(),
        SensorData::Blocked => return "X".to_string(),
    }
}