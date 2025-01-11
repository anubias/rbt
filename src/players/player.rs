use crate::game::{
    types::{Direction, Rotation, ScanType},
    user::Context,
};

pub trait Player {
    /// This is the player's turn to fight
    fn act(&mut self, context: &Context) -> Action;

    /// Returns the player's name
    fn name(&self) -> String;

    /// This indicates whether the player is ready to battle
    fn is_ready(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub enum Action {
    #[default]
    Idle,
    Fire,
    Move(Direction),
    Rotate(Rotation),
    Scan(ScanType),
}
