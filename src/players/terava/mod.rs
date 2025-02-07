use super::player::*;
use core::time;
use std::cmp::min;

struct EnemyInfo {
    timestamp: u32,
    position: Position,
    //max_health: u32,
}
pub struct PlAgiAntti {
    game_time: u32,
    terrain_map: Vec<Vec<Option<Terrain>>>,
    enemies: Vec<EnemyInfo>,
    location: Position,
    direction: Orientation,
    health: u8,
}

impl PlAgiAntti {
    pub fn new() -> Self {
        Self {
            game_time: 0,
            terrain_map: Vec::new(),
            enemies: Vec::new(),
            location: Position { x: 0, y: 0 },
            direction: Orientation::North,
            health: 0,
        }
    }

    // Calculates how many steps are needed to reach cardinal firing range.
    // TODO: Doesn't account for diagonal movement.
    // TODO: Doesn't account for rotation or turning along the way. Needs a proper pathfinding algorithm for that.
    fn distance_to_firing_range(&self, enemy_position: &Position) -> u32 {
        // The shortest path to reach a position that is in firing range:
        let mut steps = 0;
        let (mut dx, mut dy) = self.location.delta(enemy_position);
        // 1. If already within positional firing range, return 0.
        if self.in_positional_firing_range(enemy_position) {
            return steps;
        }
        // 2. Move along the x-axis until the x-coordinate is within CARDINAL_AIMING_DISTANCE.
        const CARDINAL_AIMING_DISTANCE: isize = SCANNING_DISTANCE as isize;
        if dx.abs() > CARDINAL_AIMING_DISTANCE as isize {
            steps += (dx.abs() - CARDINAL_AIMING_DISTANCE as isize) as u32;
            // Update dx to reflect the movement.
            dx -= (steps as isize) * dx.signum();
        }
        // 3. Move along the y-axis until the y-coordinate is within CARDINAL_AIMING_DISTANCE.
        if dy.abs() > CARDINAL_AIMING_DISTANCE as isize {
            steps += (dy.abs() - CARDINAL_AIMING_DISTANCE as isize) as u32;
            // Update dy to reflect the movement.
            dy -= (steps as isize) * dy.signum();
        }
        // 4. The position is now within cardinal firing distance.
        //    The closest position that can be targeted with cardinal aiming is at dx==0, dy==0 or dx==dy.
        steps += min(min(dx.abs(), dy.abs()), (dx.abs() - dy.abs()).abs()) as u32;
        return steps;
    }

    // Checks if an enemy tank is close enough for positional aiming.
    fn in_positional_firing_range(&self, enemy_position: &Position) -> bool {
        let (dx, dy) = self.location.delta(enemy_position);
        const POSITIONAL_AIMING_DISTANCE: isize = (SCANNING_DISTANCE / 2) as isize;
        return dx.abs() < POSITIONAL_AIMING_DISTANCE && dy.abs() < POSITIONAL_AIMING_DISTANCE;
    }

    // Checks if an enemy tank is close enough and in suitable direction for cardinal aiming.
    fn in_cardinal_firing_range(&self, enemy_position: &Position) -> bool {
        let (dx, dy) = self.location.delta(enemy_position);
        const CARDINAL_AIMING_DISTANCE: isize = SCANNING_DISTANCE as isize;
        return dx.abs() < CARDINAL_AIMING_DISTANCE
            && dy.abs() < CARDINAL_AIMING_DISTANCE
            && self.direction_to(enemy_position).is_some();
    }

    // Checks if the line from own location to target position is along any of the eight cardinal orientations.
    // Returns the orientation if the line is cardinal, otherwise None.
    fn direction_to(&self, target: &Position) -> Option<Orientation> {
        let (dx, dy) = self.location.delta(target);
        if dx == 0 {
            // The positions are on the same latitude.
            match dy.signum() {
                1 => Some(Orientation::North),
                -1 => Some(Orientation::South),
                _ => None,
            }
        } else if dy == 0 {
            // The positions are on the same longitude.
            match dx.signum() {
                1 => Some(Orientation::East),
                -1 => Some(Orientation::West),
                _ => None,
            }
        } else if dx.abs() == dy.abs() {
            // The positions are on a diagonal.
            match (dx.signum(), dy.signum()) {
                (1, 1) => Some(Orientation::NorthEast),
                (1, -1) => Some(Orientation::SouthEast),
                (-1, 1) => Some(Orientation::NorthWest),
                (-1, -1) => Some(Orientation::SouthWest),
                _ => None,
            }
        } else {
            None
        }
    }

    fn evaluate_threat(&self) -> u32 {
        // Check if there are any enemies in the vicinity.
        let mut threat_level = 0;
        for enemy in &self.enemies {
            let time_since_observed = self.game_time - enemy.timestamp;
            let steps_to_firing_range = self.distance_to_firing_range(&enemy.position);
            // The enemy poses a threat only when it has had enough time to move into firing range.
            if time_since_observed >= steps_to_firing_range {
                let mut threat_from_enemy = time_since_observed - steps_to_firing_range;
                // Scale down the threat if there is a long time from the last observation.
                if time_since_observed > 5 {
                    threat_from_enemy = threat_from_enemy / time_since_observed;
                }
                threat_level += threat_from_enemy;
            }

        }
        return threat_level;
    }

    fn evaluate_opportunity(&self) -> u32 {
        // Check if we know the position of any enemies.
        let mut opportunity_level = 0;
        for enemy in &self.enemies {
            if enemy.timestamp == self.game_time - 1 {
                // We know where the enemy was on previous turn.
                if self.in_positional_firing_range(&enemy.position)
                    || self.in_cardinal_firing_range(&enemy.position)
                {
                    // We can take only one shot, so the opportunity level is capped at 6.
                    opportunity_level = 6;
                }
            }
        }
        opportunity_level
    }

    fn defensive_action(&self) -> Action {
        // TODO: Implement logic for defensive actions
        Action::Move(Direction::Backward)
    }

    fn offensive_action(&self) -> Action {
        // TODO: Implement logic for offensive actions
        Action::Fire(Aiming::Cardinal(self.direction.clone()))
    }

    fn exploratory_action(&self) -> Action {
        // TODO: Implement logic for exploratory actions
        Action::Scan(ScanType::Mono(self.direction.clone()))
    }
}

impl Player for PlAgiAntti {
    fn act(&mut self, context: Context) -> Action {
        if self.terrain_map.is_empty() {
            // Initialize the terrain map.
            self.terrain_map = vec![vec![None; MAX_WORLD_SIZE]; MAX_WORLD_SIZE];
        }
        // Keep track of game time.
        self.game_time += 1;
        // Store context information.
        self.location = context.position().clone();
        self.direction = context.orientation().clone();
        self.health = context.health();
        // Store scan information.
        if let Some(_scan) = context.scanned_data() {
            /*for coord in scan.data.iter() {
                match coord {
                    MapCell::Player(_, _) => {
                        // TODO: Store enemy information
                    },
                    MapCell::Terrain(terr) => {
                        self.terrain_map[coord.y as usize][coord.x as usize] = terr;
                    }
                }
            }*/
        }

        // Evaluate the current state
        let threat_level = self.evaluate_threat();
        let opportunity_level = self.evaluate_opportunity();

        // Decision tree based on state evaluation
        if opportunity_level > threat_level {
            self.offensive_action()
        } else if threat_level > 0 {
            self.defensive_action()
        } else {
            self.exploratory_action()
        }
    }

    fn name(&self) -> String {
        // Randomize first letter of name from this list: F, H, K, L, P, R, T.
        let first_letter = match rand::random::<u32>() % 7 {
            0 => "F",
            1 => "H",
            2 => "K",
            3 => "L",
            4 => "P",
            5 => "R",
            _ => "T",
        };
        // The rest of the name is fixed.
        format!("{}antti", first_letter)
    }

    fn is_ready(&self) -> bool {
        false
    }
}
