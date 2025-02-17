use std::{collections::HashMap, time::Duration};

use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

use crate::players::player::{
    Action, Aiming, Context, Direction, MapCell, Orientation, Player, PlayerDetails, PlayerId,
    Position, Rotation, ScanResult, ScanType, Terrain, TreeType, WorldSize, CARDINAL_SHOT_DISTANCE,
    INVALID_PLAYER, MAX_WORLD_SIZE, POSITIONAL_SHOT_DISTANCE, SCANNING_DISTANCE,
};

const MAX_FIELD_AREA_PERCENTAGE: f32 = 75.0;
const MIN_OBSTACLE_SIZE_PERCENTAGE: f32 = 0.5;
const MAX_OBSTACLE_SIZE_PERCENTAGE: f32 = 2.5;

const DAMAGE_COLLISION_WITH_FOREST: u8 = 25;
const DAMAGE_COLLISION_WITH_PLAYER: u8 = 10;
const DAMAGE_DIRECT_ORDNANCE_HIT: u8 = 75;
const DAMAGE_INDIRECT_ORDNANCE_HIT: u8 = 25;

const ANIMATE_SHELLS_AND_EXPLOSIONS: bool = true;

struct Tank {
    context: Context,
    player: Box<dyn Player>,
}

#[derive(PartialEq, Eq)]
enum ShellState {
    NotLaunched,
    Flying,
    Impact,
    Explosion,
    Exploded,
    Spent,
}

#[derive(PartialEq, Eq)]
struct Shell {
    current_pos: Position,
    fired_from: Position,
    aim_type: Aiming,
    state: ShellState,
}

impl Shell {
    fn new(aim_type: Aiming, fired_from: Position) -> Self {
        Self {
            current_pos: fired_from.clone(),
            fired_from,
            aim_type,
            state: ShellState::NotLaunched,
        }
    }

    fn possible_shot(&self) -> bool {
        match &self.aim_type {
            Aiming::Cardinal(_) => true,
            Aiming::Positional(pos) => self.fired_from.could_hit_positionally(pos),
        }
    }

    fn evolve(&mut self, world_size: &WorldSize) {
        match self.state {
            ShellState::NotLaunched | ShellState::Flying => {
                self.state = ShellState::Flying;
                self.current_pos = match &self.aim_type {
                    Aiming::Positional(pos) => pos.clone(),
                    Aiming::Cardinal(orientation) => {
                        if let Some(pos) = self.current_pos.follow(orientation, world_size) {
                            pos
                        } else {
                            self.current_pos.clone()
                        }
                    }
                };
            }
            ShellState::Impact => self.state = ShellState::Explosion,
            ShellState::Explosion => self.state = ShellState::Exploded,
            ShellState::Exploded => self.state = ShellState::Spent,
            ShellState::Spent => {}
        }
    }

    fn try_to_land(&mut self) -> bool {
        if self.state == ShellState::Flying {
            let landed = match &self.aim_type {
                Aiming::Cardinal(_) => {
                    let (dx, dy) = self.fired_from.manhattan_distance(&self.current_pos);
                    let distance = (dx.abs() + dy.abs()) as usize;
                    distance >= self.max_fly_distance()
                }
                Aiming::Positional(position) => *position == self.current_pos,
            };
            if landed {
                self.impact();
            }
            landed
        } else {
            false
        }
    }

    fn impact(&mut self) {
        if self.state == ShellState::Flying {
            self.state = ShellState::Impact;
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
    tanks: HashMap<PlayerDetails, Tank>,
    tick: u64,
}

impl World {
    pub fn new(tick: u64, size: WorldSize) -> Self {
        if size.x > MAX_WORLD_SIZE || size.y > MAX_WORLD_SIZE {
            panic!(
                "\nWorld size {size} is too big! Maximum accepted size for each dimension is {MAX_WORLD_SIZE}\n\n"
            );
        }

        let mut result = Self {
            map: Box::new([[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            rng: thread_rng(),
            size,
            tanks: HashMap::new(),
            tick,
        };
        result.generate_map_border();

        // Generate obstacles
        loop {
            result.generate_obstacle(MapCell::Terrain(Terrain::Forest(TreeType::Deciduous)));
            result.generate_obstacle(MapCell::Terrain(Terrain::Forest(TreeType::Evergreen)));
            result.generate_obstacle(MapCell::Terrain(Terrain::Lake));
            result.generate_obstacle(MapCell::Terrain(Terrain::Swamp));

            if result.get_unallocated_terrain_percentage() < MAX_FIELD_AREA_PERCENTAGE {
                break;
            }
        }

        // Fill the fields
        loop {
            if let Some(start) = result.get_unallocated_cell_position() {
                result.fill_with_field_cells(&start);
            }
            if result.get_unallocated_terrain_percentage() >= 5.0 {
                result.reset_field_cells();
            } else {
                break;
            }
        }

        result.fill_unallocated_holes();

        result
    }

    pub fn new_turn(&mut self) {
        let mut actions = Vec::new();

        for (player_details, tank) in self.tanks.iter_mut() {
            if tank.player.is_ready() && tank.context.health() > 0 {
                let action = tank.player.act(tank.context.clone());
                tank.context.set_previous_action(action.clone());
                tank.context.set_scanned_data(None);
                actions.push((*player_details, action));
            }
        }

        self.process_player_actions(actions)
    }

    pub fn spawn_player(&mut self, mut player: Box<dyn Player>, avatar: char) {
        if player.is_ready() && player.initialized() {
            println!("Player {} is ready for action!", player.name());

            let player_details = PlayerDetails::new(avatar, self.tanks.len() as PlayerId + 1);
            let context = Context::new(
                player_details,
                self.get_random_location(MapCell::Terrain(Terrain::Field)),
                self.size.clone(),
            );

            if self
                .try_set_player_on_cell(player_details, context.position())
                .is_some()
            {
                self.tanks.insert(player_details, Tank { context, player });
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
    fn process_player_actions(&mut self, actions: Vec<(PlayerDetails, Action)>) {
        let mut ordnance = Vec::new();
        let mut processed_players = Vec::new();

        for (player_details, action) in actions.iter() {
            if let Some(tank) = self.tanks.get(player_details) {
                let world_size = tank.context.world_size().clone();
                match action {
                    Action::Idle => {}
                    Action::Fire(aim) => {
                        ordnance.push(Shell::new(aim.clone(), tank.context.position().clone()))
                    }
                    Action::Move(direction) => {
                        let (from, to) = compute_route(
                            tank.context.position(),
                            tank.context.orientation(),
                            direction,
                            tank.context.world_size(),
                        );
                        self.move_player(*player_details, &from, &to);
                    }
                    Action::Rotate(rotation) => self.rotate_player(*player_details, rotation),
                    Action::Scan(scan_type) => {
                        self.scan_surroundings(*player_details, scan_type, &world_size);
                    }
                }
                processed_players.push(player_details);
            }
        }

        self.fly_shells(ordnance);
    }

    fn fly_shells(&mut self, ordnance: Vec<Shell>) {
        let max_iteration = CARDINAL_SHOT_DISTANCE.max(POSITIONAL_SHOT_DISTANCE) + 3;
        let mut possible_shots = Vec::new();

        for shell in ordnance {
            if shell.possible_shot() {
                possible_shots.push(shell);
            }
        }

        let mut iteration = 0;
        loop {
            for shell in possible_shots.iter_mut() {
                match shell.state {
                    ShellState::NotLaunched => {
                        shell.evolve(&self.size);
                        self.animate_shell(shell, false);
                    }
                    ShellState::Flying => {
                        self.animate_shell(shell, true);
                        shell.evolve(&self.size);

                        let landed = shell.try_to_land();
                        let collision = if let Some(player_details) =
                            self.get_player_at_position(&shell.current_pos)
                        {
                            self.is_player_alive(&player_details)
                        } else {
                            false
                        };
                        if landed || collision {
                            shell.impact();
                        } else {
                            self.animate_shell(shell, false);
                        }
                    }
                    ShellState::Impact => {
                        self.compute_shell_impact(&shell.current_pos);
                        self.animate_impact(shell);
                        shell.evolve(&self.size);
                    }
                    ShellState::Explosion => {
                        self.animate_impact(shell);
                        self.animate_explosion(shell);
                        shell.evolve(&self.size);
                    }
                    ShellState::Exploded => {
                        self.animate_explosion(shell);
                        shell.evolve(&self.size);
                    }
                    ShellState::Spent => continue,
                }
            }
            self.update_dead_players_on_map();

            iteration += 1;
            if iteration > max_iteration {
                break;
            }

            if ANIMATE_SHELLS_AND_EXPLOSIONS && !possible_shots.is_empty() {
                println!("{self}");
            }
            std::thread::sleep(Duration::from_millis(self.tick));
        }
    }

    fn animate_shell(&mut self, shell: &Shell, clear: bool) {
        let position = &shell.current_pos;
        let cell = self.cell_read(position);

        match cell {
            MapCell::Player(player_details, terrain) => {
                if !clear {
                    self.cell_write(position, MapCell::Shell(player_details, terrain));
                }
            }
            MapCell::Terrain(terrain) => {
                if !clear {
                    self.cell_write(position, MapCell::Shell(INVALID_PLAYER, terrain));
                }
            }
            MapCell::Shell(player_details, terrain) => {
                if clear {
                    if player_details == INVALID_PLAYER {
                        self.cell_write(position, MapCell::Terrain(terrain));
                    } else {
                        self.cell_write(position, MapCell::Player(player_details, terrain));
                    }
                }
            }
            _ => {}
        }
    }

    fn animate_impact(&mut self, shell: &Shell) {
        let position = &shell.current_pos;
        let cell = self.cell_read(&shell.current_pos);

        match shell.state {
            ShellState::Impact => match cell {
                MapCell::Player(player_details, terrain) => {
                    self.cell_write(position, MapCell::Explosion(player_details, terrain))
                }
                MapCell::Terrain(terrain) => {
                    self.cell_write(position, MapCell::Explosion(INVALID_PLAYER, terrain))
                }
                _ => {}
            },
            ShellState::Explosion => {
                if let MapCell::Explosion(player_details, terrain) = cell {
                    if player_details == INVALID_PLAYER {
                        self.cell_write(position, MapCell::Terrain(terrain));
                    } else {
                        self.cell_write(position, MapCell::Player(player_details, terrain));
                    }
                }
            }
            _ => {}
        }
    }

    fn animate_explosion(&mut self, shell: &Shell) {
        for position in self.get_adjacent_positions(&shell.current_pos) {
            let cell = self.cell_read(&position);

            match shell.state {
                ShellState::Explosion => match cell {
                    MapCell::Player(player_details, terrain) => {
                        self.cell_write(&position, MapCell::Explosion(player_details, terrain))
                    }
                    MapCell::Terrain(terrain) => {
                        self.cell_write(&position, MapCell::Explosion(INVALID_PLAYER, terrain))
                    }
                    _ => {}
                },
                ShellState::Exploded => {
                    if let MapCell::Explosion(player_details, terrain) = cell {
                        if player_details == INVALID_PLAYER {
                            self.cell_write(&position, MapCell::Terrain(terrain));
                        } else {
                            self.cell_write(&position, MapCell::Player(player_details, terrain));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn compute_shell_impact(&mut self, position: &Position) {
        let (directly_hit, indirectly_hit) = self.get_hit_players(position);

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

    fn update_dead_players_on_map(&mut self) {
        let mut dead_players = Vec::new();

        for tank in self.get_tanks() {
            if tank.player.is_ready() && tank.context.health() == 0 {
                if let MapCell::Player(_, terrain) = self.cell_read(tank.context.position()) {
                    let cell = MapCell::Player(*tank.context.player_details(), terrain);
                    dead_players.push((tank.context.position().clone(), cell));
                }
            }
        }

        for (position, cell) in dead_players {
            self.cell_write(&position, cell);
        }
    }

    fn move_player(&mut self, player_details: PlayerDetails, from: &Position, to: &Position) {
        let can_move = if let Some(tank) = self.tanks.get(&player_details) {
            tank.context.is_mobile()
        } else {
            false
        };

        if can_move {
            let to_cell = self.cell_read(to);
            match to_cell {
                MapCell::Player(other_id, _) => {
                    if let Some(tank) = self.tanks.get_mut(&player_details) {
                        tank.context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                    }
                    if let Some(tank) = self.tanks.get_mut(&other_id) {
                        tank.context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                    }
                }
                MapCell::Terrain(_) => {
                    if let Some(terrain) = self.try_set_player_on_cell(player_details, to) {
                        self.unset_player_from_cell(from);

                        if let Some(tank) = self.tanks.get_mut(&player_details) {
                            tank.context.relocate(to, terrain);
                        }
                    } else if let Some(tank) = self.tanks.get_mut(&player_details) {
                        tank.context.damage(DAMAGE_COLLISION_WITH_FOREST);
                    }
                }
                _ => {}
            }
        }
    }

    fn rotate_player(&mut self, player_details: PlayerDetails, rotation: &Rotation) {
        if let Some(tank) = self.tanks.get_mut(&player_details) {
            tank.context.rotate(rotation);
        }
    }

    fn scan_surroundings(
        &mut self,
        player_details: PlayerDetails,
        scan_type: &ScanType,
        world_size: &WorldSize,
    ) {
        if let Some(tank) = self.tanks.get(&player_details) {
            let position = tank.context.position().clone();
            let data = self.read_directional_map_area(scan_type, &position, world_size);

            if let Some(tank) = self.tanks.get_mut(&player_details) {
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
            if self.count_cells(&MapCell::Unallocated) > 0 {
                let mut path = Vec::new();
                let new_pos = if let Some(p) = old_pos.as_ref() {
                    self.get_adjacent_unallocated_location(p, obstacle, &mut path)
                } else {
                    Some(self.get_random_location(MapCell::Unallocated))
                };

                if let Some(pos) = new_pos {
                    self.cell_write(&pos, obstacle);
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

    fn is_player_alive(&self, player_details: &PlayerDetails) -> bool {
        for tank in self.get_tanks() {
            if *tank.context.player_details() == *player_details && tank.context.health() > 0 {
                return true;
            }
        }

        false
    }

    fn count_cells(&self, cell: &MapCell) -> usize {
        let mut free_count = 0;
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                if self.map[i][j] == *cell {
                    free_count += 1;
                }
            }
        }

        free_count
    }

    fn get_random_location(&mut self, map_cell: MapCell) -> Position {
        let mut bag = Vec::new();

        for i in 0..self.size.y {
            for j in 0..self.size.x {
                let position = Position { x: j, y: i };
                if self.cell_read(&position) == map_cell {
                    bag.push(position);
                }
            }
        }

        loop {
            let index = self.rng.gen_range(0..bag.len());
            if let Some(pos) = bag.get(index) {
                break pos.clone();
            }
        }
    }

    fn get_adjacent_unallocated_location(
        &mut self,
        position: &Position,
        obstacle_type: MapCell,
        walked_path: &mut Vec<Position>,
    ) -> Option<Position> {
        let mut orientations_bag = vec![
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ];
        orientations_bag.shuffle(&mut self.rng);

        let mut result = None;
        loop {
            if result.is_some() || orientations_bag.is_empty() {
                break;
            }

            if let Some(orientation) = orientations_bag.pop() {
                if let Some(next_pos) = position.follow(&orientation, &self.size) {
                    if walked_path.contains(&next_pos) {
                        break;
                    } else if self.is_location_unallocated(&next_pos) {
                        result = Some(next_pos);
                    } else if self.cell_read(&next_pos) == obstacle_type {
                        walked_path.push(next_pos.clone());
                        result = self.get_adjacent_unallocated_location(
                            &next_pos,
                            obstacle_type,
                            walked_path,
                        );
                    }
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
        let mut sub_map = Box::new([[MapCell::Unallocated; SCANNING_DISTANCE]; SCANNING_DISTANCE]);

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

    fn get_unallocated_terrain_percentage(&self) -> f32 {
        100.0f32 * self.count_cells(&MapCell::Unallocated) as f32
            / (self.size.x * self.size.y) as f32
    }

    fn is_location_unallocated(&self, position: &Position) -> bool {
        matches!(self.cell_read(position), MapCell::Unallocated)
    }

    fn get_player_at_position(&self, position: &Position) -> Option<PlayerDetails> {
        match self.cell_read(position) {
            MapCell::Player(player_details, _) => Some(player_details),
            _ => None,
        }
    }

    /// Returns the list of hit players at a position as a tuple of vectors, where the first element
    /// of the tuple is a vector containing the players directly hit, and the second element of the
    /// tuple is a vector containing the players indirectly hit.
    fn get_hit_players(&self, position: &Position) -> (Vec<PlayerDetails>, Vec<PlayerDetails>) {
        let mut direct_hit = Vec::new();
        let mut indirect_hit = Vec::new();

        if let Some(direct_hit_player) = self.get_player_at_position(position) {
            direct_hit.push(direct_hit_player);
        }

        let adjacent_positions = self.get_adjacent_positions(position);
        for adjacent in adjacent_positions {
            if let Some(player) = self.get_player_at_position(&adjacent) {
                indirect_hit.push(player);
            }
        }

        (direct_hit, indirect_hit)
    }

    fn get_adjacent_positions(&self, position: &Position) -> Vec<Position> {
        let mut adjacents = Vec::new();

        let mut orientation = Orientation::North;
        loop {
            if let Some(adjacent_position) = position.follow(&orientation, &self.size) {
                adjacents.push(adjacent_position);
            }
            orientation = orientation.rotated_clockwise();
            if orientation == Orientation::North {
                break;
            }
        }

        adjacents
    }

    fn unset_player_from_cell(&mut self, position: &Position) {
        let map_cell = self.cell_read(position);

        if let MapCell::Player(_, terrain) = map_cell {
            self.cell_write(position, MapCell::Terrain(terrain));
        }
    }

    fn try_set_player_on_cell(
        &mut self,
        player_details: PlayerDetails,
        position: &Position,
    ) -> Option<Terrain> {
        let mut result = None;
        let map_cell = self.cell_read(position);

        if let MapCell::Terrain(terrain) = map_cell {
            match terrain {
                Terrain::Field | Terrain::Lake | Terrain::Swamp => {
                    self.cell_write(position, MapCell::Player(player_details, terrain));
                    result = Some(terrain);
                }
                _ => {}
            }
        }

        result
    }

    fn get_unallocated_cell_position(&self) -> Option<Position> {
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                let position = Position { x: j, y: i };
                if self.cell_read(&position) == MapCell::Unallocated {
                    return Some(position);
                }
            }
        }

        None
    }

    fn fill_with_field_cells(&mut self, start: &Position) {
        let mut remaining = Vec::new();
        remaining.push(start.clone());

        loop {
            if remaining.is_empty() {
                break;
            }

            if let Some(next) = remaining.pop() {
                self.cell_write(&next, MapCell::Terrain(Terrain::Field));

                let mut neighbors = self
                    .get_adjacent_positions(&next)
                    .into_iter()
                    .filter(|x| self.cell_read(x) == MapCell::Unallocated)
                    .collect::<Vec<Position>>();
                remaining.append(&mut neighbors);
            }
        }
    }

    fn reset_field_cells(&mut self) {
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                let position = Position { x: j, y: i };
                if self.cell_read(&position) == MapCell::Terrain(Terrain::Field) {
                    self.cell_write(&position, MapCell::Terrain(Terrain::Lake));
                }
            }
        }
    }

    fn fill_unallocated_holes(&mut self) {
        loop {
            let changes = false;

            for i in 0..self.size.y {
                for j in 0..self.size.x {
                    let position = Position { x: j, y: i };

                    if self.cell_read(&position) == MapCell::Unallocated {
                        let cell_type = self.get_most_neihbouring_terrain(&position);
                        self.cell_write(&position, MapCell::Terrain(cell_type));
                    }
                }
            }

            if !changes {
                break;
            }
        }
    }

    fn get_most_neihbouring_terrain(&self, position: &Position) -> Terrain {
        let mut field_cnt = 0;
        let mut lake_cnt = 0;
        let mut tree_d_cnt = 0;
        let mut tree_e_cnt = 0;
        let mut swamp_cnt = 0;

        let neighbors = self.get_adjacent_positions(&position);

        for neighbor in neighbors {
            let cell = self.cell_read(&neighbor);
            match cell {
                MapCell::Terrain(terrain) => match terrain {
                    Terrain::Field => {
                        field_cnt += 1;
                        if field_cnt >= 4 {
                            return Terrain::Field;
                        }
                    }
                    Terrain::Lake => {
                        lake_cnt += 1;
                        if lake_cnt >= 4 {
                            return Terrain::Lake;
                        }
                    }
                    Terrain::Forest(TreeType::Deciduous) => {
                        tree_d_cnt += 1;
                        if tree_d_cnt >= 4 {
                            return Terrain::Forest(TreeType::Deciduous);
                        }
                    }
                    Terrain::Forest(TreeType::Evergreen) => {
                        tree_e_cnt += 1;
                        if tree_e_cnt >= 4 {
                            return Terrain::Forest(TreeType::Evergreen);
                        }
                    }
                    Terrain::Swamp => {
                        swamp_cnt += 1;
                        if swamp_cnt >= 4 {
                            return Terrain::Swamp;
                        }
                    }
                },
                _ => {}
            }
        }

        if tree_d_cnt > 0 {
            Terrain::Forest(TreeType::Deciduous)
        } else if tree_e_cnt > 0 {
            Terrain::Forest(TreeType::Evergreen)
        } else if lake_cnt > 0 {
            Terrain::Lake
        } else if swamp_cnt > 0 {
            Terrain::Swamp
        } else {
            Terrain::Field
        }
    }

    fn cell_read(&self, position: &Position) -> MapCell {
        self.map[position.y][position.x]
    }

    fn cell_write(&mut self, position: &Position, value: MapCell) {
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
        Direction::Forward => *orientation,
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
                        "{line}   {}: {}",
                        tank.context.player_details().avatar,
                        tank.player.name(),
                    );
                    let name = tank.player.name();
                    line = match name.chars().count() {
                        0..9 => format!("{line}\t\t\t"),
                        9..17 => format!("{line}\t\t"),
                        17.. => format!("{line}\t"),
                    };
                    line = format!(
                        "{line}({:03}%, {}, {}, {})",
                        tank.context.health(),
                        tank.context.position(),
                        tank.context.orientation(),
                        tank.context.previous_action()
                    )
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
            map: Box::new([[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            rng: thread_rng(),
            size: WorldSize {
                x: MINI_MAP_SIZE,
                y: MINI_MAP_SIZE,
            },
            tanks: HashMap::new(),
            tick: 100,
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

    fn fill_fields(world: &mut Box<World>) {
        world.fill_with_field_cells(&Position { x: 1, y: 1 });
    }

    #[test]
    fn test_animation_is_on() {
        assert!(ANIMATE_SHELLS_AND_EXPLOSIONS);
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
    fn test_get_cell_count() {
        let mut world = generate_mini_world();
        assert_eq!(
            MINI_MAP_SIZE * MINI_MAP_SIZE,
            world.count_cells(&MapCell::Unallocated)
        );

        world.generate_map_border();
        assert_eq!(
            MINI_MAP_SIZE * MINI_MAP_SIZE - 4 * (MINI_MAP_SIZE - 1),
            world.count_cells(&MapCell::Unallocated)
        );
    }

    #[test]
    fn test_get_field_terrain_percentage() {
        let mut world = generate_mini_world();
        assert_eq!(100.0, world.get_unallocated_terrain_percentage());

        world.generate_map_border();
        let percentage = world.count_cells(&MapCell::Unallocated) as f32
            / (MINI_MAP_SIZE * MINI_MAP_SIZE) as f32
            * 100.0;
        assert_eq!(percentage, world.get_unallocated_terrain_percentage());
    }

    #[test]
    fn test_random_field_location() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        for _ in 0..1000 {
            let location = world.get_random_location(MapCell::Unallocated);
            assert!(world.is_location_unallocated(&location));
        }
    }

    #[test]
    fn try_set_player_on_field_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);
        fill_fields(&mut world);

        let position = Position { x: 5, y: 2 };
        assert!(world.cell_read(&position) == MapCell::Terrain(Terrain::Field));

        let player_details = PlayerDetails::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_some());
    }

    #[test]
    fn try_set_player_on_forest_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 6, y: 3 };
        assert!(
            world.cell_read(&position) == MapCell::Terrain(Terrain::Forest(TreeType::Deciduous))
        );

        let player_details = PlayerDetails::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_none());
    }

    #[test]
    fn try_set_player_on_lake_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 3, y: 2 };
        assert!(world.cell_read(&position) == MapCell::Terrain(Terrain::Lake));

        let player_details = PlayerDetails::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_some());
    }

    #[test]
    fn try_set_player_on_swamp_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 5, y: 0 };
        assert!(world.cell_read(&position) == MapCell::Terrain(Terrain::Swamp));

        let player_details = PlayerDetails::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_some());
    }

    #[test]
    fn get_player_at_position() {
        let mut world = generate_mini_world();
        fill_fields(&mut world);

        let position = Position { x: 5, y: 2 };
        let player_details = PlayerDetails::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
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
        fill_fields(&mut world);

        let position_lower_player = Position { x: 5, y: 2 };
        let lower_player_details = PlayerDetails::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(lower_player_details, &position_lower_player);
        assert!(result.is_some());

        let position_upper_player = position_lower_player
            .follow(&Orientation::North, &world.size)
            .unwrap();
        let upper_player_details = PlayerDetails::new(DEFAULT_AVATAR, lower_player_details.id + 1);
        let result = world.try_set_player_on_cell(upper_player_details, &position_upper_player);
        assert!(result.is_some());

        println!("{world}");

        let firing_position_1 = position_upper_player.clone();
        let (direct_hit_players_1, indirect_hit_players_1) =
            world.get_hit_players(&firing_position_1);
        assert!(!direct_hit_players_1.is_empty());
        assert!(!indirect_hit_players_1.is_empty());
        assert_eq!(*direct_hit_players_1.first().unwrap(), upper_player_details);
        assert_eq!(
            *indirect_hit_players_1.first().unwrap(),
            lower_player_details
        );

        let firing_position_2 = position_lower_player.clone();
        let (direct_hit_players_2, indirect_hit_players_2) =
            world.get_hit_players(&firing_position_2);
        assert!(!direct_hit_players_2.is_empty());
        assert_eq!(*direct_hit_players_2.first().unwrap(), lower_player_details);
        assert!(!indirect_hit_players_2.is_empty());
        assert_eq!(
            *indirect_hit_players_2.first().unwrap(),
            upper_player_details
        );

        let firing_position_3 = position_lower_player
            .follow(&Orientation::South, &world.size)
            .unwrap();
        let (direct_hit_players_3, indirect_hit_players_3) =
            world.get_hit_players(&firing_position_3);
        assert!(direct_hit_players_3.is_empty());
        assert!(!indirect_hit_players_3.is_empty());
        assert_eq!(
            *indirect_hit_players_3.first().unwrap(),
            lower_player_details
        );

        let firing_position_4 = firing_position_3
            .follow(&Orientation::South, &world.size)
            .unwrap();
        let (direct_hit_players_4, indirect_hit_players_4) =
            world.get_hit_players(&firing_position_4);
        assert!(direct_hit_players_4.is_empty());
        assert!(indirect_hit_players_4.is_empty());
    }
}
