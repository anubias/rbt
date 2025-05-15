use super::types::*;

impl MapReader for MappedTerrain {
    fn read_at(&self, position: &Position) -> MapCell {
        let tile = &self.map[position.y][position.x];
        match &tile.surface {
            Surface::Terrain(t) => MapCell::Terrain(*t),
            Surface::Unallocated => MapCell::Unallocated,
        }
    }
}

impl MapReader for &MappedTerrain {
    fn read_at(&self, position: &Position) -> MapCell {
        (*self).read_at(position)  // Delegate
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Surface {
    Terrain(Terrain),
    Unallocated,
}

#[allow(dead_code)]
impl Surface {
    fn from_cell(&self, cell: &MapCell) -> Surface {
        match cell {
            MapCell::Terrain(t) => Surface::Terrain(t.clone()),
            MapCell::Explosion(_, t) => Surface::Terrain(t.clone()),
            MapCell::Player(_, t) => Surface::Terrain(t.clone()),
            MapCell::Shell(_, t) => Surface::Terrain(t.clone()),
            MapCell::Unallocated => Surface::Unallocated,
        }
    }

    fn to_map_cell(&self) -> MapCell {
        match self {
            Surface::Terrain(t) => MapCell::Terrain(*t),
            Surface::Unallocated => MapCell::Unallocated,
        }
    }
}

impl MapCell {
    fn to_surface(&self) -> Surface {
        match self {
            MapCell::Terrain(t) => Surface::Terrain(t.clone()),
            MapCell::Explosion(_, t) => Surface::Terrain(t.clone()),
            MapCell::Player(_, t) => Surface::Terrain(t.clone()),
            MapCell::Shell(_, t) => Surface::Terrain(t.clone()),
            MapCell::Unallocated => Surface::Unallocated,
        }
    }
    fn terrain_danger(&self, t: &Terrain) -> f32 {
        match t {
            Terrain::Field => 0.0,
            Terrain::Lake => 1.2,
            Terrain::Forest(_) => 1.0,
            Terrain::Swamp => 1.5,
        }
    }

    fn positional_danger(&self) -> f32 {
        match self {
            MapCell::Player(_, t) => match t {
                Terrain::Field => 1.0,
                Terrain::Lake => 0.0,   // Drowned, should be dead
                Terrain::Forest(_) | Terrain::Swamp => self.terrain_danger(t),
            },

            MapCell::Terrain(t)
            | MapCell::Explosion(_, t)
            | MapCell::Shell(_, t) => self.terrain_danger(t),

            MapCell::Unallocated => 0.1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorldTile {
    pub surface: Surface,
    pub last_seen_turn: usize,
    pub times_seen: u32,
    pub danger_score: f32,    // Higher is more, account only terrain, not dynamic details as other tanks
    pub strategic_value: f32, // Higher is better
    pub times_hit: u32,
    pub last_damage_turn: usize,
}

impl WorldTile {
    pub fn is_transitable(&self) -> bool {
        match self.surface {
            Surface::Terrain(Terrain::Field) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MappedTerrain {
    map: Vec<Vec<WorldTile>>,
    choke_points: Vec<Position>,
}

impl MappedTerrain {
    pub fn new(world_size: WorldSize) -> Self {
        let default_tile = WorldTile {
            surface: Surface::Unallocated,
            last_seen_turn: 0,
            times_seen: 0,
            danger_score: 0.0,
            strategic_value: 0.0,
            times_hit: 0,
            last_damage_turn: 0,
        };

        let mut map = Self {
            map: vec![vec![default_tile.clone(); world_size.x]; world_size.y],
            choke_points: Vec::new(),
        };

        map.set_borders(&world_size);
        map
    }

    pub fn peek_terrain(&self) -> &Vec<Vec<WorldTile>> {
        &self.map
    }

    pub fn world_size(&self) -> WorldSize {
        WorldSize {
            x: self.map[0].len(),
            y: self.map.len()
        }
    }

    pub fn is_unexplored(&self, pos: &Position) -> bool {
        self.map[pos.y][pos.x].surface == Surface::Unallocated
    }

    pub fn is_choke_point(&self, pos: &Position) -> bool {
        self.choke_points.contains(pos)
    }

    pub fn unexplored_cells(&self) -> Vec<Position> {
        let mut unexplored = Vec::new();
        for (y, row) in self.map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.surface == Surface::Unallocated {
                    unexplored.push(Position { x, y });
                }
            }
        }
        unexplored
    }

    // FIXME: Nono not here, actions do not belong to the map
    pub fn get_best_movement_action(&self, context: &Context) -> Action {
        let pos = context.position();
        let orientation = context.player_details().orientation;
        let world_size = context.world_size();
        let previous_action = context.previous_action();

        // Get last position based on previous action
        // FIXME: Own tank history now is cached too!!
        let mut last_position = None;
        if let Action::Move(direction) = previous_action {
            let last_orientation = match direction {
                Direction::Forward => orientation.opposite(),
                Direction::Backward => orientation,
            };
            last_position = pos.follow(&last_orientation, world_size);
        }

        let directions = [
            (orientation, Direction::Forward),
            (orientation.opposite(), Direction::Backward),
            (orientation.rotated_counter_clockwise(), Direction::Forward),
            (orientation.rotated_clockwise(), Direction::Forward),
        ];

        let mut best_score = f32::NEG_INFINITY;
        let mut best_action = None;

        let mut found_any_valid_move = false;
        let mut chokepoint_actions = Vec::new();

        for (check_orientation, movement) in &directions {
            let next_pos = if let Some(np) = pos.follow(check_orientation, world_size) {
                np
            } else {
                continue; // out of bounds
            };

            if !self.is_position_transitable(&next_pos) {
                continue;
            }

            found_any_valid_move = true;

            if let Some(ref last_pos) = last_position {
                if next_pos.x == last_pos.x && next_pos.y == last_pos.y {
                    continue; // Skip position we just left
                }
            }

            let tile = &self.map[next_pos.y][next_pos.x];
            let mut score = -tile.danger_score * 5.0;
            score += tile.strategic_value * 2.0;

            let is_chokepoint_move = self.is_choke_point(&next_pos);
            if is_chokepoint_move {
                score -= 5.0;

                let action = if check_orientation == &orientation {
                    Action::Move(movement.clone())
                } else if movement == &Direction::Forward {
                    let (rotation, _) = orientation.quick_turn(check_orientation);
                    Action::Rotate(rotation)
                } else {
                    Action::Move(movement.clone())
                };

                chokepoint_actions.push((action, score));
                continue;
            }

            // Additional penalties for being near chokepoints
            for choke_point in &self.choke_points {
                let distance = next_pos.pythagorean_distance(&choke_point);
                if distance < 3.0 {
                    score -= 3.0 / distance; // More penalty when closer to chokepoints
                }
            }

            // Bonus for unexplored areas nearby
            if self.count_nearby_unexplored(&next_pos, 3) > 0 {
                score += 3.0;
            }

            // Slight bonus for continuing forward to maintain momentum
            if movement == &Direction::Forward && check_orientation == &orientation {
                score += 1.0;
            }

            // Add a small random factor
            use std::time::SystemTime;
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().subsec_nanos();
            let random_factor = (now as f32 % 1000.0) / 2000.0;
            score += random_factor;

            if score > best_score {
                best_score = score;

                best_action = if check_orientation == &orientation {
                    Some(Action::Move(movement.clone()))
                } else if movement == &Direction::Forward {
                    let (rotation, _) = orientation.quick_turn(check_orientation);
                    Some(Action::Rotate(rotation))
                } else {
                    Some(Action::Move(movement.clone()))
                };
            }
        }

        //no good non-chokepoint action but have chokepoint options available
        if best_action.is_none() && !chokepoint_actions.is_empty() && found_any_valid_move {
            chokepoint_actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            best_action = Some(chokepoint_actions[0].0.clone());
        }

        best_action.unwrap_or_else(|| Action::Scan(ScanType::Omni))
    }

    pub fn update_tile(&mut self, pos: &Position, cell: &MapCell, turn: usize) {

        let tile = &mut self.map[pos.y][pos.x];
        tile.surface = cell.to_surface();
        tile.last_seen_turn = turn;
        tile.times_seen += 1;


        /* Meh...
            danger_score: f32,    // Higher is more, account only terrain, not dynamic details as other tanks
            strategic_value: f32, // Higher is better
            times_hit: u32,
            last_damage_turn: usize,
         */

        self.calculate_strategic_value(&cell, &pos);
        self.update_danger_assessment(&pos, SCANNING_DISTANCE);
        self.update_chokepoint_detection(&pos);
    }

    pub fn has_adjacent_forest(&self, pos: &Position, world_size: &WorldSize) -> bool {
        for &orientation in &Orientation::all() {
            if let Some(adj_pos) = pos.follow(&orientation, world_size) {
                if let Surface::Terrain(Terrain::Forest(_)) = self.map[adj_pos.y][adj_pos.x].surface {
                    return true;
                }
            }
        }

        false
    }

    pub fn actions_to_nearest_unexplored(&self, context: &Context) -> Option<Vec<Action>> {
        let my_pos = context.position();
        let my_ori = context.player_details().orientation;
        let world_size = context.world_size();

        // Get unexplored cells and sort by distance
        let unexplored = self.unexplored_cells();
        // Filter to only include unexplored cells adjacent to transitable terrain
        let reachable_unexplored: Vec<Position> = unexplored.into_iter()
            .filter(|pos| {
                // Check if any adjacent cell is transitable (using existing method pattern)
                for &orientation in &Orientation::all() {
                    if let Some(adj_pos) = pos.follow(&orientation, world_size) {
                        if self.is_position_transitable(&adj_pos) {
                            return true;
                        }
                    }
                }
                false
            })
            .collect();

        // Sort filtered cells by distance
        let mut sorted_unexplored = reachable_unexplored;
        sorted_unexplored.sort_by_key(|pos| my_pos.pythagorean_distance(pos) as usize);

        for target in sorted_unexplored.iter() {
            let mut pf = PathFinder::new(self, self.world_size());
            let mut comp = pf.compute_shortest_path(my_pos, target, &my_ori);
            let path = comp.to_path();

            if path.is_empty() {
                continue;
            }

            // Check path safety (forward order, skip the unexplored target)
            let is_safe = path.iter().take(path.len().saturating_sub(1)).all(|p| {
                let is_trans = self.is_position_transitable(p);
                is_trans
            });

            if is_safe && path.len() > 1 {
                let actions = comp.to_actions();

                // Validate the final position is safe by simulating the path
                let mut sim_pos = my_pos.clone();
                let mut sim_ori = my_ori;
                let mut safe_actions = Vec::new();

                for action in &actions {
                    // Simulate this action
                    match action {
                        Action::Move(Direction::Forward) => {
                            if let Some(new_pos) = sim_pos.follow(&sim_ori, world_size) {
                                // Only include action if destination is transitable
                                if self.is_position_transitable(&new_pos) ||
                                // Or if it's the target unexplored cell
                                new_pos == *target {
                                    sim_pos = new_pos;
                                    safe_actions.push(action.clone());
                                } else {
                                    break; // Stop at non-transitable
                                }
                            }
                        },
                        Action::Move(Direction::Backward) => {
                            if let Some(new_pos) = sim_pos.follow(&sim_ori.opposite(), world_size) {
                                if self.is_position_transitable(&new_pos) {
                                    sim_pos = new_pos;
                                    safe_actions.push(action.clone());
                                } else {
                                    break;
                                }
                            }
                        },
                        Action::Rotate(rotation) => {
                            sim_ori = match rotation {
                                Rotation::Clockwise => sim_ori.rotated_clockwise(),
                                Rotation::CounterClockwise => sim_ori.rotated_counter_clockwise(),
                            };
                            safe_actions.push(action.clone());
                        },
                        _ => {}
                    }
                }

                if !safe_actions.is_empty() {
                    return Some(safe_actions);
                }
            }
        }

        None
    }

    pub fn is_position_transitable(&self, pos: &Position) -> bool {
        self.map[pos.y][pos.x].is_transitable()
    }
}

impl MappedTerrain {
    fn calculate_strategic_value(&mut self, cell: &MapCell, pos: &Position) {
        let control_bonus = self.calculate_area_control(&pos);

        let tile: &mut WorldTile = &mut self.map[pos.y][pos.x];

        // Higher value for positions with good cover
        let cover_bonus = match cell {
            MapCell::Terrain(Terrain::Forest(_)) => 2.0,
            MapCell::Terrain(Terrain::Lake) => 1.75,
            MapCell::Terrain(Terrain::Swamp) => 1.5,
            _ => 1.0,
        };

        // Lower value for positions frequently under attack
        let danger_penalty = tile.danger_score * 0.5;

        tile.strategic_value = cover_bonus + control_bonus - danger_penalty;
    }

    fn calculate_area_control(&self, _pos: &Position) -> f32 {
        // TODO: Count visible tiles from this position
        let visible_tiles = 8.0; // Basic 8-direction visibility
        visible_tiles / 8.0 // Normalized score
    }

    fn update_danger_assessment(&mut self, pos: &Position, scanning_distance: usize) {
        let world_size = self.world_size();
        let min_y = pos.y.saturating_sub(scanning_distance).max(1);
        let max_y = (pos.y + scanning_distance).min(world_size.y - 2);
        let min_x = pos.x.saturating_sub(scanning_distance).max(1);
        let max_x = (pos.x + scanning_distance).min(world_size.x - 2);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let current_pos = Position { y, x };
                self.map[y][x].danger_score = self.calculate_danger_score(&current_pos);
            }
        }
    }

    fn calculate_danger_score(&self, pos: &Position) -> f32 {
        // TODO: Make sense out of the original idea and research algorithms
        let mut score = 0.0;
        let tile = &self.map[pos.y][pos.x];
        score += match tile.surface {
            Surface::Terrain(ref t) => MapCell::Terrain(t.clone()).positional_danger(),
            _ => 0.0,
        };

        score += (tile.times_hit as f32) * 0.1; // Sort of historical danger score

        for choke_point in &self.choke_points {
            let distance = pos.pythagorean_distance(choke_point);
            if distance < 3.0 {
                score += 0.5 / distance; // Higher when closer to choke points
            }
        }

        // TODO: How to cleanly keep track of the current turn?
        // let turns_since_hit = context.turn() - tile.last_damage_turn;
        // score *= f32::max(0.0, 1.0 - (turns_since_hit as f32 * 0.1));

        // TODO: Surroundings could have an impact

        score.min(1.0)
    }

    fn set_borders(&mut self, size: &WorldSize) {
        if size.x == 0 || size.y == 0 {
            return;
        }

        for x in 0..size.x {
            self.map[0][x].surface = Surface::Terrain(Terrain::Swamp);
            self.map[size.y - 1][x].surface = Surface::Terrain(Terrain::Swamp);
        }

        for y in 0..size.y {
            self.map[y][0].surface = Surface::Terrain(Terrain::Swamp);
            self.map[y][size.x - 1].surface = Surface::Terrain(Terrain::Swamp);
        }
    }

    fn count_nearby_unexplored(&self, pos: &Position, radius: usize) -> usize {
        let mut count = 0;
        let world_size = self.world_size();

        let min_y = pos.y.saturating_sub(radius).max(1);
        let max_y = (pos.y + radius).min(world_size.y - 2);
        let min_x = pos.x.saturating_sub(radius).max(1);
        let max_x = (pos.x + radius).min(world_size.x - 2);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if self.is_unexplored(&Position { x, y }) {
                    count += 1;
                }
            }
        }

        count
    }

    fn update_chokepoint_detection(&mut self, pos: &Position) {
        if !self.is_position_transitable(pos) {
            return;
        }

        let world_size = self.world_size();
        let mut exit_paths = 0;
        let mut transit_neighbors = Vec::new();

        for &orientation in &Orientation::all() {
            if let Some(adj_pos) = pos.follow(&orientation, &world_size) {
                if self.is_position_transitable(&adj_pos) {
                    exit_paths += 1;
                    transit_neighbors.push(adj_pos);
                }
            }
        }

        let is_chokepoint = exit_paths <= 2;
        if is_chokepoint {
            if !self.choke_points.contains(pos) {
                self.choke_points.push(pos.clone());

                // Increase danger score of this position and adjacent positions
                self.map[pos.y][pos.x].danger_score += 0.3;
                for neighbor in &transit_neighbors {
                    self.map[neighbor.y][neighbor.x].danger_score += 0.2;
                }
            }
        } else {
            // If this was previously marked as a chokepoint but isn't anymore, remove it
            if let Some(index) = self.choke_points.iter().position(|p| p.x == pos.x && p.y == pos.y) {
                self.choke_points.remove(index);
            }
        }
    }
}
