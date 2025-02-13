use super::super::player::*;

pub struct Map {
    map: [[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE],
}

impl Map {
    pub fn new() -> Self {
        Self {
            map: [[MapCell::Unknown; MAX_WORLD_SIZE]; MAX_WORLD_SIZE],
        }
    }

    fn is_valid_coordinate(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.map[0].len() as isize && y >= 0 && y < self.map.len() as isize
    }

    pub fn get_cell(&self, position: &Position) -> MapCell {
        self.get_cell_isize(position.x as isize, position.y as isize)
    }

    fn get_cell_isize(&self, x: isize, y: isize) -> MapCell {
        if self.is_valid_coordinate(x, y) {
            self.map[y as usize][x as usize]
        } else {
            MapCell::Unknown
        }
    }

    fn set_cell(&mut self, x: isize, y: isize, map_cell: MapCell) {
        if self.is_valid_coordinate(x, y) {
            self.map[y as usize][x as usize] = map_cell;
        }
    }

    pub fn collect_data(&mut self, scan_result: &ScanResult, my_world_position: &Position) {
        let (scan_world_x, scan_world_y) = scan_result.get_world_position(my_world_position);

        for y in 0..scan_result.data.len() {
            for x in 0..scan_result.data[y].len() {
                let map_x = x as isize + scan_world_x;
                let map_y = y as isize + scan_world_y;
                let map_cell = Map::resolve_cell(&scan_result.data[y][x]);
                self.set_cell(map_x, map_y, map_cell);
            }
        }
    }

    fn resolve_cell(map_cell: &MapCell) -> MapCell {
        match map_cell {
            MapCell::Player(player_details, terrain) => {
                if player_details.is_alive() {
                    MapCell::Terrain(terrain.clone())
                } else {
                    // Map dead player as an obstacle
                    MapCell::Terrain(Terrain::Forest(TreeType::default()))
                }
            }
            MapCell::Explosion(_, terrain) => MapCell::Terrain(terrain.clone()),
            MapCell::Shell(_, terrain) => MapCell::Terrain(terrain.clone()),
            MapCell::Terrain(terrain) => MapCell::Terrain(terrain.clone()),
            MapCell::Unknown => MapCell::Unknown,
        }
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        let mut line = String::new();

        for y in 0..self.map.len() {
            for x in 0..self.map[y].len() {
                line = format!("{line}{}", self.map[y][x]);
            }
            line = format!("{line}\n");
        }

        println!("{line}");
    }
}
