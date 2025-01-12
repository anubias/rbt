use std::collections::HashMap;

use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::players::player::{
    Action, Context, Direction, Orientation, Player, Position, Rotation, WorldSize,
};

const MAX_SIZE: usize = 100;
const MAX_USABLE_SPACE_PERCENTAGE: f32 = 90.0;
const MAX_OBSTACLE_SIZE_PERCENTAGE: f32 = 2.0;

const DAMAGE_COLLISION_WITH_LAKE: u8 = 100;
const DAMAGE_COLLISION_WITH_MOUNTAIN: u8 = 25;
const DAMAGE_COLLISION_WITH_PLAYER: u8 = 10;

type UserId = u8;
type User = (Box<dyn Player>, Context);

pub struct World {
    rng: ThreadRng,
    size: WorldSize,
    players: HashMap<UserId, User>,
    map: [[Cell; MAX_SIZE]; MAX_SIZE],
}

impl World {
    pub fn new(size: WorldSize) -> Self {
        if size.x > MAX_SIZE || size.y > MAX_SIZE {
            panic!("\nWorld size {size} is too big! Maximum accepted size is {MAX_SIZE}\n\n");
        }

        let mut result = Self {
            rng: thread_rng(),
            size: size.clone(),
            players: HashMap::new(),
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
            result.generate_obstacle(Cell::Swamp);

            let get_usable_space_percentage = result.get_usable_space_percentage();
            if get_usable_space_percentage < MAX_USABLE_SPACE_PERCENTAGE {
                break;
            }
        }

        result
    }

    pub fn new_turn(&mut self) {
        let mut actions = Vec::new();

        for (id, (player, context)) in self.players.iter_mut() {
            if player.is_ready() && context.health() > 0 {
                let action = player.act(&context);
                actions.push((*id, action));
            }
        }

        self.process_player_actions(actions)
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

    pub fn spawn_player(&mut self, player: Box<dyn Player>) {
        if !player.is_ready() {
            // won't spawn pussy-players
            return;
        }

        let player_id = (self.players.len() + 1) as u8;
        let context = Context::new(self.get_random_field_location(), self.size.clone());
        let previous = self.try_set_value_on_map(context.position(), Cell::Player(player_id));
        match previous {
            Cell::Field => {
                self.players.insert(player_id, (player, context));
            }
            _ => {}
        }
    }
}

// Private functions
impl World {
    fn process_player_actions(&mut self, actions: Vec<(u8, Action)>) {
        for (player_id, action) in actions.iter() {
            if let Some((_, context)) = self.players.get(player_id) {
                match action {
                    Action::Move(direction) => {
                        let (from, to) = Self::compute_route(
                            context.position(),
                            context.orientation(),
                            direction,
                            context.world_size(),
                        );
                        self.move_player(*player_id, &from, &to);
                    }
                    Action::Rotate(rotation) => {
                        self.rotate_player(*player_id, rotation);
                    }
                    _ => {}
                }
            }
        }

        for (player_id, _) in actions {
            let mut pos_opt = None;

            if let Some((_, context)) = self.players.get_mut(&player_id) {
                if context.health() == 0 {
                    pos_opt = Some(context.position().clone());
                }
            }

            if let Some(position) = pos_opt {
                if self.is_player_at_position(player_id, &position) {
                    self.try_set_value_on_map(&position, Cell::Field);
                }
            }
        }
    }

    fn move_player(&mut self, player_id: u8, from: &Position, to: &Position) {
        let walk_on = self.try_set_value_on_map(&to, Cell::Player(player_id));
        let mut successfully_moved = false;

        if let Some((_, context)) = self.players.get_mut(&player_id) {
            if context.is_mobile() {
                match walk_on {
                    Cell::Field => {
                        successfully_moved = context.relocate(to.clone());
                    }
                    Cell::Lake => {
                        context.damage(DAMAGE_COLLISION_WITH_LAKE); // player drowns
                        successfully_moved = context.relocate(to.clone());
                    }
                    Cell::Mountain => {
                        context.damage(DAMAGE_COLLISION_WITH_MOUNTAIN);
                    }
                    Cell::Player(other_player_id) => {
                        context.damage(DAMAGE_COLLISION_WITH_PLAYER);

                        // inflict damage to other player as well
                        if let Some((_, other_context)) = self.players.get_mut(&other_player_id) {
                            other_context.damage(DAMAGE_COLLISION_WITH_PLAYER);
                        }
                    }
                    Cell::Swamp => {
                        context.immobilize();
                        successfully_moved = context.relocate(to.clone());
                    }
                }
            }
        }

        if successfully_moved {
            self.try_set_value_on_map(&from, Cell::Field);
        } else {
            self.try_set_value_on_map(&to, walk_on);
        }
    }

    fn rotate_player(&mut self, player_id: u8, rotation: &Rotation) {
        if let Some((_, context)) = self.players.get_mut(&player_id) {
            context.rotate(rotation);
        }
    }

    fn compute_route(
        start_position: &Position,
        orientation: &Orientation,
        direction: &Direction,
        world_size: &WorldSize,
    ) -> (Position, Position) {
        let actual_orientation = match direction {
            Direction::Backward => orientation.opposite(),
            Direction::Forward => orientation.clone(),
        };

        let new_position = if let Some(pos) = start_position.follow(&actual_orientation, world_size)
        {
            pos
        } else {
            start_position.clone()
        };

        (start_position.clone(), new_position)
    }

    fn generate_obstacle(&mut self, obstacle: Cell) {
        let range = (self.size.x * self.size.y) as f32 * MAX_OBSTACLE_SIZE_PERCENTAGE / 100.0;
        let mountain_size = self.rng.gen_range(0..range as usize);

        let mut old_pos: Option<Position> = None;
        for _ in 0..mountain_size {
            if self.get_free_count() > 0 {
                let new_pos = if let Some(p) = old_pos.as_ref() {
                    self.get_adjacent_field_location(p.clone())
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

    fn get_adjacent_field_location(&mut self, mut position: Position) -> Position {
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

    fn is_player_at_position(&self, player_id: u8, position: &Position) -> bool {
        match self.map[position.x][position.y] {
            Cell::Player(id) => player_id == id,
            _ => false,
        }
    }

    fn get_value_from_map(&self, position: &Position) -> Cell {
        self.map[position.x][position.y]
    }

    fn try_set_value_on_map(&mut self, position: &Position, value: Cell) -> Cell {
        let walk_on = self.get_value_from_map(position);

        match walk_on {
            Cell::Field | Cell::Player(_) | Cell::Swamp => {
                self.map[position.x][position.y] = value;
            }
            _ => {}
        }

        walk_on
    }
}

impl std::fmt::Display for World {
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
