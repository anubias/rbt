mod evaluator;
mod utils;

use crate::api::map_cell::MapCell;
use crate::api::player;
use crate::api::scan::ScanResult;
use crate::api::world_size::MAX_WORLD_SIZE;
use crate::api::{
    self, action::Action, aiming::Aiming, context::Context, direction::Direction, map_cell,
    orientation::Orientation, player::Player, scan::ScanType,
};

use crate::api::position::Position;
use crate::api::position::SCANNING_DISTANCE;
use crate::api::rotation::Rotation;
use evaluator::Evaluator;

pub struct Miklas {
    discovered_map: Box<[[MapCell; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>,
    update_map: Box<[[usize; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]>, // which round the tile was last updated.
}

impl Miklas {
    pub fn new() -> Self {
        Self {
            discovered_map: Box::new([[MapCell::Unallocated; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
            update_map: Box::new([[0; MAX_WORLD_SIZE]; MAX_WORLD_SIZE]),
        }
    }

    fn append_move(&self, context: &Context, moves: &mut Vec<(Action, i64)>) {
        let self_pos = context.position();
        let orientation = context.player_details().orientation;
        let world_size = context.world_size();

        // Forward
        if let Some(pos) = self_pos.follow(&orientation, world_size) {
            match self.discovered_map[pos.y][pos.x] {
                MapCell::Unallocated => {
                    moves.push((Action::Scan(ScanType::Mono(orientation)), 500));
                }
                MapCell::Terrain(terrain) => {
                    if terrain == map_cell::Terrain::Field {
                        let eval = Evaluator::evaluate_position(world_size, &pos);
                        moves.push((Action::Move(Direction::Forward), eval));
                    } else {
                        moves.push((Action::Rotate(Rotation::Clockwise), 500));
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_scan(&mut self, context: &Context, result: &ScanResult) {
        let (width, height) = (context.world_size().x, context.world_size().y);

        // find the local player use it as offset
        let player_in_map: (usize, usize) = {
            let mut found = false;
            let mut out = (0, 0);
            for y in 0..SCANNING_DISTANCE {
                if found {
                    break;
                }

                for x in 0..SCANNING_DISTANCE {
                    if let MapCell::Player(details, _) = result.data[y][x] {
                        if details.id == context.player_details().id {
                            out = (x, y);
                            found = true;
                        } else {
                            continue;
                        }
                    }
                }
            }
            out
        };

        let offset = (
            (context.position().x as i32) - (player_in_map.0 as i32),
            (context.position().y as i32) - (player_in_map.1 as i32),
        );

        for y in 0..SCANNING_DISTANCE {
            for x in 0..SCANNING_DISTANCE {
                let map = (offset.0 + x as i32, offset.1 + y as i32);
                if !(0..MAX_WORLD_SIZE).contains(&(map.0 as usize))
                    || !(0..MAX_WORLD_SIZE).contains(&(map.1 as usize))
                {
                    continue;
                }

                self.discovered_map[map.1 as usize][map.0 as usize] = result.data[y][x];
                self.update_map[map.1 as usize][map.0 as usize] = context.turn();
            }
        }

        // print
        for y in 0..height {
            for x in 0..width {
                print!("{}", self.discovered_map[y][x]);
            }
            print!("\n");
        }
    }

    fn append_scan(&mut self, context: &Context, moves: &mut Vec<(Action, i64)>) {
        if let Some(result) = context.scanned_data() {
            self.parse_scan(context, result);
        }
    }

    fn choose_action(&mut self, context: &Context) -> Action {
        use rand::distributions::{Distribution, WeightedIndex};

        // Fetch all possible actions and assign a weight for each one
        let mut moves: Vec<(Action, i64)> = vec![];
        self.append_move(context, &mut moves);
        self.append_scan(context, &mut moves);
        moves.sort_by_key(|&(_, weight)| -weight);

        // Take top-three and randomly choose on by their weight.
        let top_three = moves
            .clone()
            .into_iter()
            .take(3)
            .collect::<Vec<(Action, i64)>>();
        let weights = top_three
            .iter()
            .map(|(_, weight)| *weight)
            .collect::<Vec<i64>>();

        let dist = match WeightedIndex::new(&weights) {
            Ok(some) => some,
            Err(_) => {
                return Action::Scan(ScanType::Omni); // fallback if no actions are available.
            }
        };
        let mut rng = rand::thread_rng();

        let select_index = dist.sample(&mut rng);
        println!(
            "{:?}\n\nchose{} orientation {}",
            moves,
            select_index,
            context.player_details().orientation
        );
        top_three[select_index].0.clone()
    }
}

impl Player for Miklas {
    fn act(&mut self, context: Context) -> Action {
        if let Some(r) = context.scanned_data() {
            println!("r: {}", r);
            println!(
                "pos {}",
                Evaluator::evaluate_position(context.world_size(), context.position())
            )
        }
        self.choose_action(&context)
    }

    fn name(&self) -> String {
        "Miklas".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
