mod evaluator;
mod utils;

use crate::api::{
    self, action::Action, aiming::Aiming, context::Context, direction::Direction, map_cell,
    orientation::Orientation, player::Player, scan::ScanType,
};

use crate::api::position::Position;
use crate::api::position::SCANNING_DISTANCE;
use crate::api::rotation::Rotation;
use evaluator::Evaluator;

pub struct Miklas {
    discovered_map: Vec<map_cell::MapCell>,
}

impl Miklas {
    pub fn new() -> Self {
        Self {
            discovered_map: vec![],
        }
    }

    fn append_move(&self, context: &Context, moves: &mut Vec<(Action, i64)>) {
        let self_pos = context.position();
        let orientation = context.player_details().orientation;
        let world_size = context.world_size();

        // Forward
        if let Some(pos) = self_pos.follow(&orientation, world_size) {
            let eval = Evaluator::evaluate_position(world_size, &pos);
            moves.push((Action::Move(Direction::Forward), eval));
        }
        // Backward
        if let Some(pos) = self_pos.follow(&orientation.opposite(), world_size) {
            let eval = Evaluator::evaluate_position(world_size, &pos);
            moves.push((Action::Move(Direction::Backward), eval));
        }
    }

    fn append_rotate(&self, context: &Context, moves: &mut Vec<(Action, i64)>) {
        let self_pos = context.position();
        let orientation = context.player_details().orientation;
        let world_size = context.world_size();

        let best_adjacent_position = self_pos
            .list_adjacent_positions(world_size)
            .into_iter()
            .map(|pos| (pos.clone(), Evaluator::evaluate_position(world_size, &pos)))
            .max_by_key(|(_, eval)| *eval);

        if let Some((pos, pos_eval)) = best_adjacent_position {
            let target_orientation = self_pos.get_orientation_to_pos(&pos);
            let (rot, rot_turns) = orientation.quick_turn(&target_orientation);
            let turn_eval = pos_eval - ((rot_turns * 10) as i64);
            moves.push((Action::Rotate(rot), turn_eval));
        }
    }

    fn choose_action(&self, context: &Context) -> Action {
        use rand::distributions::{Distribution, WeightedIndex};

        // Fetch all possible actions and assign a weight for each one
        let mut moves: Vec<(Action, i64)> = vec![];
        self.append_move(context, &mut moves);
        self.append_rotate(context, &mut moves);
        moves.sort_by_key(|&(_, weight)| weight);

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

        let dist = WeightedIndex::new(&weights).unwrap();
        let mut rng = rand::thread_rng();

        let select_index = dist.sample(&mut rng);
        println!("{:?}\n\nchose{}", moves, select_index);
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

        Action::Scan(ScanType::Mono(Orientation::East));
        self.choose_action(&context)
    }

    fn name(&self) -> String {
        "Miklas".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
