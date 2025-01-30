use std::{collections::HashMap, time::Duration};

use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::players::player::{
    Action, Aiming, Context, Direction, MapCell, Orientation, Player, PlayerId, Position, Rotation,
    ScanResult, ScanType, Terrain, TreeType, WorldSize, CARDINAL_SHOT_DISTANCE, MAX_WORLD_SIZE,
    POSITIONAL_SHOT_DISTANCE, SCANNING_DISTANCE,
};

const MAX_FIELD_AREA_PERCENTAGE: f32 = 75.0;
const MIN_OBSTACLE_SIZE_PERCENTAGE: f32 = 0.5;
const MAX_OBSTACLE_SIZE_PERCENTAGE: f32 = 2.5;

const DAMAGE_COLLISION_WITH_FOREST: u8 = 25;
const DAMAGE_COLLISION_WITH_PLAYER: u8 = 10;
const DAMAGE_DIRECT_ORDNANCE_HIT: u8 = 75;
const DAMAGE_INDIRECT_ORDNANCE_HIT: u8 = 25;

struct Tank {
    context: Context,
    player: Box<dyn Player>,
}

#[derive(PartialEq, Eq)]
struct Ordnance {
    current_pos: Position,
    fired_from: Position,
    aim_type: Aiming,
}

impl Ordnance {
    fn new(aim_type: Aiming, fired_from: Position) -> Self {
        Self {
            current_pos: fired_from.clone(),
            fired_from,
            aim_type,
        }
    }

    /// Attempts to fly the ordnance one more "step" and returns whether the ordnance has just landed in this step.
    fn fly_one_step(&mut self, world_size: &WorldSize) -> bool {
        if !self.has_landed() {
            self.current_pos = match &self.aim_type {
                Aiming::Positional(pos) => pos.clone(),
                Aiming::Cardinal(orientation) => {
                    if let Some(pos) = self.current_pos.follow(&orientation, world_size) {
                        pos
                    } else {
                        self.current_pos.clone()
                    }
                }
            };
            self.has_landed()
        } else {
            false
        }
    }

    fn has_landed(&self) -> bool {
        match &self.aim_type {
            Aiming::Cardinal(_) => {
                let distance = self.fired_from.pythagorean_distance(&self.current_pos) as usize;
                distance > self.max_fly_distance()
            }
            Aiming::Positional(position) => *position == self.current_pos,
        }
    }

    fn max_fly_distance(&self) -> usize {
        match self.aim_type {
            Aiming::Cardinal(_) => CARDINAL_SHOT_DISTANCE,
            Aiming::Positional(_) => POSITIONAL_SHOT_DISTANCE,
        }
    }
}

pub struct World {
    map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
    rng: ThreadRng,
    size: WorldSize,
    tanks: HashMap<PlayerId, Tank>,
    turn_tick: u64,
}

impl World {
    pub fn new(turn_tick: u64, size: WorldSize) -> Self {
        if size.x > MAX_WORLD_SIZE || size.y > MAX_WORLD_SIZE {
            panic!(
                "\nWorld size {size} is too big! Maximum accepted size for each dimension is {MAX_WORLD_SIZE}\n\n"
            );
        }

        let mut result = Self {
            map: Box::new([[MapCell::Terrain(Terrain::Field); MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            rng: thread_rng(),
            size,
            tanks: HashMap::new(),
            turn_tick,
        };
        result.generate_map_border();

        loop {
            result.generate_obstacle(MapCell::Terrain(Terrain::Forest(TreeType::Deciduous)));
            result.generate_obstacle(MapCell::Terrain(Terrain::Forest(TreeType::Evergreen)));
            result.generate_obstacle(MapCell::Terrain(Terrain::Lake));
            result.generate_obstacle(MapCell::Terrain(Terrain::Swamp));

            if result.get_field_terrain_percentage() < MAX_FIELD_AREA_PERCENTAGE {
                break;
            }
        }

        result
    }

    pub fn new_turn(&mut self) {
        let mut actions = Vec::new();

        for (player_id, tank) in self.tanks.iter_mut() {
            if tank.player.is_ready() && tank.context.health() > 0 {
                let action = tank.player.act(tank.context.clone());
                tank.context.set_previous_action(action.clone());
                tank.context.set_scanned_data(None);
                actions.push((*player_id, action));
            }
        }

        self.process_player_actions(actions)
    }

    pub fn spawn_player(&mut self, mut player: Box<dyn Player>, avatar: char) {
        if player.is_ready() && player.initialized() {
            println!("Player {} is ready for action!", player.name());

            let player_id = PlayerId::new(avatar, self.tanks.len() + 1);
            let context = Context::new(
                player_id,
                self.get_random_field_location(),
                self.size.clone(),
            );

            if let Some(_) = self.try_set_player_on_cell(player_id, context.position()) {
                self.tanks.insert(player_id, Tank { context, player });
            }
        }
    }

    /// Indicates whether there are any ready players
    ///
    /// Reason is to allow testing whether all the players are not ready by default
    /// which eases development for each developer, since no other player's code is
    /// running by default.
    #[allow(dead_code)]
    pub fn has_ready_players(&self) -> bool {
        let mut result = false;

        for tank in self.tanks.values() {
            result |= tank.player.is_ready()
        }

        result
    }
}

// Private functions
impl World {
    fn process_player_actions(&mut self, actions: Vec<(PlayerId, Action)>) {
        let mut ordnance = Vec::new();
        let mut processed_players = Vec::new();

        for (player_id, action) in actions.iter() {
            if let Some(tank) = self.tanks.get(player_id) {
                let world_size = tank.context.world_size().clone();
                match action {
                    Action::Idle => {}
                    Action::Fire(aim) => {
                        ordnance.push(Ordnance::new(aim.clone(), tank.context.position().clone()))
                    }
                    Action::Move(direction) => {
                        let (from, to) = compute_route(
                            tank.context.position(),
                            tank.context.orientation(),
                            direction,
                            tank.context.world_size(),
                        );
                        self.move_player(*player_id, &from, &to);
                    }
                    Action::Rotate(rotation) => self.rotate_player(*player_id, rotation),
                    Action::Scan(scan_type) => {
                        self.scan_surroundings(*player_id, scan_type, &world_size);
                    }
                }
                processed_players.push(player_id.clone());
            }
        }

        self.fly_ordnances(ordnance);
        self.update_dead_players_on_map(processed_players);
    }

    fn fly_ordnances(&mut self, mut live_ordnance: Vec<Ordnance>) {
        let max_iteration = CARDINAL_SHOT_DISTANCE.max(POSITIONAL_SHOT_DISTANCE);
        let tick = self.turn_tick / max_iteration as u64;

        let mut iteration = 0;
        loop {
            for ordnance in live_ordnance.iter_mut() {
                let just_landed = ordnance.fly_one_step(&self.size);
                if just_landed {
                    let (directly_hit, indirectly_hit) =
                        self.get_hit_players(&ordnance.current_pos);

                    for player in directly_hit {
                        if let Some(tank) = self.tanks.get_mut(&player) {
                            tank.context.damage(DAMAGE_DIRECT_ORDNANCE_HIT);
                        }
                    }
                    for player in indirectly_hit {
                        if let Some(tank) = self.tanks.get_mut(&player) {
                            tank.context.damage(DAMAGE_INDIRECT_ORDNANCE_HIT);
                        }
                    }
                }
            }
            iteration += 1;
            if iteration > max_iteration {
                break;
            }

            std::thread::sleep(Duration::from_millis(tick));
        }
    }

    fn update_dead_players_on_map(&mut self, players: Vec<PlayerId>) {
        let mut dead_players = Vec::new();

        for player in players {
            if let Some(tank) = self.tanks.get(&player) {
                if tank.context.health() == 0 {
                    match self.get_value_from_map(tank.context.position()) {
                        MapCell::Player(_, terrain) => {
                            let cell = MapCell::Player(tank.context.player_id().clone(), terrain);
                            dead_players.push((tank.context.position().clone(), cell));
                        }
                        _ => {}
                    }
                }
            }
        }

        for (position, cell) in dead_players {
            self.set_value_on_map(&position, cell);
        }
    }

    fn move_player(&mut self, player_id: PlayerId, from: &Position, to: &Position) {
        if !self.is_player_at_position(player_id, from) {
            panic!("World map vs player context inconsistency detected while moving a player (player not at expected position)!");
        }

        let can_move = if let Some(tank) = self.tanks.get(&player_id) {
            tank.context.is_mobile()
        } else {
            false
        };

        if can_move {
            let to_cell = self.get_value_from_map(to);
            match to_cell {
                MapCell::Player(other_id, _) => {
                    if let Some(tank) = self.tanks.get_mut(&player_id) {
                        tank.context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                    }
                    if let Some(tank) = self.tanks.get_mut(&other_id) {
                        tank.context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                    }
                }
                MapCell::Terrain(_) => {
                    if let Some(terrain) = self.try_set_player_on_cell(player_id, to) {
                        self.unset_player_from_cell(from);

                        if let Some(tank) = self.tanks.get_mut(&player_id) {
                            tank.context.relocate(to, terrain);
                        }
                    } else {
                        if let Some(tank) = self.tanks.get_mut(&player_id) {
                            tank.context.damage(DAMAGE_COLLISION_WITH_FOREST);
                        }
                    }
                }
                MapCell::Unknown => {}
            }
        }
    }

    fn rotate_player(&mut self, player_id: PlayerId, rotation: &Rotation) {
        if let Some(tank) = self.tanks.get_mut(&player_id) {
            tank.context.rotate(rotation);
        }
    }

    fn scan_surroundings(
        &mut self,
        player_id: PlayerId,
        scan_type: &ScanType,
        world_size: &WorldSize,
    ) {
        if let Some(tank) = self.tanks.get(&player_id) {
            let position = tank.context.position().clone();
            let data = self.read_directional_map_area(scan_type, &position, world_size);

            if let Some(tank) = self.tanks.get_mut(&player_id) {
                let scan_result = ScanResult {
                    scan_type: scan_type.clone(),
                    data,
                };
                tank.context.set_scanned_data(Some(scan_result));
            }
        }
    }

    fn generate_obstacle(&mut self, obstacle: MapCell) {
        let map_size = (self.size.x * self.size.y) as f32;
        let range_min = (map_size * MIN_OBSTACLE_SIZE_PERCENTAGE / 100.0) as usize;
        let range_max = (map_size * MAX_OBSTACLE_SIZE_PERCENTAGE / 100.0) as usize;
        let obstacle_size = self.rng.gen_range(range_min..range_max);

        let mut old_pos: Option<Position> = None;
        for _ in 0..obstacle_size {
            if self.get_field_count() > 0 {
                let new_pos = if let Some(p) = old_pos.as_ref() {
                    self.get_adjacent_field_location(p, obstacle)
                } else {
                    Some(self.get_random_field_location())
                };

                if let Some(pos) = new_pos {
                    self.set_value_on_map(&pos, obstacle);
                    old_pos = Some(pos);
                }
            }
        }
    }

    fn get_tanks(&self) -> Vec<&Tank> {
        let mut result = Vec::new();

        for tank in self.tanks.values() {
            result.push(tank);
        }

        result
    }

    fn get_field_count(&self) -> usize {
        let mut free_count = 0;
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                free_count += match self.map[i][j] {
                    MapCell::Terrain(Terrain::Field) => 1,
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
            ScanType::Mono(orientation) => match orientation {
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

    fn get_field_terrain_percentage(&self) -> f32 {
        100.0f32 * self.get_field_count() as f32 / (self.size.x * self.size.y) as f32
    }

    fn is_location_free(&self, position: &Position) -> bool {
        match self.get_value_from_map(position) {
            MapCell::Terrain(Terrain::Field) => true,
            _ => false,
        }
    }

    fn is_player_at_position(&self, player_id: PlayerId, position: &Position) -> bool {
        match self.get_value_from_map(position) {
            MapCell::Player(id, _) => player_id == id,
            _ => false,
        }
    }

    fn get_player_at_position(&self, position: &Position) -> Option<PlayerId> {
        match self.get_value_from_map(position) {
            MapCell::Player(player_id, _) => Some(player_id),
            _ => None,
        }
    }

    /// Returns the list of hit players at a position as a tuple of vectors, where the first element
    /// of the tuple is a vector containing the players directly hit, and the second element of the
    /// tuple is a vector containing the players indirectly hit.
    fn get_hit_players(&self, position: &Position) -> (Vec<PlayerId>, Vec<PlayerId>) {
        let mut direct_hit = Vec::new();
        let mut indirect_hit = Vec::new();

        if let Some(direct_hit_player) = self.get_player_at_position(position) {
            direct_hit.push(direct_hit_player);
        }

        let north = Orientation::North;
        let mut orientation = north.clone();
        loop {
            if let Some(cell) = position.follow(&orientation, &self.size) {
                if let Some(player) = self.get_player_at_position(&cell) {
                    indirect_hit.push(player);
                }
            }
            orientation = orientation.rotated_clockwise();
            if orientation == north {
                break;
            }
        }

        (direct_hit, indirect_hit)
    }

    fn unset_player_from_cell(&mut self, position: &Position) {
        let map_cell = self.get_value_from_map(position);

        match map_cell {
            MapCell::Player(_, terrain) => {
                self.set_value_on_map(position, MapCell::Terrain(terrain))
            }
            _ => {}
        }
    }

    fn try_set_player_on_cell(
        &mut self,
        player_id: PlayerId,
        position: &Position,
    ) -> Option<Terrain> {
        let mut result = None;
        let map_cell = self.get_value_from_map(position);

        match map_cell {
            MapCell::Terrain(terrain) => match terrain {
                Terrain::Field | Terrain::Lake | Terrain::Swamp => {
                    self.set_value_on_map(position, MapCell::Player(player_id, terrain.clone()));
                    result = Some(terrain.clone());
                }
                _ => {}
            },
            _ => {}
        }

        result
    }

    fn get_value_from_map(&self, position: &Position) -> MapCell {
        self.map[position.y][position.x]
    }

    fn set_value_on_map(&mut self, position: &Position, value: MapCell) {
        self.map[position.y][position.x] = value;
    }

    fn generate_map_border(&mut self) {
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                if i == 0 || j == 0 || i == self.size.y - 1 || j == self.size.x - 1 {
                    self.map[i][j] = MapCell::Terrain(Terrain::Swamp);
                }
            }
        }
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

    let new_position = if let Some(pos) = start_position.follow(&actual_orientation, world_size) {
        pos
    } else {
        start_position.clone()
    };

    (start_position.clone(), new_position)
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let offset = 5;
        let tanks = self.get_tanks();

        for i in 0..self.size.y {
            let mut line = String::new();
            for j in 0..self.size.x {
                line = format!("{line}{}", self.map[i][j]);
            }

            if i == offset - 3 {
                line = format!("{line}   ACTIVE PLAYERS");
            } else if i == offset - 2 {
                line = format!("{line}   ==============");
            } else if i >= offset && i < offset + tanks.len() {
                if let Some(&tank) = tanks.get(i - offset) {
                    line = format!(
                        "{line}   {}: {}\t({}%, {})",
                        tank.context.player_id().avatar,
                        tank.player.name(),
                        tank.context.health(),
                        tank.context.previous_action()
                    );
                }
            }

            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_AVATAR;

    const MINI_MAP_SIZE: usize = 10;

    fn generate_mini_world() -> Box<World> {
        let world = World {
            map: Box::new([[MapCell::Terrain(Terrain::Field); MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            rng: thread_rng(),
            size: WorldSize {
                x: MINI_MAP_SIZE,
                y: MINI_MAP_SIZE,
            },
            tanks: HashMap::new(),
            turn_tick: 100,
        };

        Box::new(world)
    }

    fn populate_mini_world(world: &mut Box<World>) {
        world.generate_map_border();

        world.map[2][2] = MapCell::Terrain(Terrain::Lake);
        world.map[2][3] = MapCell::Terrain(Terrain::Lake);
        world.map[3][2] = MapCell::Terrain(Terrain::Lake);
        world.map[3][3] = MapCell::Terrain(Terrain::Lake);
        world.map[4][3] = MapCell::Terrain(Terrain::Lake);

        world.map[3][6] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
        world.map[3][7] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
        world.map[4][6] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
        world.map[4][7] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
        world.map[5][5] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
        world.map[5][6] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
        world.map[6][5] = MapCell::Terrain(Terrain::Forest(TreeType::Deciduous));
    }

    #[test]
    fn test_compute_route() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        // In the middle, facing northwest, going backwards
        let position = Position { x: 2, y: 3 };
        let orientation = Orientation::NorthEast;
        let direction = Direction::Backward;
        let world_size = WorldSize {
            x: MINI_MAP_SIZE,
            y: MINI_MAP_SIZE,
        };

        let (from, to) = compute_route(&position, &orientation, &direction, &world_size);
        assert_eq!((from.x, from.y), (position.x, position.y));
        assert_eq!((to.x, to.y), (1, 4));

        // Moving outside of the map, located on left edge, going west, route is "stationary"
        let position = Position { x: 0, y: 3 };
        let orientation = Orientation::West;
        let direction = Direction::Forward;
        let world_size = WorldSize {
            x: MINI_MAP_SIZE,
            y: MINI_MAP_SIZE,
        };

        let (from, to) = compute_route(&position, &orientation, &direction, &world_size);
        assert_eq!((from.x, from.y), (position.x, position.y));
        assert_eq!((from.x, from.y), (to.x, to.y));
    }

    #[test]
    fn test_generate_map_border() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        for i in 0..world.size.x {
            for j in 0..world.size.y {
                if i == 0 || i == world.size.x - 1 || j == 0 || j == world.size.y {
                    assert_eq!(MapCell::Terrain(Terrain::Swamp), world.map[i][j]);
                }
            }
        }
    }

    #[test]
    fn test_get_field_count() {
        let mut world = generate_mini_world();
        assert_eq!(MINI_MAP_SIZE * MINI_MAP_SIZE, world.get_field_count());

        world.generate_map_border();
        assert_eq!(
            MINI_MAP_SIZE * MINI_MAP_SIZE - 4 * (MINI_MAP_SIZE - 1),
            world.get_field_count()
        );
    }

    #[test]
    fn test_get_field_terrain_percentage() {
        let mut world = generate_mini_world();
        assert_eq!(100.0, world.get_field_terrain_percentage());

        world.generate_map_border();
        let percentage =
            world.get_field_count() as f32 / (MINI_MAP_SIZE * MINI_MAP_SIZE) as f32 * 100.0;
        assert_eq!(percentage, world.get_field_terrain_percentage());
    }

    #[test]
    fn test_random_field_location() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        for _ in 0..1000 {
            let location = world.get_random_field_location();
            assert!(world.is_location_free(&location));
        }
    }

    #[test]
    fn try_set_player_on_field_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 5, y: 2 };
        assert!(world.get_value_from_map(&position) == MapCell::Terrain(Terrain::Field));

        let player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_id, &position);
        assert!(result.is_some());
    }

    #[test]
    fn try_set_player_on_forest_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 6, y: 3 };
        assert!(
            world.get_value_from_map(&position)
                == MapCell::Terrain(Terrain::Forest(TreeType::Deciduous))
        );

        let player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_id, &position);
        assert!(result.is_none());
    }

    #[test]
    fn try_set_player_on_lake_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 3, y: 2 };
        assert!(world.get_value_from_map(&position) == MapCell::Terrain(Terrain::Lake));

        let player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_id, &position);
        assert!(result.is_some());
    }

    #[test]
    fn try_set_player_on_swamp_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 5, y: 0 };
        assert!(world.get_value_from_map(&position) == MapCell::Terrain(Terrain::Swamp));

        let player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_id, &position);
        assert!(result.is_some());
    }

    #[test]
    fn is_player_at_position() {
        let mut world = generate_mini_world();

        let position = Position { x: 5, y: 2 };
        let player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_id, &position);
        assert!(result.is_some());

        let other_position = Position {
            x: position.x + 1,
            y: position.y,
        };
        assert!(world.is_player_at_position(player_id, &position));
        assert!(!world.is_player_at_position(player_id, &other_position));
    }

    #[test]
    fn get_player_at_position() {
        let mut world = generate_mini_world();

        let position = Position { x: 5, y: 2 };
        let player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_id, &position);
        assert!(result.is_some());

        let other_position = Position {
            x: position.x + 1,
            y: position.y,
        };
        assert!(world.get_player_at_position(&position).is_some());
        assert!(world.get_player_at_position(&other_position).is_none());
    }

    #[test]
    fn get_hit_players() {
        let mut world = generate_mini_world();

        let position_lower_player = Position { x: 5, y: 2 };
        let lower_player_id = PlayerId::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(lower_player_id, &position_lower_player);
        assert!(result.is_some());

        let position_upper_player = position_lower_player
            .follow(&Orientation::North, &world.size)
            .unwrap();
        let upper_player_id = PlayerId::new(DEFAULT_AVATAR, lower_player_id.id + 1);
        let result = world.try_set_player_on_cell(upper_player_id, &position_upper_player);
        assert!(result.is_some());

        println!("{world}");

        let firing_position_1 = position_upper_player.clone();
        let (direct_hit_players_1, indirect_hit_players_1) =
            world.get_hit_players(&firing_position_1);
        assert!(!direct_hit_players_1.is_empty());
        assert!(!indirect_hit_players_1.is_empty());
        assert_eq!(*direct_hit_players_1.first().unwrap(), upper_player_id);
        assert_eq!(*indirect_hit_players_1.first().unwrap(), lower_player_id);

        let firing_position_2 = position_lower_player.clone();
        let (direct_hit_players_2, indirect_hit_players_2) =
            world.get_hit_players(&firing_position_2);
        assert!(!direct_hit_players_2.is_empty());
        assert_eq!(*direct_hit_players_2.first().unwrap(), lower_player_id);
        assert!(!indirect_hit_players_2.is_empty());
        assert_eq!(*indirect_hit_players_2.first().unwrap(), upper_player_id);

        let firing_position_3 = position_lower_player
            .follow(&Orientation::South, &world.size)
            .unwrap();
        let (direct_hit_players_3, indirect_hit_players_3) =
            world.get_hit_players(&firing_position_3);
        assert!(direct_hit_players_3.is_empty());
        assert!(!indirect_hit_players_3.is_empty());
        assert_eq!(*indirect_hit_players_3.first().unwrap(), lower_player_id);

        let firing_position_4 = firing_position_3
            .follow(&Orientation::South, &world.size)
            .unwrap();
        let (direct_hit_players_4, indirect_hit_players_4) =
            world.get_hit_players(&firing_position_4);
        assert!(direct_hit_players_4.is_empty());
        assert!(indirect_hit_players_4.is_empty());
    }
}
