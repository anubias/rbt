use super::player::*;

pub struct Siimesjarvi {
    strategy: Box<dyn Strategy>,
}

impl Siimesjarvi {
    pub fn new() -> Self {
        Self {
            strategy: Box::new(FireForget::default())
        }
    }
}

impl Player for Siimesjarvi {
    fn act(&mut self, context: Context) -> Action {
        self.strategy.get_next_action(context)
    }

    fn name(&self) -> String {
        "Joni Siimesjarvi".to_string()
    }
}

/// Decides next action
trait Strategy {
    fn get_next_action(&mut self, context: Context) -> Action;
}

/// Basic strategy to get started
/// 
/// Blindly fire in all directions in clockwise pattern
#[derive(Default)]
struct FireForget {
    orientation: Orientation
}

impl FireForget {
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
}

impl Strategy for FireForget {
    fn get_next_action(&mut self, _context: Context) -> Action {
        Action::Fire(Aiming::Cardinal(self.get_next_orientation()))
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
    fn fireforget_strategy_provides_next_orientation_clockwise() {
        let mut s: FireForget = FireForget { orientation: Orientation::North };
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
