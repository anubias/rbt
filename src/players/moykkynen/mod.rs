use super::player::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Enemy {
    last_pos: Position,
    details: PlayerDetails,
}
pub struct Joonas {
    id: u8,
    map: Vec<Vec<MapCell>>,
    last_action: Action,
    orientation: Orientation,
    position: Position,
    enemies: HashMap<u8, Enemy>, // key: player id
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

    fn store_scan_data(&mut self, ctx: &Context) {
        // Initialize our own map according to world size
        if self.map.is_empty() {
            self.map.resize(ctx.world_size().y, Vec::new());
            for row in &mut self.map {
                row.resize(ctx.world_size().x, MapCell::Unallocated);
            }
        }

        if let Some(scan_data) = ctx.scanned_data() {
            // Calculate the origo of scanned data as world coordinate. Use isize so calculations are possible with negative values.
            let (own_x, own_y) = (ctx.position().x as isize, ctx.position().y as isize);
            let dist = SCANNING_DISTANCE as isize;
            let (scan_origo_x, scan_origo_y) = match scan_data.scan_type {
                ScanType::Omni => (own_x - (dist / 2), own_y - (dist / 2)),
                // TODO: mono scans...
                _ => (0, 0),
            };

            // Now that we know the origo of scanned data in world coordinates, update our own map
            for scan_y in 0..scan_data.data.len() {
                for scan_x in 0..scan_data.data[scan_y].len() {
                    let x = scan_origo_x + scan_x as isize;
                    let y = scan_origo_y + scan_y as isize;

                    if x >= 0 && x < ctx.world_size().x as isize {
                        if y >= 0 && y < ctx.world_size().y as isize {
                            let scan_cell = scan_data.data[scan_y as usize][scan_x as usize];
                            self.map[y as usize][x as usize] = scan_cell;

                            // If the cell in scanned data is another player, update enemy data
                            match scan_cell {
                                MapCell::Player(details, _) => {
                                    if details.id != self.id {
                                        self.update_enemy_data(
                                            Position {
                                                x: x as usize,
                                                y: y as usize,
                                            },
                                            details,
                                        )
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_enemy_data(&mut self, pos: Position, details: PlayerDetails) {
        let enemy = Enemy {
            last_pos: pos.clone(),
            details: details.clone(),
        };
        self.enemies
            .entry(details.id)
            .and_modify(|elem| {
                elem.last_pos = pos;
                elem.details = details
            })
            .or_insert(enemy);
    }

    fn get_terrain_ahead(&self, ctx: &Context) -> MapCell {
        if let Some(cell_ahead) = ctx.position().follow(ctx.orientation(), ctx.world_size()) {
            return self.map[cell_ahead.y][cell_ahead.x];
        }
        MapCell::Unallocated
    }

    // Check if known enemies near and return coordinates to shoot at
    fn check_enemy_data(&self) -> Option<Position> {
        for enemy in self.enemies.values() {
            if enemy.details.alive {
                return Some(enemy.last_pos.clone());
            }
        }
        None
    }

    // Decide next action based on scanned data
    fn handle_scan(&mut self, ctx: &Context) -> Action {
        self.store_scan_data(ctx);

        // If enemies around, shoot at them. Otherwise act based on what is ahead.
        // TODO: some smarter movement
        if let Some(enemy_pos) = self.check_enemy_data() {
            println!("Firing at {}", enemy_pos);
            Action::Fire(Aiming::Positional(enemy_pos))
        } else {
            match self.get_terrain_ahead(ctx) {
                MapCell::Terrain(Terrain::Field) => Action::Move(Direction::Forward),
                MapCell::Terrain(Terrain::Forest(_)) => Action::Rotate(Rotation::Clockwise),
                MapCell::Terrain(Terrain::Lake) => Action::Rotate(Rotation::Clockwise),
                MapCell::Terrain(Terrain::Swamp) => Action::Rotate(Rotation::Clockwise),
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
        self.orientation = context.orientation().clone();
        self.position = context.position().clone();

        let next_action = match context.previous_action() {
            Action::Idle => Action::Scan(ScanType::Omni),
            Action::Fire(_) => Action::Scan(ScanType::Omni),
            Action::Move(_) => Action::Scan(ScanType::Omni),
            Action::Rotate(_) => Action::Scan(ScanType::Omni),
            Action::Scan(_) => self.handle_scan(&context),
        };
        // self.print_map();
        return next_action;
    }

    fn name(&self) -> String {
        "Joonas".to_string()
    }

    fn is_ready(&self) -> bool {
        false
    }
}
