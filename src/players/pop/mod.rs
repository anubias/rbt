use super::player::*;

pub struct Aurelian {
    live_iteration: usize,
    scan_iteration: usize,
    world_map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
}

// Public functions
impl Aurelian {
    pub fn new() -> Self {
        Self {
            live_iteration: 0,
            scan_iteration: 0,
            world_map: Box::new([[MapCell::Unknown; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
        }
    }
}

// Private functions
impl Aurelian {
    fn ai_logic(&mut self, context: &Context) -> Action {
        self.update_world_map(context);
        if let Some(enemy_position) = self.find_closest_enemy_position(context) {
            println!("enemy position: {enemy_position}");
        }

        match self.live_iteration % 2 {
            1 => Action::Scan(ScanType::Omni),
            _ => Action::Move(Direction::Forward),
        }
    }

    fn find_closest_enemy_position(&self, context: &Context) -> Option<Position> {
        let world_size = context.world_size();
        let origin_position = Position { x: 0, y: 0 };
        let opposite_position = Position {
            x: world_size.x,
            y: world_size.y,
        };
        let mut closest_distance = origin_position.pythagorean_distance(&opposite_position);
        let mut closest_player_position = None;

        for i in 0..world_size.y {
            for j in 0..world_size.x {
                match self.world_map[i][j] {
                    MapCell::Player(player) => {
                        if player != context.player_id() {
                            let player_pos = Position { x: j, y: i };
                            let distance = context.position().pythagorean_distance(&player_pos);
                            if distance < closest_distance {
                                closest_distance = distance;
                                closest_player_position = Some(player_pos);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        closest_player_position
    }

    fn update_world_map(&mut self, context: &Context) {
        if let Some(scan_result) = context.scanned_data() {
            if let Some(my_pos) =
                self.locate_myself_in_scanned_map(context.player_id(), scan_result)
            {
                let (start_x, start_y) = context.position().manhattan_distance(&my_pos);
                self.copy_scanned_data(scan_result, start_x, start_y);
            }
        }

        println!("+++++++++++++++++++++++");
        for i in 0..context.world_size().y {
            for j in 0..context.world_size().x {
                print!("{}", self.world_map[i][j]);
            }
            println!("");
        }
        println!("=======================");
    }

    fn locate_myself_in_scanned_map(
        &self,
        player_id: u8,
        scan_result: &ScanResult,
    ) -> Option<Position> {
        for i in 0..SCANNING_DISTANCE {
            for j in 0..SCANNING_DISTANCE {
                if scan_result.data[i][j] == MapCell::Player(player_id) {
                    return Some(Position { x: j, y: i });
                }
            }
        }

        None
    }

    fn copy_scanned_data(&mut self, scan_result: &ScanResult, start_x: isize, start_y: isize) {
        for i in 0..SCANNING_DISTANCE {
            let map_y = start_y + i as isize;

            if map_y >= 0 && map_y < MAX_WORLD_SIZE as isize {
                for j in 0..SCANNING_DISTANCE {
                    let map_x = start_x + j as isize;

                    if map_x >= 0 && map_x < MAX_WORLD_SIZE as isize {
                        let (x, y) = (map_x as usize, map_y as usize);
                        self.world_map[y][x] = scan_result.data[i][j];
                    }
                }
            }
        }
    }

    fn compute_horizon_distance(&self, context: &Context, iteration: usize) -> usize {
        let orientation = Orientation::from(iteration * 2);

        let mut position = context.position().clone();
        let mut distance = 0;
        loop {
            if let Some(pos) = position.follow(&orientation, context.world_size()) {
                position = pos;
                distance += 1;
            } else {
                distance = MAX_WORLD_SIZE;
            }

            if distance >= SCANNING_DISTANCE {
                break;
            }
        }

        distance
    }
}

impl Player for Aurelian {
    fn act(&mut self, context: &Context) -> Action {
        self.live_iteration += 1;

        if self.live_iteration == 1 {
            Action::Scan(ScanType::Omni)
        } else {
            self.ai_logic(context)
        }
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
