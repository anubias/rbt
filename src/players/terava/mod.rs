use super::player::*;

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
            if time_since_observed < 5
            {
                // We have seen the enemy recently.
                if self.in_positional_firing_range(&enemy.position)
                    || self.in_cardinal_firing_range(&enemy.position)
                {
                    threat_level += time_since_observed;
                }
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

        // Evaluate the current state
        let threat_level = self.evaluate_threat();
        let opportunity_level = self.evaluate_opportunity();

        // Decision tree based on state evaluation
        if threat_level >= opportunity_level {
            self.defensive_action()
        } else if opportunity_level > 0 {
            self.offensive_action()
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
        true
    }
}
