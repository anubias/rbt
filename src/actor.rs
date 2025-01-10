use crate::{
    utils::{Action, Direction, Orientation, Player, Position, Rotation},
    world::WorldSize,
};

pub struct Actor<'a> {
    player: &'a dyn Player,
    context: ActorContext,
    _health: u8,
    _score: u32,
}

impl<'a> Actor<'a> {
    pub fn new(player: &'a dyn Player, context: ActorContext) -> Self {
        Self {
            player,
            context,
            _health: 100,
            _score: 0,
        }
    }

    pub fn act(&mut self) {
        match self.player.act(&self.context) {
            Action::Fire => {}
            Action::_Move(direction) => self.context.reposition(direction),
            Action::_Rotate(rotation) => self.context.rotate(rotation),
            Action::_Scan(_scan_type) => {}
        }
    }

    pub fn ready_for_action(&self) -> bool {
        self.player.is_ready()
    }
}

// #[derive(Clone)]
pub struct ActorContext {
    pub position: Position,
    pub orientation: Orientation,
    pub _world_size: WorldSize,
}

impl ActorContext {
    pub fn reposition(&mut self, direction: Direction) {
        match direction {
            Direction::_Backward => {}
            Direction::_Forward => {}
        }
    }

    pub fn rotate(&mut self, rotation: Rotation) {
        self.orientation = match rotation {
            Rotation::_Clockwise => self.orientation.rotate_clockwise(),
            Rotation::_CounterClockwise => self.orientation.rotate_counter_clockwise(),
        }
    }
}
