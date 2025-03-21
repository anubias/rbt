use std::{collections::HashMap, time::Duration};

use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

use crate::{
    api::{
        action::Action,
        direction::Direction,
        map_cell::{MapCell, Terrain, TreeType},
        orientation::Orientation,
        player::{Details, Player, PlayerId, INVALID_PLAYER},
        position::{Position, CARDINAL_SHOT_DISTANCE, POSITIONAL_SHOT_DISTANCE, SCANNING_DISTANCE},
        rotation::Rotation,
        scan::{ScanResult, ScanType},
        world_size::{WorldSize, MAX_WORLD_SIZE},
    },
    engine::{
        context::Context,
        shell::{Shell, ShellState},
        tank::Tank,
    },
    terminal::Terminal,
};

const SEA_WORLD_PERCENTAGE: f32 = 20.0;
const MAX_FIELD_AREA_PERCENTAGE: f32 = 75.0;
const MIN_OBSTACLE_SIZE_PERCENTAGE: f32 = 0.5;
const MAX_OBSTACLE_SIZE_PERCENTAGE: f32 = 2.5;

struct ScanRequest {
    requester_id: PlayerId,
    scan_type: ScanType,
}

pub struct World {
    animation: bool,
    map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
    max_turns: usize,
    rng: ThreadRng,
    size: WorldSize,
    tanks: HashMap<PlayerId, Tank>,
    tick: u64,
    turn_number: usize,
}

impl World {
    pub fn new(animation: bool, tick: u64, size: WorldSize) -> Self {
        if size.x > MAX_WORLD_SIZE || size.y > MAX_WORLD_SIZE {
            panic!(
                "\nWorld size {size} is too big! Maximum accepted size for each dimension is {MAX_WORLD_SIZE}\n\n"
            );
        }

        loop {
            let result = World::generate_world(animation, tick, size.clone());

            if !result.sea_world() {
                break result;
            } else {
                println!("Rejecting sea world...");
            }
        }
    }

    pub fn update_animation(&mut self, animation: bool) {
        self.animation = animation;
    }

    pub fn update_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    pub fn new_turn(&mut self, terminal: &mut Terminal) {
        let mut actions = Vec::new();

        self.turn_number += 1;
        for (player_details, tank) in self.tanks.iter_mut() {
            if tank.player().is_ready() && tank.context().health() > 0 {
                let context = tank.context().clone();
                let action = tank.player_mut().act(context.into());

                tank.context_mut().set_previous_action(action.clone());
                tank.context_mut().set_scanned_data(None);
                tank.context_mut().set_turn(self.turn_number);

                actions.push((*player_details, action));
            }
        }

        terminal.move_caret_to_origin();
        terminal.println(&self);

        self.process_player_actions(terminal, actions)
    }

    pub fn spawn_player(&mut self, mut player: Box<dyn Player>, avatar: char) {
        let random = self.get_random_location(MapCell::Terrain(Terrain::Field));

        if let Some(position) = random {
            if player.is_ready() && player.initialized() {
                let player_details = Details::new(avatar, self.tanks.len() as PlayerId + 1);
                let context =
                    Context::new(player_details, position, self.max_turns, self.size.clone());

                if self
                    .try_set_player_on_cell(player_details, context.position())
                    .is_some()
                {
                    self.tanks
                        .insert(player_details.id, Tank::new(player, context));
                }
            }
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.count_live_players() <= 1 || self.turn_number >= self.max_turns
    }

    pub fn reward_survivors(&mut self) {
        self.tanks
            .iter_mut()
            .map(|(_, t)| t.survivor_bonus())
            .count();
    }

    pub fn get_ready_players(&self) -> Vec<&Tank> {
        self.tanks
            .iter()
            .filter_map(|(_, tank)| tank.player().is_ready().then(|| tank))
            .collect()
    }
}

// Private functions
impl World {
    fn process_player_actions(
        &mut self,
        terminal: &mut Terminal,
        actions: Vec<(PlayerId, Action)>,
    ) {
        let mut shot_queue = Vec::new();
        let mut scan_queue = Vec::new();

        for (player_id, action) in actions.iter() {
            if let Some(tank) = self.tanks.get(player_id) {
                let tank_position = tank.context().position().clone();

                match action {
                    Action::Idle => {}
                    Action::Fire(aim) => shot_queue.push(Shell::new(aim.clone(), tank_position)),
                    Action::Move(direction) => {
                        let (from, to) = self.compute_step(
                            &tank_position,
                            &tank.context().player_details().orientation,
                            direction,
                        );
                        self.move_player(*player_id, &from, &to);
                    }
                    Action::Rotate(rotation) => self.rotate_player(*player_id, rotation),
                    Action::Scan(scan_type) => {
                        scan_queue.push(ScanRequest {
                            requester_id: *player_id,
                            scan_type: scan_type.clone(),
                        });
                    }
                }
            }
        }

        self.process_shots(terminal, shot_queue);
        self.update_players_on_world_map(); // we need to update the world map before processing scans
        self.process_scans(scan_queue);
    }

    fn process_scans(&mut self, scan_queue: Vec<ScanRequest>) {
        let word_size = self.size.clone();

        for scan in scan_queue {
            self.scan_surroundings(scan, &word_size);
        }
    }

    fn update_players_on_world_map(&mut self) {
        for i in 0..self.size.y {
            for j in 0..self.size.x {
                if let MapCell::Player(player_details, terrain) = self.map[i][j] {
                    if let Some(tank) = self.tanks.get(&player_details.id) {
                        self.map[i][j] = MapCell::Player(*tank.context().player_details(), terrain);
                    }
                }
            }
        }
    }

    fn process_shots(&mut self, terminal: &mut Terminal, shot_queue: Vec<Shell>) {
        let max_iteration = CARDINAL_SHOT_DISTANCE.max(POSITIONAL_SHOT_DISTANCE) + 3;
        let mut possible_shots = Vec::new();

        for shell in shot_queue {
            if shell.possible_shot() {
                possible_shots.push(shell);
            }
        }

        let mut iteration = 0;
        loop {
            for shell in possible_shots.iter_mut() {
                match shell.state() {
                    ShellState::NotLaunched => {
                        shell.evolve(&self.size);
                    }
                    ShellState::Flying => {
                        self.animate_shell_movement(shell, true);
                        shell.evolve(&self.size);

                        let landed = shell.try_to_land();
                        let collision = if let Some(position) = shell.pos() {
                            if let Some(player_details) = self.get_player_at_position(&position) {
                                self.is_player_alive(&player_details)
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if landed || collision {
                            shell.impact();
                        } else {
                            self.animate_shell_movement(shell, false);
                        }
                    }
                    ShellState::Impact => {
                        self.animate_direct_shell_explosion(shell);
                        shell.evolve(&self.size);
                    }
                    ShellState::Explosion => {
                        self.animate_direct_shell_explosion(shell);
                        self.animate_indirect_shell_explosion(shell);
                        shell.evolve(&self.size);
                    }
                    ShellState::Exploded => {
                        self.animate_indirect_shell_explosion(shell);
                        self.compute_shell_damage(shell);
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

            if self.animation && !possible_shots.is_empty() {
                terminal.move_caret_to_origin();
                terminal.println(self.to_string());
            }
            std::thread::sleep(Duration::from_millis(self.tick));
        }
    }

    fn animate_shell_movement(&mut self, shell: &Shell, clear: bool) {
        if let Some(position) = shell.pos() {
            let cell = self.cell_read(&position);

            match cell {
                MapCell::Player(player_details, terrain) => {
                    if !clear {
                        self.cell_write(&position, MapCell::Shell(player_details, terrain));
                    }
                }
                MapCell::Terrain(terrain) => {
                    if !clear {
                        self.cell_write(&position, MapCell::Shell(INVALID_PLAYER, terrain));
                    }
                }
                MapCell::Shell(player_details, terrain) => {
                    if clear {
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

    fn animate_direct_shell_explosion(&mut self, shell: &Shell) {
        if let Some(position) = shell.pos() {
            self.animate_cell_explosion(&position, shell.state() == ShellState::Impact);
        }
    }

    fn animate_indirect_shell_explosion(&mut self, shell: &Shell) {
        if let Some(position) = shell.pos() {
            for adjacent_pos in position.list_adjacent_positions(&self.size) {
                self.animate_cell_explosion(&adjacent_pos, shell.state() == ShellState::Explosion);
            }
        }
    }

    fn animate_cell_explosion(&mut self, position: &Position, exploding: bool) {
        let cell = self.cell_read(position);

        if exploding {
            match cell {
                MapCell::Player(player_details, terrain) => {
                    self.cell_write(position, MapCell::Explosion(player_details, terrain))
                }
                MapCell::Terrain(terrain) => {
                    self.cell_write(position, MapCell::Explosion(INVALID_PLAYER, terrain))
                }
                _ => {}
            }
        } else {
            if let MapCell::Explosion(player_details, terrain) = cell {
                if player_details == INVALID_PLAYER {
                    self.cell_write(position, MapCell::Terrain(terrain));
                } else {
                    self.cell_write(position, MapCell::Player(player_details, terrain));
                }
            }
        }
    }

    fn compute_shell_damage(&mut self, shell: &Shell) {
        if let Some(at) = shell.pos() {
            let (directly_hit, indirectly_hit) = self.get_hit_players(&at);

            if let Some(shooter_details) = self.get_player_at_position(&shell.fired_from()) {
                let mut reward = 0;

                for player_id in directly_hit {
                    if let Some(tank) = self.tanks.get_mut(&player_id) {
                        reward += tank.context_mut().damage_direct_hit(shooter_details.id);
                    }
                }

                for player_id in indirectly_hit {
                    if let Some(tank) = self.tanks.get_mut(&player_id) {
                        reward += tank.context_mut().damage_indirect_hit(shooter_details.id);
                    }
                }

                if let Some(shooter) = self.tanks.get_mut(&shooter_details.id) {
                    shooter.context_mut().reward_hits(reward);
                }
            }
        }
    }

    fn update_dead_players_on_map(&mut self) {
        let mut dead_players = Vec::new();

        for tank in self.get_tanks() {
            if tank.player().is_ready() && tank.context().health() == 0 {
                if let MapCell::Player(_, terrain) = self.cell_read(tank.context().position()) {
                    let cell = MapCell::Player(*tank.context().player_details(), terrain);
                    dead_players.push((tank.context().position().clone(), cell));
                }
            }
        }

        for (position, cell) in dead_players {
            self.cell_write(&position, cell);
        }
    }

    fn move_player(&mut self, player_id: PlayerId, from: &Position, to: &Position) {
        let to_cell = self.cell_read(to);

        let tank_context = if let Some(tank) = self.tanks.get_mut(&player_id) {
            Some(tank.context().clone())
        } else {
            None
        };

        if let Some(mut temp_context) = tank_context {
            if temp_context.is_mobile() {
                match to_cell {
                    MapCell::Player(other_details, _) => {
                        if let Some(other_tank) = self.tanks.get_mut(&other_details.id) {
                            temp_context.damage_collision_player(other_tank.context_mut());
                        }
                    }
                    MapCell::Terrain(_) => {
                        if let Some(terrain) =
                            self.try_set_player_on_cell(*temp_context.player_details(), to)
                        {
                            self.unset_player_from_cell(from);

                            temp_context.relocate(to, terrain);
                        } else {
                            // only terrain we cannot move into is `Terrain::Forest`
                            temp_context.damage_collision_forest();
                        }
                    }
                    _ => {}
                }

                // Update new context
                if let Some(tank) = self.tanks.get_mut(&player_id) {
                    tank.set_context(temp_context);
                }
            }
        }
    }

    fn rotate_player(&mut self, player_id: PlayerId, rotation: &Rotation) {
        if let Some(tank) = self.tanks.get_mut(&player_id) {
            tank.context_mut().rotate(rotation);
        }
    }

    fn scan_surroundings(&mut self, scan_request: ScanRequest, world_size: &WorldSize) {
        if let Some(tank) = self.tanks.get(&scan_request.requester_id) {
            let position = tank.context().position().clone();
            let data =
                self.read_directional_map_area(&scan_request.scan_type, &position, world_size);

            if let Some(tank) = self.tanks.get_mut(&scan_request.requester_id) {
                let scan_result = ScanResult {
                    scan_type: scan_request.scan_type.clone(),
                    data,
                };
                tank.context_mut().set_scanned_data(Some(scan_result));
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
                    if let Some(pos) = self.get_random_location(MapCell::Unallocated) {
                        Some(pos)
                    } else {
                        None
                    }
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

        result.sort_by(|&a, &b| {
            a.player()
                .name()
                .partial_cmp(&b.player().name())
                .expect("Unable to sort player names!")
        });
        result
    }

    fn is_player_alive(&self, player_details: &Details) -> bool {
        for tank in self.get_tanks() {
            if *tank.context().player_details() == *player_details && tank.context().health() > 0 {
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

    fn count_live_players(&self) -> usize {
        self.tanks
            .iter()
            .filter(|&t| t.1.context().player_details().alive)
            .count()
    }

    fn get_random_location(&mut self, map_cell: MapCell) -> Option<Position> {
        let mut bag = Vec::new();

        for i in 0..self.size.y {
            for j in 0..self.size.x {
                let position = Position { x: j, y: i };
                if self.cell_read(&position) == map_cell {
                    bag.push(position);
                }
            }
        }

        if bag.is_empty() {
            None
        } else {
            let index = self.rng.gen_range(0..bag.len());
            if let Some(pos) = bag.get(index) {
                Some(pos.clone())
            } else {
                None
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

    fn compute_terrain_percentage(&self, cell_count: usize) -> f32 {
        100.0f32 * cell_count as f32 / (self.size.x * self.size.y) as f32
    }

    fn is_location_unallocated(&self, position: &Position) -> bool {
        matches!(self.cell_read(position), MapCell::Unallocated)
    }

    fn get_player_at_position(&self, position: &Position) -> Option<Details> {
        match self.cell_read(position) {
            MapCell::Player(player_details, _) => Some(player_details),
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
            direct_hit.push(direct_hit_player.id);
        }

        let adjacent_positions = position.list_adjacent_positions(&self.size);
        for adjacent in adjacent_positions {
            if let Some(player) = self.get_player_at_position(&adjacent) {
                indirect_hit.push(player.id);
            }
        }

        (direct_hit, indirect_hit)
    }

    fn unset_player_from_cell(&mut self, position: &Position) {
        let map_cell = self.cell_read(position);

        if let MapCell::Player(_, terrain) = map_cell {
            self.cell_write(position, MapCell::Terrain(terrain));
        }
    }

    fn try_set_player_on_cell(
        &mut self,
        player_details: Details,
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

    fn fill_with_field_cells(&mut self, start: &Position) {
        let mut remaining = Vec::new();
        remaining.push(start.clone());

        loop {
            if remaining.is_empty() {
                break;
            }

            if let Some(next) = remaining.pop() {
                self.cell_write(&next, MapCell::Terrain(Terrain::Field));

                let mut neighbors = next
                    .list_adjacent_positions(&self.size)
                    .into_iter()
                    .filter(|x| self.cell_read(x) == MapCell::Unallocated)
                    .collect::<Vec<Position>>();
                remaining.append(&mut neighbors);
            }
        }
    }

    fn flood_fields(&mut self) {
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

        let neighbors = position.list_adjacent_positions(&self.size);

        for neighbor in neighbors {
            if let MapCell::Terrain(terrain) = self.cell_read(&neighbor) {
                match terrain {
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
                }
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

    fn compute_step(
        &self,
        start_position: &Position,
        orientation: &Orientation,
        direction: &Direction,
    ) -> (Position, Position) {
        let actual_orientation = match direction {
            Direction::Backward => orientation.opposite(),
            Direction::Forward => *orientation,
        };

        let new_position = if let Some(pos) = start_position.follow(&actual_orientation, &self.size)
        {
            pos
        } else {
            start_position.clone()
        };

        (start_position.clone(), new_position)
    }

    fn sea_world(&self) -> bool {
        let lakes = self.count_cells(&MapCell::Terrain(Terrain::Lake));
        let percentage = self.compute_terrain_percentage(lakes);

        percentage >= SEA_WORLD_PERCENTAGE
    }

    fn generate_world(animation: bool, tick: u64, size: WorldSize) -> Self {
        let mut result = Self {
            animation,
            map: Box::new([[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            max_turns: compute_game_turns(&size),
            rng: thread_rng(),
            size,
            tanks: HashMap::new(),
            tick,
            turn_number: 0,
        };
        result.generate_map_border();

        // Generate obstacles
        loop {
            result.generate_obstacle(MapCell::Terrain(Terrain::Forest(TreeType::Deciduous)));
            result.generate_obstacle(MapCell::Terrain(Terrain::Forest(TreeType::Evergreen)));
            result.generate_obstacle(MapCell::Terrain(Terrain::Lake));
            result.generate_obstacle(MapCell::Terrain(Terrain::Lake));
            result.generate_obstacle(MapCell::Terrain(Terrain::Swamp));

            let cell_count = result.count_cells(&MapCell::Unallocated);
            if result.compute_terrain_percentage(cell_count) < MAX_FIELD_AREA_PERCENTAGE {
                break;
            }
        }

        // Fill the fields
        loop {
            if let Some(start) = result.get_random_location(MapCell::Unallocated) {
                result.fill_with_field_cells(&start);
            }
            let cell_count = result.count_cells(&MapCell::Unallocated);
            if result.compute_terrain_percentage(cell_count) >= 5.0 {
                result.flood_fields();
            } else {
                break;
            }
        }

        result.fill_unallocated_holes();

        result
    }
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PRINT_OFFSET: usize = 1;
        const HELP_SECTION_OFFSET: usize = PRINT_OFFSET;
        const STATS_SECTION_OFFSET: usize = HELP_SECTION_OFFSET + 10;
        const PLAYERS_SECTION_OFFSET: usize = STATS_SECTION_OFFSET + 8;
        const PLAYERS_LIST_OFSSET: usize = PLAYERS_SECTION_OFFSET + 2;

        let tanks = self.get_tanks();

        for i in 0..self.size.y {
            let mut line = String::new();
            for j in 0..self.size.x {
                line = format!("{line}{}", self.map[i][j]);
            }

            if i == HELP_SECTION_OFFSET {
                line = format!("{line}   [USER KEYS]");
            } else if i == HELP_SECTION_OFFSET + 1 {
                line = format!("{line}   ===========");
            } else if i == HELP_SECTION_OFFSET + 2 {
                line = format!("{line}   Esc  - Interrupt game");
            } else if i == HELP_SECTION_OFFSET + 3 {
                line = format!("{line}   A    - Toggle shell animation");
            } else if i == HELP_SECTION_OFFSET + 4 {
                line = format!("{line}   N    - Run next turn and then pause");
            } else if i == HELP_SECTION_OFFSET + 5 {
                line = format!("{line}   P    - Toggle pause game");
            } else if i == HELP_SECTION_OFFSET + 6 {
                line = format!("{line}   Up   - Increase tick by 1 millisecond");
            } else if i == HELP_SECTION_OFFSET + 7 {
                line = format!("{line}   Down - Decrease tick by 1 millisecond");
            } else if i == STATS_SECTION_OFFSET {
                line = format!("{line}   [GAME STATS]");
            } else if i == STATS_SECTION_OFFSET + 1 {
                line = format!("{line}   ============");
            } else if i == STATS_SECTION_OFFSET + 2 {
                let a = if self.animation { "on" } else { "off" };
                line = format!("{line}   Animation:\t\t{}   ", a);
            } else if i == STATS_SECTION_OFFSET + 3 {
                line = format!("{line}   Players alive:\t{}   ", self.count_live_players());
            } else if i == STATS_SECTION_OFFSET + 4 {
                line = format!("{line}   Tick ms:\t\t{}   ", self.tick);
            } else if i == STATS_SECTION_OFFSET + 5 {
                line = format!(
                    "{line}   Turn:\t\t{} / {}",
                    self.turn_number, self.max_turns,
                );
            } else if i == PLAYERS_SECTION_OFFSET {
                line = format!("{line}   [ACTIVE PLAYERS]");
            } else if i == PLAYERS_SECTION_OFFSET + 1 {
                line = format!("{line}   ================");
            } else if i >= PLAYERS_LIST_OFSSET && i < PLAYERS_LIST_OFSSET + tanks.len() {
                if let Some(&tank) = tanks.get(i - PLAYERS_LIST_OFSSET) {
                    line = format!(
                        "{line}   {}: {}",
                        tank.context().player_details().avatar,
                        tank.player().name(),
                    );
                    let name = tank.player().name();
                    line = match name.chars().count() {
                        0..9 => format!("{line}\t\t\t"),
                        9..17 => format!("{line}\t\t"),
                        17.. => format!("{line}\t"),
                    };
                    line = format!(
                        "{line}{:02} pts, {:03}% [{}], {}, {}, {})",
                        tank.context().score(),
                        tank.context().health(),
                        tank.health_bar(),
                        tank.context().position(),
                        tank.context().player_details().orientation,
                        tank.context().previous_action()
                    )
                }
            }

            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

/// Computes a balanced number of turns for the game, in relation with the world size
fn compute_game_turns(world_size: &WorldSize) -> usize {
    let cell_count = (world_size.x * world_size.y) as f64;

    (cell_count.sqrt() * f64::log2(cell_count * cell_count)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_AVATAR;

    const MINI_MAP_SIZE: usize = 10;

    fn generate_mini_world() -> Box<World> {
        let size = WorldSize {
            x: MINI_MAP_SIZE,
            y: MINI_MAP_SIZE,
        };
        let world = World {
            animation: false,
            map: Box::new([[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            max_turns: compute_game_turns(&size),
            rng: thread_rng(),
            size,
            tanks: HashMap::new(),
            turn_number: 0,
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
    fn test_compute_step() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        // In the middle, facing northwest, going backwards
        let position = Position { x: 2, y: 3 };
        let orientation = Orientation::NorthEast;
        let direction = Direction::Backward;

        let (from, to) = world.compute_step(&position, &orientation, &direction);
        assert_eq!((from.x, from.y), (position.x, position.y));
        assert_eq!((to.x, to.y), (1, 4));

        // Moving outside of the map, located on left edge, going west, route is "stationary"
        let position = Position { x: 0, y: 3 };
        let orientation = Orientation::West;
        let direction = Direction::Forward;

        let (from, to) = world.compute_step(&position, &orientation, &direction);
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
        let world = generate_mini_world();
        let unallocated_cells = world.count_cells(&MapCell::Unallocated);
        assert_eq!(100.0, world.compute_terrain_percentage(unallocated_cells));
    }

    #[test]
    fn test_random_field_location() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        for _ in 0..1000 {
            let random = world.get_random_location(MapCell::Unallocated);
            assert!(random.is_some());
            assert!(world.is_location_unallocated(&random.unwrap()));
        }
    }

    #[test]
    fn try_set_player_on_field_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);
        fill_fields(&mut world);

        let position = Position { x: 5, y: 2 };
        assert!(world.cell_read(&position) == MapCell::Terrain(Terrain::Field));

        let player_details = Details::new(DEFAULT_AVATAR, 10);
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

        let player_details = Details::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_none());
    }

    #[test]
    fn try_set_player_on_lake_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 3, y: 2 };
        assert!(world.cell_read(&position) == MapCell::Terrain(Terrain::Lake));

        let player_details = Details::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_some());
    }

    #[test]
    fn try_set_player_on_swamp_cell() {
        let mut world = generate_mini_world();
        populate_mini_world(&mut world);

        let position = Position { x: 5, y: 0 };
        assert!(world.cell_read(&position) == MapCell::Terrain(Terrain::Swamp));

        let player_details = Details::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(player_details, &position);
        assert!(result.is_some());
    }

    #[test]
    fn get_player_at_position() {
        let mut world = generate_mini_world();
        fill_fields(&mut world);

        let position = Position { x: 5, y: 2 };
        let player_details = Details::new(DEFAULT_AVATAR, 10);
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
        let lower_player_details = Details::new(DEFAULT_AVATAR, 10);
        let result = world.try_set_player_on_cell(lower_player_details, &position_lower_player);
        assert!(result.is_some());

        let position_upper_player = position_lower_player
            .follow(&Orientation::North, &world.size)
            .unwrap();
        let upper_player_details = Details::new(DEFAULT_AVATAR, lower_player_details.id + 1);
        let result = world.try_set_player_on_cell(upper_player_details, &position_upper_player);
        assert!(result.is_some());

        let firing_position_1 = position_upper_player.clone();
        let (direct_hit_players_1, indirect_hit_players_1) =
            world.get_hit_players(&firing_position_1);
        assert!(!direct_hit_players_1.is_empty());
        assert!(!indirect_hit_players_1.is_empty());
        assert_eq!(
            *direct_hit_players_1.first().unwrap(),
            upper_player_details.id
        );
        assert_eq!(
            *indirect_hit_players_1.first().unwrap(),
            lower_player_details.id
        );

        let firing_position_2 = position_lower_player.clone();
        let (direct_hit_players_2, indirect_hit_players_2) =
            world.get_hit_players(&firing_position_2);
        assert!(!direct_hit_players_2.is_empty());
        assert_eq!(
            *direct_hit_players_2.first().unwrap(),
            lower_player_details.id
        );
        assert!(!indirect_hit_players_2.is_empty());
        assert_eq!(
            *indirect_hit_players_2.first().unwrap(),
            upper_player_details.id
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
            lower_player_details.id
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
