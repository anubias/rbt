use super::player::*;

pub struct Joonas {
    last_action: Action,
    direction: Orientation,
}

// Public functions
impl Joonas {
    pub fn new() -> Self {
        Self {
            last_action: Action::default(),
            direction: Orientation::default(),
        }
    }
}

// Private functions
impl Joonas {
    // if last action was scan, do this
    fn analyze_scanned_data(&mut self, ctx: &Context) -> Action {
        match ctx.scanned_data() {
            _ => Action::default(),
        }
    }
}

impl Player for Joonas {
    fn act(&mut self, _context: &Context) -> Action {
        match self.last_action {
            Action::Scan(_) => self.analyze_scanned_data(_context),
            // Action::Fire => Action::Scan(ScanType::Directional(self.direction)),
            // Action::Move(dir) => Action
            _ => Action::Scan(ScanType::Omni),
        }

        // for now, drive around scanning forward once in a while and avoid obstacles
    }

    fn name(&self) -> String {
        "Joonas☢️".to_string()
    }

    fn is_ready(&self) -> bool {
        // I'm not ready to battle yet :(
        false
    }
}
