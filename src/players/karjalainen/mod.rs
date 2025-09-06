mod utils;

use crate::api::map_cell::MapCell;
use crate::api::scan::ScanResult;
use crate::api::world_size::MAX_WORLD_SIZE;
use crate::api::{
    action::Action, aiming::Aiming, context::Context, direction::Direction, map_cell,
    player::Player, scan::ScanType,
};

use crate::api::position::Position;
use crate::api::position::SCANNING_DISTANCE;
use crate::api::rotation::Rotation;

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

    fn append_shooting(&self, context: &Context, moves: &mut Vec<(Action, i64)>) {
        let players_found = {
            let mut players = vec![];
            // just imagine if the array was a flat 1D array ;-;. iter, reduce, filter, collect.
            for y in 0..MAX_WORLD_SIZE {
                for x in 0..MAX_WORLD_SIZE {
                    if let MapCell::Player(details, _) = self.discovered_map[y][x] {
                        if details.id != context.player_details().id && details.alive {
                            players.push((x, y, self.update_map[y][x]));
                        }
                    }
                }
            }
            players.sort_by_key(|&(_, _, last_updated)| std::usize::MAX - last_updated);
            players
        };
        if let Some((x, y, last_updated)) = players_found.first().cloned() {
            let delta = context.turn() - last_updated;
            let target_pos = Position { x, y };
            if delta <= 5 {
                if context.position().could_hit_positionally(&target_pos) {
                    moves.push((Action::Fire(Aiming::Positional(target_pos)), 1500));
                } else if context.position().could_hit_cardinally(&target_pos) {
                    // println!("card: {}", target_pos);
                    let or = context
                        .position()
                        .get_orientation_to_pos(&target_pos.direction_normalize());
                    moves.push((Action::Fire(Aiming::Cardinal(or)), 1500));
                }
            }
        }
    }

    fn append_move(&self, context: &Context, moves: &mut Vec<(Action, i64)>) {
        let self_pos = context.position();
        let orientation = context.player_details().orientation;
        let world_size = context.world_size();

        // Forward
        if let Some(pos) = self_pos.follow(&orientation, world_size) {
            if (context.turn() - self.update_map[pos.y][pos.x]) > 10 {
                moves.push((Action::Scan(ScanType::Mono(orientation)), 500));
                return;
            }

            match self.discovered_map[pos.y][pos.x] {
                MapCell::Unallocated => {
                    moves.push((Action::Scan(ScanType::Mono(orientation)), 500));
                }
                MapCell::Terrain(terrain) => {
                    if terrain == map_cell::Terrain::Field && (rand::random::<i32>() % 5) != 0 {
                        moves.push((Action::Move(Direction::Forward), 500));
                    } else {
                        moves.push((Action::Rotate(Rotation::Clockwise), 500));
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_scan(&mut self, context: &Context, result: &ScanResult) {
        // let (width, height) = (context.world_size().x, context.world_size().y);

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
        // for y in 0..height {
        //     for x in 0..width {
        //         print!("{}", self.discovered_map[y][x]);
        //     }
        //     print!("\n");
        // }
    }

    fn append_scan(&mut self, context: &Context, _: &mut Vec<(Action, i64)>) {
        if let Some(result) = context.scanned_data() {
            self.parse_scan(context, result);
        }
    }

    fn choose_action(&mut self, context: &Context) -> Action {
        use rand::distr::{Distribution, weighted::WeightedIndex};

        // Fetch all possible actions and assign a weight for each one
        let mut moves: Vec<(Action, i64)> = vec![];
        self.append_move(context, &mut moves);
        self.append_scan(context, &mut moves);
        self.append_shooting(context, &mut moves);
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
        let mut rng = rand::rng();

        let select_index = dist.sample(&mut rng);
        // println!(
        //     "{:?}\n\nchose{} orientation {}",
        //     moves,
        //     select_index,
        //     context.player_details().orientation
        // );
        top_three[select_index].0.clone()
    }
}

impl Player for Miklas {
    fn act(&mut self, context: Context) -> Action {
        self.choose_action(&context)
    }

    fn name(&self) -> String {
        "Miklas".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
