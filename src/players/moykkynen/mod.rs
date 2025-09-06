use crate::api::{
    action::Action,
    aiming::Aiming,
    context::Context,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    player::{Details, Player},
    position::{Position, SCANNING_DISTANCE},
    rotation::Rotation,
    scan::ScanType,
    world_size::WorldSize
};
use positionbuffer::PositionBuffer;
use rand::Rng;
use std::collections::HashMap;
mod positionbuffer;

const DEBUG_MODE: bool = false;

#[derive(Clone, Debug)]
pub struct Enemy {
    position: Position, // TODO: could remove this and use pos_history.last()
    details: Details,
    timestamp: u32,
    pos_history: PositionBuffer,
}
pub struct Joonas {
    id: u8,
    map: Vec<Vec<MapCell>>,
    last_action: Action,
    orientation: Orientation,
    position: Position,
    enemies: HashMap<u8, Enemy>, // key: player id
    ongoing_turn: Option<Rotation>,
    prev_scan_type: ScanType,
    time: u32,
}

// Public functions
impl Joonas {
    pub fn new() -> Self {
        Self {
            id: 0,
            map: Vec::new(),
            last_action: Action::default(),
            orientation: Orientation::default(),
            position: Position { x: 0, y: 0 },
            enemies: HashMap::new(),
            ongoing_turn: None,
            prev_scan_type: ScanType::Mono(Orientation::North),
            time: 0,
        }
    }
}

// Private functions
impl Joonas {
    #[allow(dead_code)]
    fn print_map(&self) {
        for row in &self.map {
            for cell in row {
                print!("{cell}");
            }
            println!();
        }
    }

    #[allow(dead_code)]
    fn print_enemies(&self) {
        for enemy in self.enemies.values() {
            println!(
                "{}: {}, timestamp: {}",
                enemy.details.avatar, enemy.position, enemy.timestamp
            );
        }
    }

    fn get_scan_origo(&self, scan_type: &ScanType) -> (isize, isize) {
        // Use isize in the calculations, as scan origo might be outside map boundaries and therefore negative.
        let (own_x, own_y) = (self.position.x as isize, self.position.y as isize);
        let dist = SCANNING_DISTANCE as isize;
        match scan_type {
            ScanType::Omni => (own_x - (dist / 2), own_y - (dist / 2)),
            ScanType::Mono(orientation) => match orientation {
                Orientation::North => (own_x - (dist / 2), own_y - (dist - 1)),
                Orientation::NorthEast => (own_x, own_y - (dist - 1)),
                Orientation::East => (own_x, own_y - (dist / 2)),
                Orientation::SouthEast => (own_x, own_y),
                Orientation::South => (own_x - (dist / 2), own_y),
                Orientation::SouthWest => (own_x - (dist - 1), own_y),
                Orientation::West => (own_x - (dist - 1), own_y - (dist / 2)),
                Orientation::NorthWest => (own_x - (dist - 1), own_y - (dist - 1)),
            },
        }
    }

    fn scan_coordinate_to_world_coordinate(
        &self,
        scan_origo: (isize, isize),
        scan_coord: Position,
    ) -> Option<Position> {
        let x = scan_origo.0 + scan_coord.x as isize;
        let y = scan_origo.1 + scan_coord.y as isize;
        if y >= 0 && y < self.map.len() as isize {
            if x >= 0 && x < self.map[y as usize].len() as isize {
                // Coordinate is valid
                return Some(Position {
                    x: x as usize,
                    y: y as usize,
                });
            }
        }
        None
    }

    fn update_enemy_data(&mut self, pos: Position, details: Details) {
        if DEBUG_MODE {
            println!("Enemy spotted in {}, timestamp: {}", pos, self.time);
        }
        let enemy = Enemy {
            position: pos.clone(),
            details: details.clone(),
            timestamp: self.time,
            pos_history: PositionBuffer::new(),
        };
        self.enemies
            .entry(details.id)
            .and_modify(|e| {
                e.position = pos.clone();
                e.details = details;
                e.timestamp = self.time;
                e.pos_history.push(pos);
            })
            .or_insert(enemy);
    }

    fn analyze_cell(&mut self, pos: Position) {
        match self.map[pos.y][pos.x] {
            MapCell::Player(details, terrain) => {
                if details.id != self.id {
                    // It's another player
                    self.update_enemy_data(pos, details)
                } else {
                    // It's us, remove player and only store terrain
                    self.map[pos.y][pos.x] = MapCell::Terrain(terrain);
                }
            }
            // TODO: consider if explosions etc. need some handling
            _ => (),
        }
    }

    fn store_and_analyze_scan_data(&mut self, ctx: &Context) {
        // Initialize our own map according to world size
        if self.map.is_empty() {
            self.map = vec![vec![MapCell::Unallocated; ctx.world_size().x]; ctx.world_size().y];
        }

        if let Some(scan_data) = ctx.scanned_data() {
            let (scan_origo_x, scan_origo_y) = self.get_scan_origo(&scan_data.scan_type);
            // Now that we know the origo of scanned data in world coordinates, update our own map
            for scan_y in 0..scan_data.data.len() {
                for scan_x in 0..scan_data.data[scan_y].len() {
                    let scan_coord = Position {
                        x: scan_x,
                        y: scan_y,
                    };
                    if let Some(coordinate) = self.scan_coordinate_to_world_coordinate(
                        (scan_origo_x, scan_origo_y),
                        scan_coord,
                    ) {
                        // We have the scanned map cell with world coordinates. Store it in our own map.
                        // TODO: enemies remain on own map in their last known position if they move out of scan range, probably not an issue though
                        self.map[coordinate.y][coordinate.x] = scan_data.data[scan_y][scan_x];
                        self.analyze_cell(coordinate);
                    }
                }
            }
        }
    }

    fn get_terrain_ahead(&self) -> MapCell {
        if let Some(cell_ahead) = self.position.follow(
            &self.orientation,
            &WorldSize {
                x: self.map[0].len(),
                y: self.map.len(),
            },
        ) {
            return self.map[cell_ahead.y][cell_ahead.x];
        }
        MapCell::Unallocated
    }

    // Go through enemy data and return enemy details, if a recently seen enemy is alive.
    // If multiple enemies to choose from, pick based on firing possibility and timestamp (most recent)
    // If no recently seen enemies alive, return None.
    fn check_enemy_data(&self) -> Option<Enemy> {
        let mut target_enemy: Option<Enemy> = None;
        for enemy in self.enemies.values() {
            if enemy.details.alive && (self.time - enemy.timestamp) <= 3 {
                if let Some(prev_enemy) = target_enemy.clone() {
                    // Another enemy already chosen as target, do some comparing
                    if self.position.could_hit_positionally(&enemy.position)
                        && self.position.could_hit_positionally(&prev_enemy.position)
                    {
                        // Both are within positional range, change target if new enemy has bigger timestamp
                        if enemy.timestamp > prev_enemy.timestamp {
                            target_enemy = Some(enemy.clone());
                        }
                    } else if self.position.could_hit_positionally(&enemy.position) {
                        // Previous enemy not within positional range, but new one is
                        target_enemy = Some(enemy.clone());
                    } else if self.position.could_hit_positionally(&prev_enemy.position) {
                        // Previous enemy is within positional range, but new one is not
                        continue;
                    } else {
                        // Neither within positional range, how about cardinal shooting?
                        if self.position.could_hit_cardinally(&enemy.position) {
                            target_enemy = Some(enemy.clone());
                        } else if self.position.could_hit_cardinally(&prev_enemy.position) {
                            continue;
                        }
                    }
                } else {
                    // No enemy chosen as target yet, let's go with this one
                    target_enemy = Some(enemy.clone());
                }
            }
        }
        target_enemy
    }

    fn get_enemy_direction(&self, enemy_pos: &Position) -> Orientation {
        let dx = enemy_pos.x as isize - self.position.x as isize;
        let dy = enemy_pos.y as isize - self.position.y as isize;

        // Check for direct horizontal/vertical
        if dx == 0 {
            if dy > 0 {
                return Orientation::South;
            } else if dy < 0 {
                return Orientation::North;
            }
        }
        if dy == 0 {
            if dx > 0 {
                return Orientation::East;
            } else if dx < 0 {
                return Orientation::West;
            }
        }

        // Check for diagonal areas
        return match (dx.signum(), dy.signum()) {
            (1, 1) => Orientation::SouthEast,
            (1, -1) => Orientation::NorthEast,
            (-1, -1) => Orientation::NorthWest,
            (-1, 1) => Orientation::SouthWest,
            _ => unreachable!(), // signum() returned NaN for either dx or dy value
        };
    }

    fn get_cardinal_shooting_direction(&self, enemy_pos: &Position) -> Option<Orientation> {
        let dx = enemy_pos.x as isize - self.position.x as isize;
        let dy = enemy_pos.y as isize - self.position.y as isize;

        // Check for horizontal/vertical
        if dx == 0 {
            if dy > 0 {
                return Some(Orientation::South);
            } else if dy < 0 {
                return Some(Orientation::North);
            }
        }
        if dy == 0 {
            if dx > 0 {
                return Some(Orientation::East);
            } else if dx < 0 {
                return Some(Orientation::West);
            }
        }

        // Check for exact diagonal
        if dx.abs() == dy.abs() {
            return match (dx.signum(), dy.signum()) {
                (1, 1) => Some(Orientation::SouthEast),
                (1, -1) => Some(Orientation::NorthEast),
                (-1, -1) => Some(Orientation::NorthWest),
                (-1, 1) => Some(Orientation::SouthWest),
                _ => unreachable!(), // signum() returned NaN for either dx or dy value
            };
        }

        // Not in position to fire cardinal, damnit!
        None
    }

    fn facing_map_border(&self) -> bool {
        let omniscan_distance = SCANNING_DISTANCE / 2;
        let south_boundary = self.map.len() - omniscan_distance;
        let east_boundary = self.map[0].len() - omniscan_distance;

        match self.orientation {
            Orientation::North => if self.position.y < omniscan_distance {
                return true;
            },
            Orientation::NorthEast => if self.position.y < omniscan_distance ||  self.position.x > east_boundary{
                return true;
            },
            Orientation::East => if self.position.x > east_boundary {
                return true;
            },
            Orientation::SouthEast => if self.position.y > south_boundary || self.position.x > east_boundary {
                return true;
            },
            Orientation::South => if self.position.y > south_boundary {
                return true;
            },
            Orientation::SouthWest => if self.position.y > south_boundary || self.position.x < omniscan_distance {
                return true;
            },
            Orientation::West => if self.position.x < omniscan_distance {
                return true;
            },
            Orientation::NorthWest => if self.position.y < omniscan_distance || self.position.x < omniscan_distance {
                return true;
            },
        }
        return false;
    }

    fn decide_next_scan_type(&mut self, enemy_direction: Option<Orientation>) -> Action {
        // Alternate between omni and mono directly ahead. If enemies are near, scan towards them instead of ahead.
        // If facing near the border of the map, only do mono scan.
        match self.prev_scan_type {
            ScanType::Omni => {
                if let Some(enemy) = enemy_direction {
                    self.prev_scan_type = ScanType::Mono(enemy)
                } else if self.facing_map_border() {
                    self.prev_scan_type = ScanType::Omni
                }
                else {
                    self.prev_scan_type = ScanType::Mono(self.orientation)
                }
            }
            ScanType::Mono(_) => self.prev_scan_type = ScanType::Omni,
        };
        Action::Scan(self.prev_scan_type.clone())
    }

    fn decide_firing_option(&mut self, enemy: Enemy) -> Action {
        // these are the ranges for omni and mono scans and therefore the ranges of positional and cardinal shooting
        let positional_range = SCANNING_DISTANCE / 2;
        let cardinal_range = SCANNING_DISTANCE - 1;

        // Check if in range for positional shooting
        if self.position.x.abs_diff(enemy.position.x) <= positional_range
            && self.position.y.abs_diff(enemy.position.y) <= positional_range
        {
            return Action::Fire(Aiming::Positional(enemy.position));
        }

        // How about cardinal shooting
        if self.position.x.abs_diff(enemy.position.x) <= cardinal_range
            && self.position.y.abs_diff(enemy.position.y) <= cardinal_range
        {
            if let Some(orientation) = self.get_cardinal_shooting_direction(&enemy.position) {
                return Action::Fire(Aiming::Cardinal(orientation));
            }
        }

        // Enemy cannot be shot at. If the enemy has remained stationary for a while, continue exploration. Otherwise do a scan.
        if enemy.pos_history.is_full()
            && enemy.pos_history.all_equal()
            && self.last_action == Action::Scan(ScanType::Omni)
        {
            return self.explore();
        }
        self.decide_next_scan_type(Some(self.get_enemy_direction(&enemy.position)))
    }

    fn decide_rotation_direction(&mut self) -> Action {
        // If we rotated as last action and still need to rotate, keep rotating in the same direction. Otherwise random.
        match self.ongoing_turn {
            Some(Rotation::Clockwise) => Action::Rotate(Rotation::Clockwise),
            Some(Rotation::CounterClockwise) => Action::Rotate(Rotation::CounterClockwise),
            None => {
                let mut rng = rand::rng();
                if rng.random_bool(0.5) {
                    self.ongoing_turn = Some(Rotation::Clockwise);
                    Action::Rotate(Rotation::Clockwise)
                } else {
                    self.ongoing_turn = Some(Rotation::CounterClockwise);
                    Action::Rotate(Rotation::CounterClockwise)
                }
            }
        }
    }

    // Move forward if possible, otherwise turn to a random direction until no obstacle ahead
    // TODO: some smarter movement?
    fn explore(&mut self) -> Action {
        match self.get_terrain_ahead() {
            MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
            MapCell::Terrain(Terrain::Forest(_)) => self.decide_rotation_direction(),
            MapCell::Terrain(Terrain::Lake) => self.decide_rotation_direction(),
            MapCell::Terrain(Terrain::Swamp) => self.decide_rotation_direction(),
            MapCell::Player(_, _) => self.decide_rotation_direction(),
            MapCell::Unallocated => Action::Scan(ScanType::Mono(self.orientation)),
            _ => Action::Fire(Aiming::Cardinal(self.orientation)),
            // TODO: handle explosion/shell somehow?
        }
    }

    // Decide next action based on scanned data
    fn handle_scan(&mut self, ctx: &Context) -> Action {
        self.store_and_analyze_scan_data(ctx);

        // If (alive) enemies around, shoot at them. Otherwise act based on what is ahead.
        if let Some(enemy) = self.check_enemy_data() {
            self.decide_firing_option(enemy)
        } else {
            self.explore()
        }
    }
}

impl Player for Joonas {
    fn act(&mut self, context: Context) -> Action {
        // Update our own details
        self.id = context.player_details().id.clone();
        self.last_action = context.previous_action().clone();
        self.orientation = context.player_details().orientation.clone();
        self.position = context.position().clone();
        self.time += 1;

        // After turning completed and moving again, remove information about ongoing turn
        match self.last_action {
            Action::Move(_) => self.ongoing_turn = None,
            _ => (),
        }

        // TODO: when close to map border, do directional scan instead of omni?
        let next_action = match self.last_action {
            Action::Idle => Action::Scan(ScanType::Omni),
            Action::Fire(Aiming::Positional(_)) => Action::Scan(ScanType::Omni),
            Action::Fire(Aiming::Cardinal(_)) => Action::Scan(ScanType::Mono(self.orientation)),
            Action::Move(_) => self.decide_next_scan_type(None),
            Action::Rotate(_) => Action::Scan(ScanType::Omni),
            Action::Scan(_) => self.handle_scan(&context),
        };
        if DEBUG_MODE {
            self.print_enemies();
            self.print_map();
        }
        return next_action;
    }

    fn name(&self) -> String {
        "Joonas".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
