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
};
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Enemy {
    last_pos: Position,
    details: Details,
    timestamp: u32,
}
pub struct Joonas {
    id: u8,
    map: Vec<Vec<MapCell>>,
    last_action: Action,
    orientation: Orientation,
    position: Position,
    enemies: HashMap<u8, Enemy>, // key: player id
    prev_turn: Option<Rotation>,
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
            prev_turn: None,
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
                enemy.details.avatar, enemy.last_pos, enemy.timestamp
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
        println!("Enemy spotted in {}, timestamp: {}", pos, self.time);
        let enemy = Enemy {
            last_pos: pos.clone(),
            details: details.clone(),
            timestamp: self.time,
        };
        self.enemies
            .entry(details.id)
            .and_modify(|elem| {
                elem.last_pos = pos;
                elem.details = details;
                elem.timestamp = self.time;
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

    fn get_terrain_ahead(&self, ctx: &Context) -> MapCell {
        if let Some(cell_ahead) = ctx
            .position()
            .follow(&ctx.player_details().orientation, ctx.world_size())
        {
            // println!(
            //     "Terrain ahead {}, ({}, {})",
            //     self.map[cell_ahead.y][cell_ahead.x], cell_ahead.x, cell_ahead.y
            // );
            return self.map[cell_ahead.y][cell_ahead.x];
        }
        MapCell::Unallocated
    }

    // Check if recently spotted enemies near and return coordinates to shoot at
    fn check_enemy_data(&self) -> Option<Position> {
        // TODO: find the closest (or most recent if many within omniscan range?) enemy and shoot that target
        for enemy in self.enemies.values() {
            if enemy.details.alive {
                if (self.time - enemy.timestamp) <= 3 {
                    return Some(enemy.last_pos.clone());
                }
            }
        }
        None
    }

    fn get_enemy_direction(&self, enemy_pos: &Position) -> Orientation {
        let dx = enemy_pos.x as isize - self.position.x as isize;
        let dy = enemy_pos.y as isize - self.position.y as isize;

        // Check for horizontal/vertical
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

        // Check for diagonal
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

    fn decide_next_scan_type(&mut self, enemy_direction: Option<Orientation>) -> Action {
        // Alternate between omni and mono directly ahead. If enemies are near, scan towards them instead of ahead.
        match self.prev_scan_type {
            ScanType::Omni => {
                if let Some(enemy) = enemy_direction {
                    self.prev_scan_type = ScanType::Mono(enemy)
                } else {
                    self.prev_scan_type = ScanType::Mono(self.orientation)
                }
            }
            ScanType::Mono(_) => self.prev_scan_type = ScanType::Omni,
        };
        Action::Scan(self.prev_scan_type.clone())
    }

    fn decide_firing_option(&mut self, enemy_pos: Position) -> Action {
        // these are the ranges for omni and mono scans and therefore the ranges of positional and cardinal shooting
        let positional_range = SCANNING_DISTANCE / 2;
        let cardinal_range = SCANNING_DISTANCE - 1;

        // Check if in range for positional shooting
        if self.position.x.abs_diff(enemy_pos.x) <= positional_range
            && self.position.y.abs_diff(enemy_pos.y) <= positional_range
        {
            return Action::Fire(Aiming::Positional(enemy_pos));
        }

        // How about cardinal shooting
        if self.position.x.abs_diff(enemy_pos.x) <= cardinal_range
            && self.position.y.abs_diff(enemy_pos.y) <= cardinal_range
        {
            if let Some(orientation) = self.get_cardinal_shooting_direction(&enemy_pos) {
                return Action::Fire(Aiming::Cardinal(orientation));
            }
        }

        // Not within firing distance or direction, look puzzled and do a scan
        self.decide_next_scan_type(Some(self.get_enemy_direction(&enemy_pos)))
    }

    fn decide_rotation_direction(&mut self) -> Action {
        // If we rotated as last action and still need to rotate, keep rotating in the same direction. Otherwise random.
        match self.prev_turn {
            Some(Rotation::Clockwise) => Action::Rotate(Rotation::Clockwise),
            Some(Rotation::CounterClockwise) => Action::Rotate(Rotation::CounterClockwise),
            None => {
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    self.prev_turn = Some(Rotation::Clockwise);
                    Action::Rotate(Rotation::Clockwise)
                } else {
                    self.prev_turn = Some(Rotation::CounterClockwise);
                    Action::Rotate(Rotation::CounterClockwise)
                }
            }
        }
    }

    // Decide next action based on scanned data
    fn handle_scan(&mut self, ctx: &Context) -> Action {
        self.store_and_analyze_scan_data(ctx);

        // If (alive) enemies around, shoot at them. Otherwise act based on what is ahead.
        // TODO: some smarter movement?
        if let Some(enemy_pos) = self.check_enemy_data() {
            self.decide_firing_option(enemy_pos)
        } else {
            match self.get_terrain_ahead(ctx) {
                MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
                MapCell::Terrain(Terrain::Forest(_)) => self.decide_rotation_direction(),
                MapCell::Terrain(Terrain::Lake) => self.decide_rotation_direction(),
                MapCell::Terrain(Terrain::Swamp) => self.decide_rotation_direction(),
                MapCell::Player(_, _) => self.decide_rotation_direction(),
                MapCell::Unallocated => Action::Scan(ScanType::Mono(self.orientation)),
                _ => Action::Fire(Aiming::Cardinal(self.orientation)),
            }
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
            Action::Move(_) => self.prev_turn = None,
            _ => (),
        }

        // TODO: when close to map border, do directional scan instead of omni?
        // TODO: if the enemy we are shooting at is also shooting, shoot again instead of scan? => can't know enemy action :(
        let next_action = match self.last_action {
            Action::Idle => Action::Scan(ScanType::Omni),
            Action::Fire(_) => Action::Scan(ScanType::Omni),
            Action::Move(_) => self.decide_next_scan_type(None),
            Action::Rotate(_) => Action::Scan(ScanType::Omni),
            Action::Scan(_) => self.handle_scan(&context),
        };
        self.print_enemies();
        self.print_map();
        return next_action;
    }

    fn name(&self) -> String {
        "Joonas".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
