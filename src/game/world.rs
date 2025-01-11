use rand::{rngs::ThreadRng, thread_rng, Rng};

use super::{
    types::{Orientation, Position},
    user::{Context, User},
};
use crate::players::player::Player;

const MAX_SIZE: usize = 1000;
const MAX_USABLE_SPACE_PERCENTAGE: f32 = 90.0;
const MAX_MOUNTAIN_SIZE_PERCENTAGE: f32 = 2.0;

pub struct World {
    rng: ThreadRng,
    size: WorldSize,
    map: [[Cell; MAX_SIZE]; MAX_SIZE],
}

impl<'a> World {
    pub fn new(size: WorldSize) -> Self {
        if size.x > MAX_SIZE || size.y > MAX_SIZE {
            panic!("\nWorld size {size} is too big! Maximum accepted size is {MAX_SIZE}\n\n");
        }

        let mut result = Self {
            rng: thread_rng(),
            size,
            map: [[Cell::Field; MAX_SIZE]; MAX_SIZE],
        };

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

    pub fn spawn_user(&mut self, player: &'a mut dyn Player) -> User<'a> {
        let context = Context {
            position: self.get_random_field_location(),
            orientation: Orientation::North,
            _world_size: self.size.clone(),
        };
        let pos = &context.position;
        self.map[pos.x][pos.y] = Cell::Player;

        User::new(player, context)
    }
}

// Private functions
impl<'a> World {
    fn generate_obstacle(&mut self, obstacle: Cell) {
        let range = (self.size.x * self.size.y) as f32 * MAX_MOUNTAIN_SIZE_PERCENTAGE / 100.0;
        let mountain_size = self.rng.gen_range(0..range as u32);

        let mut old_pos: Option<Position> = None;
        for _ in 0..mountain_size {
            if self.get_free_count() > 0 {
                let new_pos = if let Some(p) = old_pos.as_ref() {
                    self.get_connected_field_location(p.clone())
                } else {
                    self.get_random_field_location()
                };

                self.map[new_pos.x][new_pos.y] = obstacle;
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
}

impl std::fmt::Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "World size: {}", self.size)?;
        let mut line = String::new();
        for _ in 0..self.size.y {
            line = format!("{line}\u{2550}");
        }
        writeln!(f, "\u{2554}{line}\u{2557}")?;

        for i in 0..self.size.x {
            line = String::new();
            for j in 0..self.size.y {
                let c = match self.map[i][j] {
                    Cell::Field => " ",
                    Cell::Lake => "~",
                    Cell::Mountain => "^",
                    Cell::Player => "T",
                };
                line = format!("{line}{c}");
            }

            writeln!(f, "\u{2551}{line}\u{2551}")?;
        }

        line = String::new();
        for _ in 0..self.size.y {
            line = format!("{line}\u{2550}");
        }
        writeln!(f, "\u{255A}{line}\u{255D}")
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

#[derive(Clone, Copy)]
enum Cell {
    Field,
    Lake,
    Mountain,
    Player,
}
