use super::player::*;

pub struct Siimesjarvi {}

impl Siimesjarvi {
    pub fn new() -> Self {
        Self {}
    }
}

impl Player for Siimesjarvi {
    fn act(&mut self, context: &Context) -> Action {
        match context.position() {
            _ => {}
        }

        Action::default()
    }

    fn name(&self) -> String {
        "Joni Siimesjarvi".to_string()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_that_initial_player_values_are_correct() {
        let s = Siimesjarvi::new();
        assert_eq!(false, s.is_ready());
    }
}
