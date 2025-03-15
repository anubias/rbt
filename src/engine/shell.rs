use crate::api::{
    aiming::Aiming,
    position::{Position, CARDINAL_SHOT_DISTANCE, POSITIONAL_SHOT_DISTANCE},
    world_size::WorldSize,
};

#[derive(Clone, PartialEq, Eq)]
pub enum ShellState {
    NotLaunched,
    Flying,
    Impact,
    Explosion,
    Exploded,
    Spent,
}

#[derive(PartialEq, Eq)]
pub struct Shell {
    current_pos: Option<Position>,
    fired_from: Position,
    aim_type: Aiming,
    state: ShellState,
}

impl Shell {
    pub fn new(aim_type: Aiming, fired_from: Position) -> Self {
        Self {
            current_pos: Some(fired_from.clone()),
            fired_from,
            aim_type,
            state: ShellState::NotLaunched,
        }
    }

    pub fn fired_from(&self) -> Position {
        self.fired_from.clone()
    }

    pub fn pos(&self) -> Option<Position> {
        self.current_pos.clone()
    }

    pub fn state(&self) -> ShellState {
        self.state.clone()
    }

    pub fn possible_shot(&self) -> bool {
        match &self.aim_type {
            Aiming::Cardinal(_) => true,
            Aiming::Positional(pos) => self.fired_from.could_hit_positionally(pos),
        }
    }

    pub fn evolve(&mut self, world_size: &WorldSize) {
        match self.state {
            ShellState::NotLaunched => self.state = ShellState::Flying,
            ShellState::Flying => {
                self.state = ShellState::Flying;
                if let Some(pos) = &self.current_pos {
                    self.current_pos = match &self.aim_type {
                        Aiming::Positional(p) => Some(p.clone()),
                        Aiming::Cardinal(orientation) => pos.follow(orientation, world_size),
                    };
                }
            }
            ShellState::Impact => self.state = ShellState::Explosion,
            ShellState::Explosion => self.state = ShellState::Exploded,
            ShellState::Exploded => self.state = ShellState::Spent,
            ShellState::Spent => {}
        }
    }

    pub fn try_to_land(&mut self) -> bool {
        if self.state == ShellState::Flying {
            if let Some(cur_pos) = &self.current_pos {
                let landed = match &self.aim_type {
                    Aiming::Cardinal(_) => {
                        let (dx, dy) = self.fired_from.manhattan_distance(cur_pos);
                        let (dx, dy) = (dx.unsigned_abs(), dy.unsigned_abs());
                        let max_distance = self.max_fly_distance();

                        dx >= max_distance || dy >= max_distance
                    }
                    Aiming::Positional(position) => *position == *cur_pos,
                };
                if landed {
                    self.impact();
                }
                landed
            } else {
                // no current position
                false
            }
        } else {
            // not flying
            false
        }
    }

    pub fn impact(&mut self) {
        if self.state == ShellState::Flying {
            self.state = ShellState::Impact;
        }
    }

    pub fn max_fly_distance(&self) -> usize {
        match self.aim_type {
            Aiming::Cardinal(_) => CARDINAL_SHOT_DISTANCE,
            Aiming::Positional(_) => POSITIONAL_SHOT_DISTANCE,
        }
    }
}
