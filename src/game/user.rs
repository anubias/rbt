use super::{
    types::{Direction, Orientation, Position, Rotation},
    world::WorldSize,
};
use crate::players::player::{Action, Player};

pub struct User<'a> {
    player: &'a mut dyn Player,
    context: Context,
    _health: u8,
    _score: u32,
}

impl<'a> User<'a> {
    pub fn new(player: &'a mut dyn Player, context: Context) -> Self {
        Self {
            player,
            context,
            _health: 100,
            _score: 0,
        }
    }

    pub fn act(&mut self) {
        match self.player.act(&self.context) {
            Action::Idle => {}
            Action::Fire => {}
            Action::Move(direction) => self.context.relocate(direction),
            Action::Rotate(rotation) => self.context.rotate(rotation),
            Action::Scan(_scan_type) => {}
        }
    }

    pub fn ready_for_action(&self) -> bool {
        self.player.is_ready()
    }
}

// Private functions
impl<'a> User<'a> {}

// #[derive(Clone)]
pub struct Context {
    pub position: Position,
    pub orientation: Orientation,
    pub _world_size: WorldSize,
}

impl Context {
    pub fn relocate(&mut self, direction: Direction) {
        match direction {
            Direction::Backward => {}
            Direction::Forward => {}
        }
    }

    pub fn rotate(&mut self, rotation: Rotation) {
        self.orientation = match rotation {
            Rotation::Clockwise => self.orientation.rotate_clockwise(),
            Rotation::CounterClockwise => self.orientation.rotate_counter_clockwise(),
        }
    }
}
