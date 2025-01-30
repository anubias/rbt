use super::player::*;

pub struct Siimesjarvi {
    strategy: Strategy,
}

impl Siimesjarvi {
    pub fn new() -> Self {
        Self {
            strategy: Strategy::Basic(BasicStrategy::default())
        }
    }
}

impl Player for Siimesjarvi {
    fn act(&mut self, context: Context) -> Action {
        match self.strategy {
            Strategy::Basic(ref mut x) => { x.get_next_action(context) },
            Strategy::Advanced(ref mut x) => { x.get_next_action(context) }
        }
    }

    fn name(&self) -> String {
        "Joni Siimesjarvi".to_string()
    }
}

enum Strategy {
    Basic(BasicStrategy),
    Advanced(AdvancedStrategy)
}

/// Basic strategy to get started
/// 
/// Blindly fire in all directions in clockwise pattern
#[derive(Default)]
struct BasicStrategy {
    orientation: Orientation
}

impl BasicStrategy {
    /// Provides Orientation that should be used next
    /// 
    /// Updates the next orientation so repeatedly calling this will eventually
    /// give all orientations
    fn get_next_orientation(&mut self) -> Orientation {
        match self.orientation {
            Orientation::North => {
                self.orientation = Orientation::NorthEast;
                Orientation::North
            }
            Orientation::NorthEast => {
                self.orientation = Orientation::East;
                Orientation::NorthEast
            }
            Orientation::East => {
                self.orientation = Orientation::SouthEast;
                Orientation::East
            }
            Orientation::SouthEast => {
                self.orientation = Orientation::South;
                Orientation::SouthEast
            }
            Orientation::South => {
                self.orientation = Orientation::SouthWest;
                Orientation::South
            }
            Orientation::SouthWest => {
                self.orientation = Orientation::West;
                Orientation::SouthWest
            }
            Orientation::West => {
                self.orientation = Orientation::NorthWest;
                Orientation::West
            }
            Orientation::NorthWest => {
                self.orientation = Orientation::North;
                Orientation::NorthWest
            }
        }
    }

    pub fn get_next_action(&mut self, _context: Context) -> Action {
        Action::Fire(Aiming::Cardinal(self.get_next_orientation()))
    }
}

struct AdvancedStrategy {
    previous_action: Action,
    previous_context: Option<Context>,
    world_map: [MapCell; MAX_WORLD_SIZE*MAX_WORLD_SIZE]
}

impl AdvancedStrategy {
    fn new() -> Self {
        Self {
            previous_action: Action::Idle,
            previous_context: Option::None,
            world_map: [MapCell::Unknown; MAX_WORLD_SIZE*MAX_WORLD_SIZE]
        }
    }

    fn get_next_action(&mut self, context: Context) -> Action {
        let next_action: Action = match self.previous_action {
            Action::Idle => { Action::Scan(ScanType::Omni)}
            _ => Action::Idle
        };
        self.previous_context = Option::Some(context);
        next_action
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn check_initial_player_values() {
        let s = Siimesjarvi::new();
        assert_eq!(false, s.is_ready());
    }

    #[test]
    fn basic_strategy_provides_next_orientation_clockwise() {
        let mut s: BasicStrategy = BasicStrategy { orientation: Orientation::North };
        assert_eq!(Orientation::North, s.get_next_orientation());
        assert_eq!(Orientation::NorthEast, s.get_next_orientation());
        assert_eq!(Orientation::East, s.get_next_orientation());
        assert_eq!(Orientation::SouthEast, s.get_next_orientation());
        assert_eq!(Orientation::South, s.get_next_orientation());
        assert_eq!(Orientation::SouthWest, s.get_next_orientation());
        assert_eq!(Orientation::West, s.get_next_orientation());
        assert_eq!(Orientation::NorthWest, s.get_next_orientation());
        assert_eq!(Orientation::North, s.get_next_orientation());
    }

}
