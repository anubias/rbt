use std::collections::VecDeque;

use crate::{api::{
    context::Context, map_cell::{MapCell, Terrain}, orientation::Orientation, path_finder::MapReader, player::Details, position::{Position, SCANNING_DISTANCE}, scan::ScanType, world_size::MAX_WORLD_SIZE
}, WORLD_SIZE};

use super::shared::{self, Map, SensorData, Track};

impl Map {
    pub fn new() -> Self {
        let mut v = Vec::new();
        // initialize
        for _ in 0..MAX_WORLD_SIZE {
            let mut line = Vec::new();
            for _ in 0..MAX_WORLD_SIZE {
                line.push(shared::SensorData::NotScanned);
            }
            v.push(line);
        }
        Self {
            sensor_data: v,
            tracks: VecDeque::new(),
            initial_scan_done: false,
            actions_since_last_scan_northwest: 100,
            actions_since_last_scan_northeast: 100,
            actions_since_last_scan_southeast: 100, // "never"
            actions_since_last_scan_southwest: 100,
            
        }
    }

    pub fn update_map(&mut self, context: &Context) {
        if let Some(scanned_data) = context.scanned_data() {
            match scanned_data.scan_type {
                ScanType::Mono(Orientation::NorthWest) => {
                    self.actions_since_last_scan_northwest = 0;
                    for scan_x in 0..SCANNING_DISTANCE {
                        if context.position().x + scan_x < SCANNING_DISTANCE {
                            continue;
                        }

                        let global_x = context.position().x + scan_x - (SCANNING_DISTANCE - 1);
                        for scan_y in 0..SCANNING_DISTANCE {
                            if context.position().y + scan_y < (SCANNING_DISTANCE - 1) {
                                continue;
                            }
                            let global_y = context.position().y + scan_y - (SCANNING_DISTANCE - 1);
                            self.store_map_tile(
                                context,
                                global_x,
                                global_y,
                                scanned_data.data[scan_y][scan_x],
                            )
                        }
                    }
                },
                ScanType::Mono(Orientation::NorthEast) => {
                    self.actions_since_last_scan_northeast = 0;
                    for scan_x in 0..SCANNING_DISTANCE {
                        let global_x = context.position().x + scan_x;
                        for scan_y in 0..SCANNING_DISTANCE {
                            if context.position().y + scan_y < (SCANNING_DISTANCE - 1) {
                                continue;
                            }
                            let global_y = context.position().y + scan_y - (SCANNING_DISTANCE - 1);
                            self.store_map_tile(
                                context,
                                global_x,
                                global_y,
                                scanned_data.data[scan_y][scan_x],
                            )
                        }
                    }
                },
                ScanType::Mono(Orientation::SouthEast) => {
                    self.actions_since_last_scan_southeast = 0;
                    for scan_x in 0..SCANNING_DISTANCE {
                        let global_x = context.position().x + scan_x;
                        for scan_y in 0..SCANNING_DISTANCE {
                            let global_y = context.position().y + scan_y;
                            self.store_map_tile(
                                context,
                                global_x,
                                global_y,
                                scanned_data.data[scan_y][scan_x],
                            )
                        }
                    }
                },
                ScanType::Mono(Orientation::SouthWest) => {
                    self.actions_since_last_scan_southwest = 0;
                    for scan_x in 0..SCANNING_DISTANCE {
                        if context.position().x + scan_x < (SCANNING_DISTANCE - 1) {
                            continue;
                        }

                        let global_x = context.position().x + scan_x - (SCANNING_DISTANCE - 1);
                        for scan_y in 0..SCANNING_DISTANCE {
                            let global_y = context.position().y + scan_y;
                            self.store_map_tile(
                                context,
                                global_x,
                                global_y,
                                scanned_data.data[scan_y][scan_x],
                            )
                        }
                    }
                },
                ScanType::Mono(_) => (),
                ScanType::Omni => {
                    self.initial_scan_done = true;
                    for scan_x in 0..SCANNING_DISTANCE {
                        if context.position().x + scan_x < SCANNING_DISTANCE / 2 {
                            continue;
                        }

                        let global_x = context.position().x + scan_x - SCANNING_DISTANCE / 2;
                        for scan_y in 0..SCANNING_DISTANCE {
                            if context.position().y + scan_y < SCANNING_DISTANCE / 2 {
                                continue;
                            }
                            let global_y = context.position().y + scan_y - SCANNING_DISTANCE / 2;
                            self.store_map_tile(
                                context,
                                global_x,
                                global_y,
                                scanned_data.data[scan_y][scan_x],
                            )
                        }
                    }
                }
            }
            //println!("{}", context.scanned_data().as_ref().unwrap());
            //self.draw_map();
        }
    }

    fn store_track(&mut self, context: &Context, global_x: usize, global_y: usize, player: Details) {
        if player.id != context.player_details().id && player.alive {
            self.tracks.push_front(Track {
                timestamp: context.turn() as i32,
                x: global_x,
                y: global_y,
            });
        }
    }
/*
    fn draw_map(&self) {
        let mut y_num = 0;
        for y in 0..self.sensor_data.len() as isize {
            let mut line = y_num.to_string();
            y_num += 1;
            for x in 0..self.sensor_data[0].len() as isize {
                line.push_str(&self.get_cell_char(&self.get_map_tile(x, y)));
            }
            println!("{line}");
        }
    }

    fn get_cell_char(&self, cell_data: &SensorData) -> String {
        match cell_data {
            SensorData::NotScanned => return "-".to_string(),
            SensorData::Empty => return "_".to_string(),
            SensorData::Blocked => return "X".to_string(),
        }
    }
*/
    fn store_map_tile(
        &mut self,
        context: &Context,
        global_x: usize,
        global_y: usize,
        data: MapCell,
    ) {
        if global_x >= MAX_WORLD_SIZE || global_y >= MAX_WORLD_SIZE {
            return;
        }
        let mut cell = Terrain::Swamp;
        let mut player = None;
        match data {
            MapCell::Explosion(_, terrain) => cell = terrain,
            MapCell::Player(details, terrain) => {
                player = Some(details);
                cell = terrain;
            }
            MapCell::Shell(_, terrain) => cell = terrain,
            MapCell::Terrain(terrain) => cell = terrain,
            MapCell::Unallocated => (),
        }

        match cell {
            Terrain::Field => self.sensor_data[global_x][global_y] = SensorData::Empty,
            Terrain::Lake => self.sensor_data[global_x][global_y] = SensorData::Blocked,
            Terrain::Forest(_) => self.sensor_data[global_x][global_y] = SensorData::Blocked,
            Terrain::Swamp => self.sensor_data[global_x][global_y] = SensorData::Blocked,
        }

        if let Some(player) = player {
            self.store_track(context, global_x, global_y, player);
            if !player.alive {
                self.sensor_data[global_x][global_y] = SensorData::Blocked;
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

    pub fn get_any_unexplored_tile(&self) -> Option<Position> {
        for x in 3..(WORLD_SIZE.x - 3) as isize {
            for y in 3..(WORLD_SIZE.y - 3) as isize {
                match self.get_map_tile(x, y) {
                    SensorData::NotScanned => return Some(Position { x: x as usize, y: y as usize }),
                    _ => (),
                }
            }
        }
        return None;
    }

    pub fn available_cardinal_shot(&self, context: &Context) -> Option<Orientation> {
        for track in &self.tracks {
            if track.timestamp as usize == context.turn() || track.timestamp as usize == context.turn() - 1 {
                if let Some(direction) = self.get_direction_to_position(context, &Position{ x: track.x, y: track.y}) {
                    return Some(direction);
                }
            }
        }
        return None;
    }

    pub fn get_direction_to_position(&self, context: &Context, position: &Position) -> Option<Orientation> {
        if position.x == context.position().x {
            if position.x < context.position().x {
                return Some(Orientation::West);
            }
            else {
                return Some(Orientation::East);
            }
        }
        if position.y == context.position().y {
            if position.y < context.position().y {
                return Some(Orientation::North);
            }
            else {
                return Some(Orientation::South);
            }
        }

        let relative_x = position.x as isize - context.position().x as isize;
        let relative_y = position.y as isize - context.position().y as isize;
        if  relative_x == relative_y {
            if relative_x < 0 {
                return Some(Orientation::NorthWest);
            }
            else {
                return Some(Orientation::SouthEast);
            }
        }
        if relative_x == -relative_y {
            if relative_x < 0 {
                return Some(Orientation::SouthWest);
            }
            else {
                return Some(Orientation::NorthEast);
            }
        }
        return None;
    }

    pub fn get_closest_ordinal_direction_to_position(&self, context: &Context, position: &Position) -> Orientation {
        let relative_x = position.x as isize - context.position().x as isize;
        let relative_y = position.y as isize - context.position().y as isize;
        if  relative_x < 0 {
            if relative_y < 0 {
                return Orientation::NorthWest;
            }
            else {
                return Orientation::SouthWest;
            }
        }
        else {
            if relative_y < 0 {
                return Orientation::NorthEast;
            }
            else {
                return Orientation::SouthEast;
            }
        }
    }

}

impl MapReader for Map {
    fn read_at(&self, position: &crate::api::position::Position) -> MapCell {
        match self.get_map_tile(position.x as isize, position.y as isize) {
            SensorData::NotScanned => MapCell::Unallocated,
            SensorData::Empty => MapCell::Terrain(Terrain::Field),
            SensorData::Blocked => MapCell::Terrain(Terrain::Forest(crate::api::map_cell::TreeType::Deciduous)),
        }
    }
}