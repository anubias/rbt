use super::player::*;

struct EnemyInfo {
    timestamp: u32,
    position: Position,
    max_health: u32,
}
pub struct PlAgiAntti {
    game_time: u32,
    terrain_map: Vec<Vec<Option<Terrain>>>,
    enemies: Vec<EnemyInfo>,
}

impl PlAgiAntti {
    pub fn new() -> Self {
        Self {
            game_time: 0,
            terrain_map: Vec::new(),
            enemies: Vec::new(),
        }
    }

    fn evaluate_threat(&self, context: &Context) -> u32 {
        // Check if there are any enemies in the vicinity.
        let mut threat_level = 0;
        for enemy in &self.enemies {
            let time_since_observed = self.game_time - enemy.timestamp;
            if time_since_observed < 5 && context.position().manhattan_distance(&enemy.position) < SCANNING_DISTANCE {
                threat_level += 1;
            }
        }
        return threat_level;
    }

    fn evaluate_opportunity(&self, context: &Context) -> u32 {
        // Check if we know the position of any enemies.
        let mut opportunity_level = 0;
        for enemy in &self.enemies {
            if enemy.timestamp == self.game_time - 1 {
                // We know where the enemy was on previous turn.
                let enemy_distance = context.position().pythagorean_distance(&enemy.position);
                if enemy_distance < POSITIONAL_AIMING_DISTANCE {
                    // The enemy is close enough for positional aiming.
                    opportunity_level += 10;
                } else if enemy_distance < SCANNING_DISTANCE  &&
                Orientation::direction_towards(&context.position(), &enemy.position) == context.orientation() {
                    // The enemy is close enough for cardinal aiming and in the correct direction.
                    opportunity_level += 10;
                }
            }
        }
        // TODO: Implement logic to assess the opportunity level
        rand::random::<u32>() % 2
    }

    fn defensive_action(&self, _context: &Context) -> Action {
        // TODO: Implement logic for defensive actions
        Action::Move(Direction::Backward)
    }

    fn offensive_action(&self, _context: &Context) -> Action {
        // TODO: Implement logic for offensive actions
        Action::Fire
    }

    fn exploratory_action(&self, _context: &Context) -> Action {
        // TODO: Implement logic for exploratory actions
        Action::Scan(ScanType::Directional(Orientation::East))
    }
}

impl Player for PlAgiAntti {
    fn act(&mut self, context: &Context) -> Action {
        if self.terrain_map.is_empty() {
            // Initialize the terrain map.
            self.terrain_map = vec![vec![None; MAX_WORLD_SIZE]; MAX_WORLD_SIZE];
        }
        // Keep track of game time.
        self.game_time += 1;

        // Evaluate the current state
        let threat_level = self.evaluate_threat(context);
        let opportunity_level = self.evaluate_opportunity(context);

        // Decision tree based on state evaluation
        if threat_level >= opportunity_level {
            self.defensive_action(context)
        } else if opportunity_level > 0 {
            self.offensive_action(context)
        } else {
            self.exploratory_action(context)
        }
    }

    fn name(&self) -> String {
        "Tantti".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
