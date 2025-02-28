use std::{cmp::max, collections::HashMap};

use super::super::player::*;

pub struct Map {
    map: [[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE],
}

impl Map {
    pub fn new() -> Self {
        Self {
            map: [[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE],
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
            MapCell::Unallocated
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
                let map_cell = Map::resolve_terrain(&scan_result.data[y][x]);
                self.set_cell(map_x, map_y, map_cell);
            }
        }
    }

    fn resolve_terrain(map_cell: &MapCell) -> MapCell {
        match map_cell {
            MapCell::Player(player_details, terrain) => {
                if player_details.alive {
                    MapCell::Terrain(terrain.clone())
                } else {
                    // Map dead player as an obstacle
                    MapCell::Terrain(Terrain::Forest(TreeType::default()))
                }
            }
            MapCell::Explosion(_, terrain) => MapCell::Terrain(terrain.clone()),
            MapCell::Shell(_, terrain) => MapCell::Terrain(terrain.clone()),
            MapCell::Terrain(terrain) => MapCell::Terrain(terrain.clone()),
            MapCell::Unallocated => MapCell::Unallocated,
        }
    }

    pub fn find_path(
        &self,
        start_position: Position,
        start_orientation: Orientation,
        target_position: &Position,
    ) -> Option<Vec<Position>> {
        // A* pathfinding algorithm

        // Table to store the cheapest known path cost from start_position to each of the map cells
        let mut path_cost = [[f32::INFINITY; MAX_WORLD_SIZE]; MAX_WORLD_SIZE];
        path_cost[start_position.y][start_position.x] = 0.0;

        // Map to store the path candidate nodes that needs to be further expanded.
        // Key is the node position, value contains estimated cost for the path that goes through
        // this node and the player orientation in this node.
        let mut path_candidates = HashMap::from([(start_position, (0.0f32, start_orientation))]);

        // Map to store the previous nodes in the cheapest known path.
        // Key is the node in the path and value is the previous node in the path.
        let mut path_history: HashMap<Position, Position> = HashMap::new();

        // While there is path candidate nodes to expand, find the node/path with the lowest
        // estimated total cost. I.e. expand the most promising path.
        while let Some((candidate_position, candidate_orientation)) = path_candidates
            .iter()
            .min_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
            .map(|a| (a.0.clone(), a.1 .1.clone()))
        {
            // Construct and return path if reached the target position
            if &candidate_position == target_position {
                return Map::construct_path(&path_history, target_position);
            }

            path_candidates.remove(&candidate_position);
            let candidate_cost = path_cost[candidate_position.y][candidate_position.x];

            // Expand the node to all neighbors
            for neighbor in self.get_neighbors(&candidate_position) {
                let path_cost_to_neighbor =
                    self.get_path_cost(&candidate_position, &candidate_orientation, &neighbor);
                let neighbor_cost = candidate_cost + path_cost_to_neighbor;

                // If a new cheaper path to neighbor was found
                if neighbor_cost < path_cost[neighbor.y][neighbor.x] {
                    path_cost[neighbor.y][neighbor.x] = neighbor_cost;

                    let distance = neighbor.manhattan_distance(target_position);
                    let estimated_cost_to_target = max(distance.0.abs(), distance.1.abs()) as f32;
                    let estimated_total_cost = neighbor_cost + estimated_cost_to_target;
                    let orientation = candidate_position.get_orientation_to(&neighbor);
                    path_candidates.insert(neighbor.clone(), (estimated_total_cost, orientation));

                    path_history.insert(neighbor, candidate_position.clone());
                }
            }
        }

        // All candidates expanded and no path to target was found
        return None;
    }

    fn get_neighbors(&self, position: &Position) -> Vec<Position> {
        let mut neighbors = Vec::with_capacity(8);

        for (delta_x, delta_y) in [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ] {
            let neighbor_x = position.x as isize + delta_x;
            let neighbor_y = position.y as isize + delta_y;
            if self.is_valid_coordinate(neighbor_x, neighbor_y) {
                neighbors.push(Position {
                    x: neighbor_x as usize,
                    y: neighbor_y as usize,
                });
            }
        }

        return neighbors;
    }

    fn get_path_cost(
        &self,
        current_position: &Position,
        current_orientation: &Orientation,
        next_position: &Position,
    ) -> f32 {
        match self.get_cell(next_position) {
            MapCell::Terrain(Terrain::Field) | MapCell::Unallocated => {
                // Calculate turns needed to reach the target position
                let target_orientation = current_position.get_orientation_to(next_position);
                let turning_cost = current_orientation
                    .quick_turn_bidirectional(&target_orientation)
                    .1 as f32;
                turning_cost + 1.0
            }
            _ => f32::INFINITY, // Cannot enter the next_position cell
        }
    }

    fn construct_path(
        path_history: &HashMap<Position, Position>,
        target_position: &Position,
    ) -> Option<Vec<Position>> {
        let mut path = Vec::from([target_position.clone()]);
        let mut current = target_position;

        while let Some(position) = path_history.get(current) {
            current = position;
            path.push(current.clone());
        }

        match path.len() {
            ..=1 => None,
            2.. => {
                path.pop(); // Remove the start position from the path
                path.reverse();
                Some(path)
            }
        }
    }

    #[allow(dead_code)]
    pub fn print(&self, world_size: &WorldSize, path: Option<&Vec<Position>>) {
        let mut output = String::new();

        for y in 0..world_size.y {
            for x in 0..world_size.x {
                let in_path = path.is_some_and(|p| p.iter().any(|pos| pos.x == x && pos.y == y));
                let map_cell = match in_path {
                    true => MapCell::Shell(INVALID_PLAYER, Terrain::Field),
                    false => self.map[y][x],
                };
                output = format!("{output}{}", map_cell);
            }
            output = format!("{output}\n");
        }

        println!("{output}");
    }
}
