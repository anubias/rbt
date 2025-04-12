//! The Player trait must be implemented by every player

use super::{action::Action, context::Context, orientation::Orientation};

pub type Avatar = char;
pub type PlayerId = u8;

/// An invalid player details instance
pub const INVALID_PLAYER: Details = Details {
    avatar: ' ',
    alive: false,
    id: 0,
    orientation: Orientation::North,
};

/// Public trait that players need to implement, in order for the game engine
/// to be able to interact with the player.
pub trait Player: Send {
    /// Implement this method if and only if you need to perform expensive and
    /// potentially failing initialization.
    ///
    /// The return value should indicate the initialization success.
    fn initialized(&mut self) -> bool {
        true
    }

    /// This is the player's turn to fight.
    ///
    /// The changes performed by the game engine are provided in the `context`.
    fn act(&mut self, context: Context) -> Action;

    /// Returns the player's name
    fn name(&self) -> String;

    /// This indicates whether the player is ready for battle or not.
    fn is_ready(&self) -> bool {
        false
    }
}

/// Defines the player id type
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Details {
    pub avatar: Avatar,
    pub alive: bool,
    pub id: PlayerId,
    pub orientation: Orientation,
}

impl Details {
    pub fn new(avatar: Avatar, id: PlayerId) -> Self {
        if id == 0 {
            panic!("Invalid player id=0 used!");
        }

        Self {
            avatar,
            alive: true,
            id,
            orientation: Orientation::default(),
        }
    }
}

impl std::fmt::Display for Details {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{id: {}, avatar: {}}}", self.id, self.avatar)
    }
}
