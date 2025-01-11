use super::{
    types::{Direction, Orientation, Position, Rotation},
    world::WorldSize,
};
use crate::players::player::{Action, Player};

pub struct User<'a> {
    pub player: &'a mut dyn Player,
    pub context: Context,
}

impl<'a> User<'a> {
    pub fn new(player: &'a mut dyn Player, context: Context) -> Self {
        Self { player, context }
    }

    pub fn act(&mut self) -> Option<Request> {
        if self.context.health > 0 {
            let player_action = self.player.act(&self.context);
            self.process_player_actions(player_action)
        } else {
            None
        }
    }
}

// Private functions
impl<'a> User<'a> {
    fn process_player_actions(&self, player_action: Action) -> Option<Request> {
        match player_action {
            Action::Idle | Action::Fire | Action::Scan(_) => None,
            Action::Move(direction) => {
                let (from, to) = self.route(direction);
                Some(Request::Move(from, to))
            }
            Action::Rotate(_rotation) => {
                // self.context.rotate(rotation);
                None
            }
        }
    }

    fn route(&self, direction: Direction) -> (Position, Position) {
        let orientation = match direction {
            Direction::Backward => self.context.orientation.opposite(),
            Direction::Forward => self.context.orientation.clone(),
        };

        let next_position = self
            .context
            .position
            .follow(&orientation, &self.context.world_size);

        let new_position = if let Some(position) = next_position {
            position
        } else {
            self.context.position.clone()
        };

        (self.context.position.clone(), new_position)
    }
}

pub enum Request {
    Move(Position, Position),
}

// #[derive(Clone)]
pub struct Context {
    pub health: u8,
    pub mobile: bool,
    pub position: Position,
    pub orientation: Orientation,
    pub world_size: WorldSize,
}

impl Context {
    pub fn rotate(&mut self, rotation: Rotation) {
        self.orientation = match rotation {
            Rotation::Clockwise => self.orientation.rotate_clockwise(),
            Rotation::CounterClockwise => self.orientation.rotate_counter_clockwise(),
        }
    }

    pub fn damage(&mut self, damage: u8) {
        self.health -= self.health.min(damage);
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{health={}, mobile={}, position={}, orientation=\"{}\"}}",
            self.health, self.mobile, self.position, self.orientation
        )
    }
}
