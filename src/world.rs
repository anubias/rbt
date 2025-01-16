use std::collections::HashMap;

use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::players::player::{
    Action, Context, Direction, MapCell, Orientation, Player, Position, Rotation, ScanResult,
    ScanType, WorldSize, MAX_WORLD_SIZE, SCANNING_DISTANCE,
};

const MAX_USABLE_SPACE_PERCENTAGE: f32 = 75.0;
const MIN_OBSTACLE_SIZE_PERCENTAGE: f32 = 0.5;
const MAX_OBSTACLE_SIZE_PERCENTAGE: f32 = 2.5;

const DAMAGE_COLLISION_WITH_MOUNTAIN: u8 = 25;
const DAMAGE_COLLISION_WITH_PLAYER: u8 = 10;

type UserId = u8;
type User = (Box<dyn Player>, Context);

pub struct World {
    rng: ThreadRng,
    size: WorldSize,
    players: HashMap<UserId, User>,
    map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
}

impl World {
    pub fn new(size: WorldSize) -> Self {
        if size.x > MAX_WORLD_SIZE || size.y > MAX_WORLD_SIZE {
            panic!(
                "\nWorld size {size} is too big! Maximum accepted size for each dimension is {MAX_WORLD_SIZE}\n\n"
            );
        }

        let mut result = Self {
            rng: thread_rng(),
            size: size.clone(),
            players: HashMap::new(),
            map: Box::new([[MapCell::Field; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
        };

        for i in 0..size.y {
            for j in 0..size.x {
                if i == 0 || j == 0 || i == size.y - 1 || j == size.x - 1 {
                    result.map[i][j] = MapCell::Swamp;
                }
            }
        }

        loop {
            result.generate_obstacle(MapCell::Mountain);
            result.generate_obstacle(MapCell::Lake);
            result.generate_obstacle(MapCell::Swamp);

            if result.get_usable_space_percentage() < MAX_USABLE_SPACE_PERCENTAGE {
                break;
            }
        }

        result
    }

    pub fn new_turn(&mut self) {
        let mut actions = Vec::new();

        for (id, (player, context)) in self.players.iter_mut() {
            if player.is_ready() && context.health() > 0 {
                let action = player.act(&context);
                context.set_scanned_data(None);
                actions.push((*id, action));
            }
        }

        self.process_player_actions(actions)
    }

    pub fn spawn_player(&mut self, player: Box<dyn Player>) {
        if !player.is_ready() {
            // won't spawn pussy-players
            return;
        }

        let player_id = (self.players.len() + 1) as u8;
        let context = Context::new(
            player_id,
            self.get_random_field_location(),
            MapCell::Field,
            self.size.clone(),
        );
        let previous = self.try_set_value_on_map(context.position(), MapCell::Player(player_id));
        match previous {
            MapCell::Field => {
                self.players.insert(player_id, (player, context));
            }
            _ => {}
        }
    }
}

// Private functions
impl World {
    fn process_player_actions(&mut self, actions: Vec<(u8, Action)>) {
        for (player_id, action) in actions.iter() {
            if let Some((_, context)) = self.players.get(player_id) {
                let world_size = context.world_size().clone();
                match action {
                    Action::Move(direction) => {
                        let (from, to) = Self::compute_route(
                            context.position(),
                            context.orientation(),
                            direction,
                            context.world_size(),
                        );
                        self.move_player(*player_id, &from, &to);
                    }
                    Action::Rotate(rotation) => {
                        self.rotate_player(*player_id, rotation);
                    }
                    Action::Scan(scan_type) => {
                        let position = context.position().clone();
                        self.scan_surroundings(*player_id, scan_type, &position, &world_size);
                    }
                    _ => {}
                }
            }
        }

        self.clear_dead_players_from_map(actions);
    }

    fn clear_dead_players_from_map(&mut self, actions: Vec<(u8, Action)>) {
        for (player_id, _) in actions {
            let mut pos_opt = None;
            let mut under = MapCell::Field;

            if let Some((_, context)) = self.players.get_mut(&player_id) {
                if context.health() == 0 {
                    pos_opt = Some(context.position().clone());
                    under = context.under().clone();
                }
            }

            if let Some(position) = pos_opt {
                if self.is_player_at_position(player_id, &position) {
                    self.try_set_value_on_map(&position, under);
                }
            }
        }
    }

    fn move_player(&mut self, player_id: u8, from: &Position, to: &Position) {
        let mut was_on = MapCell::default();
        let walk_on = self.try_set_value_on_map(&to, MapCell::Player(player_id));
        let mut successfully_moved = false;

        if let Some((_, context)) = self.players.get_mut(&player_id) {
            if context.is_mobile() {
                match walk_on {
                    MapCell::Field | MapCell::Lake | MapCell::Swamp => {
                        was_on = context.under().clone();
                        successfully_moved = context.relocate(to.clone(), walk_on);
                    }
                    MapCell::Mountain => {
                        context.damage(DAMAGE_COLLISION_WITH_MOUNTAIN);
                    }
                    MapCell::Player(other_player_id) => {
                        context.damage(DAMAGE_COLLISION_WITH_PLAYER);

                        // inflict damage to other player as well
                        if let Some((_, other_context)) = self.players.get_mut(&other_player_id) {
                            other_context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                        }
                    }
                    MapCell::Unknown => {}
                }
            }
        }

        if successfully_moved {
            self.try_set_value_on_map(&from, was_on);
        } else {
            self.try_set_value_on_map(&to, walk_on);
        }
    }

    fn rotate_player(&mut self, player_id: u8, rotation: &Rotation) {
        if let Some((_, context)) = self.players.get_mut(&player_id) {
            context.rotate(rotation);
        }
    }

    fn scan_surroundings(
        &mut self,
        player_id: u8,
        scan_type: &ScanType,
        position: &Position,
        world_size: &WorldSize,
    ) {
        let data = self.read_directional_map_area(scan_type, position, world_size);
        if let Some((_, context)) = self.players.get_mut(&player_id) {
            let scan_result = ScanResult {
                scan_type: scan_type.clone(),
                data,
            };
            context.set_scanned_data(Some(scan_result));
        }
    }

    fn compute_route(
        start_position: &Position,
        orientation: &Orientation,
        direction: &Direction,
        world_size: &WorldSize,
    ) -> (Position, Position) {
        let actual_orientation = match direction {
            Direction::Backward => orientation.opposite(),
            Direction::Forward => orientation.clone(),
        };

        let new_position = if let Some(pos) = start_position.follow(&actual_orientation, world_size)
        {
            pos
        } else {
            start_position.clone()
        };

        (start_position.clone(), new_position)
    }

    fn generate_obstacle(&mut self, obstacle: MapCell) {
        let map_size = (self.size.x * self.size.y) as f32;
        let range_min = (map_size * MIN_OBSTACLE_SIZE_PERCENTAGE / 100.0) as usize;
        let range_max = (map_size * MAX_OBSTACLE_SIZE_PERCENTAGE / 100.0) as usize;
        let obstacle_size = self.rng.gen_range(range_min..range_max);

        let mut old_pos: Option<Position> = None;
        for _ in 0..obstacle_size {
            if self.get_free_count() > 0 {
                let new_pos = if let Some(p) = old_pos.as_ref() {
                    self.get_adjacent_field_location(p, obstacle)
                } else {
                    Some(self.get_random_field_location())
                };

                if let Some(pos) = new_pos {
                    self.try_set_value_on_map(&pos, obstacle);
                    old_pos = Some(pos);
                }
            }
        }
    }

    fn get_free_count(&self) -> usize {
        let mut free_count = 0;
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                free_count += match self.map[i][j] {
                    MapCell::Field => 1,
                    _ => 0,
                }
            }
        }

        free_count
    }

    fn get_random_field_location(&mut self) -> Position {
        loop {
            let x = self.rng.gen_range(0..self.size.x);
            let y = self.rng.gen_range(0..self.size.y);
            let pos = Position { x, y };

            if self.is_location_free(&pos) {
                break pos;
            }
        }
    }

    fn get_adjacent_field_location(
        &mut self,
        position: &Position,
        obstacle_type: MapCell,
    ) -> Option<Position> {
        let mut orientations_bag = vec![
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ];

        let mut result = None;
        loop {
            if result.is_some() || orientations_bag.is_empty() {
                break;
            }

            let index = self.rng.gen_range(0..orientations_bag.len());
            let orientation = orientations_bag.remove(index);
            if let Some(next_pos) = position.follow(&orientation, &self.size) {
                if self.is_location_free(&next_pos) {
                    result = Some(next_pos);
                } else if self.get_value_from_map(&next_pos) == obstacle_type {
                    result = self.get_adjacent_field_location(&next_pos, obstacle_type)
                }
            }
        }

        result
    }

    fn read_directional_map_area(
        &self,
        scan_type: &ScanType,
        position: &Position,
        world_size: &WorldSize,
    ) -> Box<[[MapCell; SCANNING_DISTANCE]; SCANNING_DISTANCE]> {
        let mut sub_map = Box::new([[MapCell::Unknown; SCANNING_DISTANCE]; SCANNING_DISTANCE]);

        let (pos_x, pos_y, dist) = (
            position.x as isize,
            position.y as isize,
            SCANNING_DISTANCE as isize,
        );

        // remember that position (x,y) and arrays have the axis switched up
        let (start_j, start_i) = match scan_type {
            ScanType::Directional(orientation) => match orientation {
                Orientation::North => (pos_x - dist / 2, pos_y - dist + 1),
                Orientation::NorthEast => (pos_x, pos_y - dist + 1),
                Orientation::East => (pos_x, pos_y - dist / 2),
                Orientation::SouthEast => (pos_x, pos_y),
                Orientation::South => (pos_x - dist / 2, pos_y),
                Orientation::SouthWest => (pos_x - dist + 1, pos_y),
                Orientation::West => (pos_x - dist + 1, pos_y - dist / 2),
                Orientation::NorthWest => (pos_x - dist + 1, pos_y - dist + 1),
            },
            ScanType::Omni => (pos_x - dist / 2, pos_y - dist / 2),
        };

        for i in 0..sub_map.len() {
            let y = start_i + i as isize;
            if y >= 0 && y < world_size.y as isize {
                for j in 0..sub_map.len() {
                    let x = start_j + j as isize;
                    if x >= 0 && x < world_size.x as isize {
                        sub_map[i][j] = self.map[y as usize][x as usize];
                    }
                }
            }
        }

        sub_map
    }

    fn get_usable_space_percentage(&self) -> f32 {
        100.0f32 * self.get_free_count() as f32 / (self.size.x * self.size.y) as f32
    }

    fn is_location_free(&self, position: &Position) -> bool {
        match self.get_value_from_map(position) {
            MapCell::Field => true,
            _ => false,
        }
    }

    fn is_player_at_position(&self, player_id: u8, position: &Position) -> bool {
        match self.get_value_from_map(position) {
            MapCell::Player(id) => player_id == id,
            _ => false,
        }
    }

    fn try_set_value_on_map(&mut self, position: &Position, value: MapCell) -> MapCell {
        let walk_on = self.get_value_from_map(position);

        match walk_on {
            MapCell::Field | MapCell::Player(_) | MapCell::Swamp => {
                self.set_value_from_map(position, value);
            }
            _ => {}
        }

        walk_on
    }

    fn get_value_from_map(&self, position: &Position) -> MapCell {
        self.map[position.y][position.x]
    }

    fn set_value_from_map(&mut self, position: &Position, value: MapCell) {
        self.map[position.y][position.x] = value;
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.size.y {
            let mut line = String::new();
            for j in 0..self.size.x {
                line = format!("{line}{}", self.map[i][j]);
            }

            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_field_location() {
        let mut world = World::new(WorldSize { x: 100, y: 100 });

        for _ in 0..10_000 {
            let location = world.get_random_field_location();
            assert!(world.is_location_free(&location));
        }
    }
}
