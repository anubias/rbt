use std::collections::HashMap;

use rand::{rngs::ThreadRng, thread_rng, Rng};

use super::{
    types::{Orientation, Position},
    user::{Context, Request, User},
};
use crate::players::player::Player;

const MAX_SIZE: usize = 100;
const MAX_USABLE_SPACE_PERCENTAGE: f32 = 90.0;
const MAX_OBSTACLE_SIZE_PERCENTAGE: f32 = 2.0;

const DAMAGE_COLLISION_WITH_LAKE: u8 = 100;
const DAMAGE_COLLISION_WITH_MOUNTAIN: u8 = 25;
const DAMAGE_COLLISION_WITH_PLAYER: u8 = 10;

pub struct World<'a> {
    rng: ThreadRng,
    size: WorldSize,
    users: HashMap<u8, User<'a>>,
    map: [[Cell; MAX_SIZE]; MAX_SIZE],
}

impl<'a> World<'a> {
    pub fn new(size: WorldSize) -> Self {
        if size.x > MAX_SIZE || size.y > MAX_SIZE {
            panic!("\nWorld size {size} is too big! Maximum accepted size is {MAX_SIZE}\n\n");
        }

        let mut result = Self {
            rng: thread_rng(),
            size: size.clone(),
            users: HashMap::new(),
            map: [[Cell::Field; MAX_SIZE]; MAX_SIZE],
        };

        for i in 0..size.x {
            for j in 0..size.y {
                if i == 0 || j == 0 || i == size.x - 1 || j == size.y - 1 {
                    result.map[i][j] = Cell::Swamp;
                }
            }
        }

        loop {
            result.generate_obstacle(Cell::Mountain);
            result.generate_obstacle(Cell::Lake);

            let get_usable_space_percentage = result.get_usable_space_percentage();
            if get_usable_space_percentage < MAX_USABLE_SPACE_PERCENTAGE {
                break;
            }
        }

        result
    }

    pub fn new_turn(&mut self) {
        let mut requests = Vec::new();

        for (id, user) in self.users.iter_mut() {
            if user.player.is_ready() {
                let user_request = user.act();
                if let Some(request) = user_request {
                    requests.push((*id, request));
                }
            }
        }

        self.process_user_requests(requests)
    }

    pub fn is_location_free(&self, pos: &Position) -> bool {
        match self.map[pos.x][pos.y] {
            Cell::Field => true,
            _ => false,
        }
    }

    pub fn get_random_field_location(&mut self) -> Position {
        loop {
            let x = self.rng.gen_range(0..self.size.x);
            let y = self.rng.gen_range(0..self.size.y);
            let pos = Position { x, y };

            if self.is_location_free(&pos) {
                break pos;
            }
        }
    }

    pub fn spawn_user(&mut self, player: &'a mut dyn Player) {
        let user_id = (self.users.len() + 1) as u8;
        let context = Context {
            health: 100,
            mobile: true,
            position: self.get_random_field_location(),
            orientation: Orientation::default(),
            world_size: self.size.clone(),
        };
        let previous = self.try_set_value_on_map(&context.position, Cell::Player(user_id));

        match previous {
            Cell::Field => {
                self.users.insert(user_id, User::new(player, context));
            }
            _ => {}
        }
    }
}

// Private functions
impl<'a> World<'a> {
    fn process_user_requests(&mut self, requests: Vec<(u8, Request)>) {
        for (user_id, request) in requests.iter() {
            match request {
                Request::Move(from, to) => self.move_user(user_id, from, to),
            }
        }

        for (user_id, _request) in requests {
            let mut pos_opt = None;

            if let Some(user) = self.users.get_mut(&user_id) {
                if user.context.health == 0 {
                    pos_opt = Some(user.context.position.clone());
                }
            }

            if let Some(position) = pos_opt {
                if self.is_user_at_position(user_id, &position) {
                    self.try_set_value_on_map(&position, Cell::Field);
                }
            }
        }
    }

    fn move_user(&mut self, user_id: &u8, from: &Position, to: &Position) {
        let previous = self.try_set_value_on_map(to, Cell::Player(*user_id));
        let user = self.users.get_mut(user_id);

        if let Some(u) = user {
            if u.context.mobile {
                match previous {
                    Cell::Field => {
                        u.context.position = to.clone();
                        self.try_set_value_on_map(from, Cell::Field);
                    }
                    Cell::Lake => {
                        u.context.damage(DAMAGE_COLLISION_WITH_LAKE); // player drowns
                    }
                    Cell::Mountain => {
                        u.context.damage(DAMAGE_COLLISION_WITH_MOUNTAIN);

                        // revert user_move, as it cannot be done
                        self.try_set_value_on_map(to, previous);
                    }
                    Cell::Player(other_user_id) => {
                        u.context.damage(DAMAGE_COLLISION_WITH_PLAYER);

                        // revert user_move, as it cannot be done
                        self.try_set_value_on_map(to, previous);

                        // inflict damage to other use as well
                        if let Some(other_user) = self.users.get_mut(&other_user_id) {
                            other_user.context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                        }
                    }
                    Cell::Swamp => {
                        u.context.mobile = false;
                        u.context.position = to.clone();
                        self.try_set_value_on_map(from, Cell::Field);
                    }
                }
            }
        } else {
            self.try_set_value_on_map(to, previous);
        }
    }

    fn generate_obstacle(&mut self, obstacle: Cell) {
        let range = (self.size.x * self.size.y) as f32 * MAX_OBSTACLE_SIZE_PERCENTAGE / 100.0;
        let mountain_size = self.rng.gen_range(0..range as usize);

        let mut old_pos: Option<Position> = None;
        for _ in 0..mountain_size {
            if self.get_free_count() > 0 {
                let new_pos = if let Some(p) = old_pos.as_ref() {
                    self.get_connected_field_location(p.clone())
                } else {
                    self.get_random_field_location()
                };

                self.try_set_value_on_map(&new_pos, obstacle);

                if old_pos.is_none() {
                    old_pos = Some(new_pos);
                }
            }
        }
    }

    fn get_free_count(&self) -> usize {
        let mut free_count = 0;
        for i in 0..self.size.x {
            for j in 0..self.size.y {
                free_count += match self.map[i][j] {
                    Cell::Field => 1,
                    _ => 0,
                }
            }
        }

        free_count
    }

    fn get_connected_field_location(&mut self, mut position: Position) -> Position {
        loop {
            let o = self
                .rng
                .gen_range(0..Orientation::get_cardinal_direction_count());

            let orientation = Orientation::from(o);
            match orientation {
                Orientation::North | Orientation::East | Orientation::South | Orientation::West => {
                    let candidate = position.follow(&orientation, &self.size);
                    if let Some(p) = candidate {
                        position = p;

                        if self.is_location_free(&position) {
                            break position;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn get_usable_space_percentage(&self) -> f32 {
        100.0f32 * self.get_free_count() as f32 / (self.size.x * self.size.y) as f32
    }

    fn is_user_at_position(&self, user_id: u8, position: &Position) -> bool {
        match self.map[position.x][position.y] {
            Cell::Player(id) => user_id == id,
            _ => false,
        }
    }

    fn get_value_from_map(&self, position: &Position) -> Cell {
        self.map[position.x][position.y]
    }

    fn try_set_value_on_map(&mut self, position: &Position, value: Cell) -> Cell {
        let previous_value = self.get_value_from_map(position);

        match previous_value {
            Cell::Field | Cell::Player(_) | Cell::Swamp => {
                self.map[position.x][position.y] = value;
            }
            _ => {}
        }

        previous_value
    }
}

impl<'a> std::fmt::Display for World<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.size.x {
            let mut line = String::new();
            for j in 0..self.size.y {
                let c = match self.map[i][j] {
                    Cell::Field => " ",
                    Cell::Lake => "~",
                    Cell::Mountain => "^",
                    Cell::Player(_) => "T",
                    Cell::Swamp => ",",
                };
                line = format!("{line}{c}");
            }

            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct WorldSize {
    pub x: usize,
    pub y: usize,
}

impl std::fmt::Display for WorldSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug)]
enum Cell {
    Field,
    Lake,
    Mountain,
    Player(u8),
    Swamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_field_location() {
        let mut world = World::new(WorldSize { x: 100, y: 100 });

        for _ in 0..10_000 {
            let location = world.get_random_field_location();
            assert!(world.is_location_free(&location));
        }
    }
}
