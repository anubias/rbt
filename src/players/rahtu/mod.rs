use crate::api::{
    action::Action,
    context::Context,
    player::Player
};

mod plans;
mod shared;
mod sensors;

use shared::Data;

const DEBUG_PRINTS: bool = false;

pub struct Rahtu {
    pub data: Data,
}

impl Rahtu {
    pub fn new() -> Self {
        Self {
            data: Data::new(),
        }
    }
}

impl Player for Rahtu {
    fn act(&mut self, context: Context) -> Action {
        {
            self.data.map.update_map(&context);
        }
        if DEBUG_PRINTS {
            println!("{context}");
        }
        let action = plans::get_next_action(&mut self.data, &context);

        if DEBUG_PRINTS {
            dbg!(&action);
        }
        action
    }

    fn name(&self) -> String {
        "Nascar".to_string()
    }
    fn is_ready(&self) -> bool {
        true
    }
}
