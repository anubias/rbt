use crate::{
    utils::{Action, Direction, Orientation, Player, Position, Rotation},
    world::WorldSize,
};

pub struct Actor<'a> {
    player: &'a mut dyn Player,
    context: ActorContext,
    _health: u8,
    _score: u32,
}

impl<'a> Actor<'a> {
    pub fn new(player: &'a mut dyn Player, context: ActorContext) -> Self {
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
            Action::Move(direction) => self.context.reposition(direction),
            Action::Rotate(rotation) => self.context.rotate(rotation),
            Action::Scan(_scan_type) => {}
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
