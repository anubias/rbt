use super::player::*;

pub struct PlAgiAntti {}

impl PlAgiAntti {
    pub fn new() -> Self {
        Self {}
    }

    fn evaluate_threat(&self, _context: &Context) -> u32 {
        // TODO: Implement logic to assess the threat level
        rand::random::<u32>() % 2
    }

    fn evaluate_opportunity(&self, _context: &Context) -> u32 {
        // TODO: Implement logic to assess the opportunity level
        rand::random::<u32>() % 2
    }

    fn defensive_action(&self, _context: &Context) -> Action {
        // TODO: Implement logic for defensive actions
        Action::Move(Direction::Backward)
    }

    fn offensive_action(&self, _context: &Context) -> Action {
        // TODO: Implement logic for offensive actions
        Action::Fire(Aiming::Cardinal(Orientation::default()))
    }

    fn exploratory_action(&self, _context: &Context) -> Action {
        // TODO: Implement logic for exploratory actions
        Action::Scan(ScanType::Directional(Orientation::East))
    }
}

impl Player for PlAgiAntti {
    fn act(&mut self, context: Context) -> Action {
        // Evaluate the current state
        let threat_level = self.evaluate_threat(&context);
        let opportunity_level = self.evaluate_opportunity(&context);

        // Decision tree based on state evaluation
        if threat_level >= opportunity_level {
            self.defensive_action(&context)
        } else if opportunity_level > 0 {
            self.offensive_action(&context)
        } else {
            self.exploratory_action(&context)
        }
    }

    fn name(&self) -> String {
        "Tantti".to_string()
    }
}
