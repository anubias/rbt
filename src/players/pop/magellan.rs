use crate::api::{
    action::Action,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    path_finder::MapReader,
    player::PlayerId,
    position::{Position, CARDINAL_SHOT_DISTANCE, POSITIONAL_SHOT_DISTANCE, SCANNING_DISTANCE},
    scan::ScanResult,
    world_size::{WorldSize, MAX_WORLD_SIZE},
};

#[derive(Clone)]
pub struct Magellan {
    discovered_map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
}

impl MapReader for Magellan {
    fn read_at(&self, position: &Position) -> MapCell {
        self.validate_coordinates_or_panic(position.y, position.x);
        self.discovered_map[position.y][position.x]
    }
}

impl Magellan {
    pub fn new() -> Self {
        Self {
            discovered_map: Box::new([[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
        }
    }

    pub fn update_map(&mut self, scan_result: &ScanResult, start_x: isize, start_y: isize) {
        for i in 0..SCANNING_DISTANCE {
            let map_y = start_y + i as isize;

            if map_y >= 0 && map_y < MAX_WORLD_SIZE as isize {
                for j in 0..SCANNING_DISTANCE {
                    let map_x = start_x + j as isize;

                    if map_x >= 0 && map_x < MAX_WORLD_SIZE as isize {
                        let (longitude, latitude) = (map_x as usize, map_y as usize);
                        self.discovered_map[latitude][longitude] = scan_result.data[i][j];
                    }
                }
            }
        }
    }

    pub fn compute_quadrant(&self, from: &Position, to: &Position) -> Orientation {
        let (dx, dy) = from.manhattan_distance(to);

        match (dx, dy) {
            (..0, ..0) => Orientation::SouthEast,
            (..0, 0..) => Orientation::NorthEast,
            (0.., 0..) => Orientation::NorthWest,
            (0.., ..0) => Orientation::SouthWest,
        }
    }

    pub fn count_unallocated_cells(&self, world_size: &WorldSize) -> usize {
        let mut count = 0;

        for i in 0..world_size.y {
            for j in 0..world_size.x {
                if self.discovered_map[i][j] == MapCell::Unallocated {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn reorient(
        &self,
        curr_orientation: &Orientation,
        next_orientation: &Orientation,
    ) -> Action {
        if *next_orientation == *curr_orientation {
            Action::Move(Direction::Forward)
        } else if *next_orientation == curr_orientation.opposite() {
            Action::Move(Direction::Backward)
        } else {
            let (rotation, steps) = curr_orientation.quick_turn(&next_orientation);
            if steps <= 2 {
                Action::Rotate(rotation)
            } else {
                Action::Rotate(rotation.opposite())
            }
        }
    }

    pub fn find_alignment(&self, from: &Position, to: &Position) -> Option<Orientation> {
        let (dx, dy) = from.manhattan_distance(to);

        match (dx, dy) {
            (0, ..0) => Some(Orientation::South),
            (0, 0..) => Some(Orientation::North),
            (..0, 0) => Some(Orientation::East),
            (0.., 0) => Some(Orientation::West),
            _ => {
                if dx.abs() == dy.abs() {
                    if dx < 0 && dy < 0 {
                        Some(Orientation::SouthEast)
                    } else if dx < 0 && dy > 0 {
                        Some(Orientation::NorthEast)
                    } else if dx > 0 && dy < 0 {
                        Some(Orientation::SouthWest)
                    } else {
                        Some(Orientation::NorthWest)
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn find_all_enemies(&self, ignore_id: PlayerId, world_size: &WorldSize) -> Vec<Position> {
        let mut result = Vec::new();

        for i in 0..world_size.y {
            for j in 0..world_size.x {
                if let MapCell::Player(details, _) = self.discovered_map[i][j] {
                    if details.alive && details.id != ignore_id {
                        result.push(Position { x: j, y: i });
                    }
                }
            }
        }

        result
    }

    pub fn find_forward_scans(&self, orientation: Orientation) -> Vec<Orientation> {
        match orientation {
            Orientation::North => vec![Orientation::NorthEast, Orientation::NorthWest],
            Orientation::NorthEast => vec![
                Orientation::East,
                Orientation::NorthEast,
                Orientation::North,
            ],
            Orientation::East => vec![Orientation::SouthEast, Orientation::NorthEast],
            Orientation::SouthEast => vec![
                Orientation::South,
                Orientation::SouthEast,
                Orientation::East,
            ],
            Orientation::South => vec![Orientation::SouthWest, Orientation::SouthEast],
            Orientation::SouthWest => vec![
                Orientation::West,
                Orientation::SouthWest,
                Orientation::South,
            ],
            Orientation::West => vec![Orientation::NorthWest, Orientation::SouthWest],
            Orientation::NorthWest => vec![
                Orientation::North,
                Orientation::NorthWest,
                Orientation::West,
            ],
        }
    }

    pub fn find_closest_position(
        &self,
        reference: &Position,
        positions: &Vec<Position>,
    ) -> Option<Position> {
        let mut closest_distance = f32::MAX;
        let mut closest_position = None;

        for position in positions {
            let distance = reference.pythagorean_distance(&position);
            if distance < closest_distance {
                closest_distance = distance;
                closest_position = Some(position.clone());
            }
        }

        closest_position
    }

    pub fn list_all_unexplored(
        &self,
        unreachable: &Vec<Position>,
        world_size: &WorldSize,
    ) -> Vec<Position> {
        let mut result = Vec::new();

        for i in 0..world_size.y {
            for j in 0..world_size.x {
                let position = Position { x: j, y: i };
                if !unreachable.contains(&position)
                    && self.discovered_map[i][j] == MapCell::Unallocated
                {
                    let mut edge_cell = false;
                    for adjacent in position.list_adjacent_positions(&world_size) {
                        if self.read_at(&adjacent) != MapCell::Unallocated {
                            edge_cell = true;
                            break;
                        }
                    }

                    if edge_cell {
                        result.push(position);
                    }
                }
            }
        }

        result
    }

    pub fn list_safe_firing_positions(
        &self,
        target: &Position,
        world_size: &WorldSize,
    ) -> Vec<Position> {
        let mut result = Vec::new();
        let min_safe_distance = POSITIONAL_SHOT_DISTANCE + 1;
        let max_shot_distance = CARDINAL_SHOT_DISTANCE;

        for safe_distance in min_safe_distance..max_shot_distance + 1 {
            if let Some(north) =
                self.follow_cardinal(target, safe_distance, &Orientation::North, world_size)
            {
                result.push(north);
            }
            if let Some(north_east) =
                self.follow_cardinal(target, safe_distance, &Orientation::NorthEast, world_size)
            {
                result.push(north_east);
            }
            if let Some(east) =
                self.follow_cardinal(target, safe_distance, &Orientation::East, world_size)
            {
                result.push(east);
            }
            if let Some(south_east) =
                self.follow_cardinal(target, safe_distance, &Orientation::SouthEast, world_size)
            {
                result.push(south_east);
            }
            if let Some(south) =
                self.follow_cardinal(target, safe_distance, &Orientation::South, world_size)
            {
                result.push(south);
            }
            if let Some(south_west) =
                self.follow_cardinal(target, safe_distance, &Orientation::SouthWest, world_size)
            {
                result.push(south_west);
            }
            if let Some(west) =
                self.follow_cardinal(target, safe_distance, &Orientation::West, world_size)
            {
                result.push(west);
            }
            if let Some(north_west) =
                self.follow_cardinal(target, safe_distance, &Orientation::NorthWest, world_size)
            {
                result.push(north_west);
            }
        }

        result = result
            .into_iter()
            .filter(|x| {
                matches!(self.read_at(x), MapCell::Terrain(Terrain::Field))
                    || matches!(self.read_at(x), MapCell::Unallocated)
            })
            .collect();

        result
    }
}

impl Magellan {
    fn follow_cardinal(
        &self,
        reference: &Position,
        steps: usize,
        orientation: &Orientation,
        world_size: &WorldSize,
    ) -> Option<Position> {
        let mut result = Some(reference.clone());

        for _ in 0..steps {
            if let Some(r) = result {
                result = r.follow(orientation, world_size);
            }
        }

        result
    }

    fn validate_coordinates_or_panic(&self, latitude: usize, longitude: usize) {
        if latitude >= MAX_WORLD_SIZE || longitude >= MAX_WORLD_SIZE {
            panic!("Invalid array coordinates [[{latitude},{longitude}]] (max={MAX_WORLD_SIZE})")
        }
    }
}
