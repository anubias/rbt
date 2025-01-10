use crate::actor::ActorContext;
use crate::utils::{Action, Player};

pub struct Aurelian {
    name: String,
}

impl Aurelian {
    pub fn new() -> Self {
        Aurelian {
            name: "Aurelian".to_string(),
        }
    }
}

impl Player for Aurelian {
    fn act(&self, context: &ActorContext) -> Action {
        println!("My position is: {}", context.position);
        Action::Fire
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
