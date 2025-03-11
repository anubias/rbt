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
    prev_scan_type: Action,
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
            prev_scan_type: Action::Scan(ScanType::Mono(Orientation::North)),
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
            println!("{}: {}, timestamp: {}", enemy.details.avatar, enemy.last_pos, enemy.timestamp);
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

    fn store_scan_data(&mut self, ctx: &Context) {
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
                        self.map[coordinate.y][coordinate.x] = scan_data.data[scan_y][scan_x];
                        match self.map[coordinate.y][coordinate.x] {
                            MapCell::Player(details, terrain) => {
                                if details.id != self.id {
                                    // It's another player
                                    self.update_enemy_data(coordinate, details)
                                } else {
                                    // It's us, only store terrain
                                    self.map[coordinate.y][coordinate.x] =
                                        MapCell::Terrain(terrain);
                                }
                            }
                            // TODO: explosions etc?
                            _ => (),
                        }
                    }
                }
            }
        }
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
        // TODO: find the most recent (or weakest if all same timestamp) enemy and shoot that target
        // TODO: is distance too long for positional shooting? Check readme for details.
        for enemy in self.enemies.values() {
            if enemy.details.alive {
                if (self.time - enemy.timestamp) <= 3 {
                    return Some(enemy.last_pos.clone());
                }
            }
        }
        None
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
        self.store_scan_data(ctx);

        // If (alive) enemies around, shoot at them. Otherwise act based on what is ahead.
        // TODO: cardinal shooting?
        // TODO: some smarter movement?
        if let Some(enemy_pos) = self.check_enemy_data() {
            Action::Fire(Aiming::Positional(enemy_pos))
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

    fn decide_next_scan_type(&mut self) -> Action {
        // Alternate between omni and mono scan and keep track of which one was done last
        match self.prev_scan_type {
            Action::Scan(ScanType::Omni) => {
                self.prev_scan_type = Action::Scan(ScanType::Mono(self.orientation))
            }
            Action::Scan(ScanType::Mono(_)) => self.prev_scan_type = Action::Scan(ScanType::Omni),
            _ => self.prev_scan_type = Action::Scan(ScanType::Omni),
        };
        self.prev_scan_type.clone()
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
        // TODO: After spotting an enemy, do the next scan in the direction the enemy was seen?
        let next_action = match self.last_action {
            Action::Idle => Action::Scan(ScanType::Omni),
            Action::Fire(_) => Action::Scan(ScanType::Omni),
            Action::Move(_) => self.decide_next_scan_type(),
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
