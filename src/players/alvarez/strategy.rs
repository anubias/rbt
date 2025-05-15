use super::model::*;
use super::scanner::*;
use super::tanks::TankInfo;
use super::types::*;
use std::collections::{HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Situation {
    UnderAttack,
    InDanger,
    EnemyNearby,
    AtChokePoint,
    Exploring,
    Safe,
}

fn orientation_between(from: &Position, to: &Position) -> Option<Orientation> {
    let dx = to.x as isize - from.x as isize;
    let dy = to.y as isize - from.y as isize;

    match (dx, dy) {
        (0, -1) => Some(Orientation::North),
        (1, -1) => Some(Orientation::NorthEast),
        (1, 0) => Some(Orientation::East),
        (1, 1) => Some(Orientation::SouthEast),
        (0, 1) => Some(Orientation::South),
        (-1, 1) => Some(Orientation::SouthWest),
        (-1, 0) => Some(Orientation::West),
        (-1, -1) => Some(Orientation::NorthWest),
        _ => None,
    }
}

pub trait Strategy: Send + Sync {
    fn decide_action(&self, world: &WorldModel, context: &Context) -> Action;

    fn decide_action_with_plan(
        &mut self,
        world: &WorldModel,
        context: &Context,
    ) -> (Action, Vec<Action>);

    fn find_safe_direction(
        &self,
        world: &WorldModel,
        context: &Context,
    ) -> Option<Vec<Action>> {
        let action = world.map.get_best_movement_action(context);
        match action {
            Action::Move(_) | Action::Rotate(_) => Some(vec![action]),
            _ => None
        }
    }

    fn get_next_scan_orientation(&self) -> Orientation {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CURRENT_SCAN_INDEX: AtomicUsize = AtomicUsize::new(0);

        let index = CURRENT_SCAN_INDEX.fetch_add(1, Ordering::SeqCst)
            % Orientation::get_cardinal_direction_count();
        index.into()
    }
}

pub struct CombatStrategy {}

impl CombatStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Strategy for CombatStrategy {
    fn decide_action_with_plan(
        &mut self,
        world: &WorldModel,
        ctx: &Context,
    ) -> (Action, Vec<Action>) {
        let my_pos = ctx.position();
        let current_turn = ctx.turn();
        let tile = &world.map.peek_terrain()[my_pos.y][my_pos.x];

        // Being under attack? Retreat
        if tile.last_damage_turn == ctx.turn() {
            if let Some(safe_actions) = self.find_emergency_evasion(world, ctx) {
                if !safe_actions.is_empty() {
                    return (safe_actions[0].clone(), safe_actions[1..].to_vec());
                }
            }
        }

        // If enemies onsight, shoot, shoot, move
        let enemies = world.tanks.enemies_found(current_turn);
        if enemies.len() > 0 {
            if let Some(fire_action) = self.find_best_shot(my_pos, &enemies, current_turn) {
                let mut plan = Vec::new();
                plan.push(fire_action.clone());
                if let Some(position_actions) = self.find_tactical_position(
                    world,
                    ctx,
                    &enemies
                ) {
                    plan.extend(position_actions);
                } else {
                    if let Some(mut safe_actions) = self.find_safe_direction(world, ctx) {
                        if !safe_actions.is_empty() {
                            plan.push(safe_actions.remove(0));
                        }
                    } else {
                        plan.push(world.map.get_best_movement_action(ctx));
                    }
                }

                return (fire_action, plan);
            }
        }

        // Try to find a path to the nearest unexplored cell
        if let Some(actions) = world.map.actions_to_nearest_unexplored(ctx) {
            let first_action = actions[0].clone();
            let rest_actions = actions[1..].to_vec();
            return (first_action, rest_actions);
        }
        // FIXME: Missing actions history to avoid too many moves without a scan
        if let Some(safe_actions) = self.find_safe_direction(world, ctx) {
            if !safe_actions.is_empty() {
                let first_action = safe_actions[0].clone();
                let rest_actions = safe_actions[1..].to_vec();
                return (first_action, rest_actions);
            }
        }

        // If nothing else, scan in the direction of last known enemy
        let best_movement_action = world.map.get_best_movement_action(ctx);
        // add scan 2 nearest cardinals to the player orientation
        let mut scan_actions = Vec::new();
        for orientation in Orientation::cardinals() {
            scan_actions.push(Action::Scan(ScanType::Mono(orientation)));
        }
        return (best_movement_action, Vec::new());
    }

    fn decide_action(&self, world: &WorldModel, context: &Context) -> Action {
        let my_pos = context.position();
        let enemies = &world.tanks.enemies_found(context.turn());
        let current_turn = context.turn();

        // Reuse find_best_shot instead of duplicating the targeting logic
        if let Some(fire_action) = self.find_best_shot(my_pos, enemies, current_turn) {
            return fire_action;
        }

        // No shot available, fallback to randomized scan
        Action::Scan(ScanType::Mono(self.get_next_scan_orientation()))
    }
}

// Helper methods for CombatStrategy
impl CombatStrategy {
    fn find_best_shot(
        &self,
        my_pos: &Position,
        enemies: &[TankInfo],
        current_turn: usize,
    ) -> Option<Action> {
        let positional_targets: Vec<&TankInfo> = enemies
            .iter()
            .filter(|e| {
                if let Some((pos, _, turn)) = e.known_positions.back() {
                    if *turn == current_turn && my_pos.could_hit_positionally(pos) {
                        return true;
                    }
                }
                false
            })
            .collect();

        if !positional_targets.is_empty() {
            if let Some(closest) = positional_targets.iter().min_by_key(|e| {
                let (_, dist, _) = e.known_positions.back().unwrap();
                *dist as usize
            }) {
                let (pos, _, _) = closest.known_positions.back().unwrap();
                return Some(Action::Fire(Aiming::Positional(pos.clone())));
            }
        }

        let cardinal_targets: Vec<&TankInfo> = enemies
            .iter()
            .filter(|e| {
                if let Some((pos, _, turn)) = e.known_positions.back() {
                    if *turn == current_turn {
                        return e.find_alignment(my_pos, pos).is_some();
                    }
                }
                false
            })
            .collect();

        if !cardinal_targets.is_empty() {
            if let Some(closest) = cardinal_targets.iter().min_by_key(|e| {
                let (_, dist, _) = e.known_positions.back().unwrap();
                *dist as usize
            }) {
                let (pos, _, _) = closest.known_positions.back().unwrap();
                if let Some(alignment) = closest.find_alignment(my_pos, pos) {
                    return Some(Action::Fire(Aiming::Cardinal(alignment)));
                }
            }
        }

        None
    }

    fn find_emergency_evasion(
        &self,
        world: &WorldModel,
        ctx: &Context,
    ) -> Option<Vec<Action>> {
        let enemies = world.tanks.enemies_found(ctx.turn());
        let my_pos = ctx.position();

        let mut danger_positions = HashSet::new();
        for enemy in &enemies {
            if let Some((pos, _, _)) = enemy.known_positions.back() {
                // Direct line of fire
                self.add_fire_line_positions(&mut danger_positions, pos, &my_pos, ctx.world_size());

                // Adjacent lines for if enemy rotates
                for orientation in &[
                    enemy.orientation.rotated_clockwise(),
                    enemy.orientation.rotated_counter_clockwise()
                ] {
                    self.add_potential_fire_line(&mut danger_positions, pos, orientation, ctx.world_size());
                }
            }
        }

        // Directions away from danger
        let mut best_actions = Vec::new();
        let mut best_score = f32::MAX;

        for &orientation in &Orientation::all() {
            if let Some(next_pos) = my_pos.follow(&orientation, ctx.world_size()) {
                let tile = &world.map.peek_terrain()[next_pos.y][next_pos.x];
                if !tile.is_transitable() || tile.danger_score > 0.4 {
                    continue;
                }

                if danger_positions.contains(&next_pos) {
                    continue;
                }

                let score = tile.danger_score;
                if score < best_score {
                    best_score = score;
                    best_actions = self.actions_to_position(ctx, &orientation);
                }
            }
        }

        if !best_actions.is_empty() {
            return Some(best_actions);
        }
        None
    }

        fn add_fire_line_positions(
        &self,
        danger_set: &mut HashSet<Position>,
        enemy_pos: &Position,
        my_pos: &Position,
        world_size: &WorldSize
    ) {
        if let Some(orientation) = orientation_between(enemy_pos, my_pos) {
            self.add_potential_fire_line(danger_set, enemy_pos, &orientation, world_size);
        }
    }

    fn add_potential_fire_line(
        &self,
        danger_set: &mut HashSet<Position>,
        start_pos: &Position,
        orientation: &Orientation,
        world_size: &WorldSize
    ) {
        let mut current_pos = start_pos.clone();
        for _ in 0..8 { // Check reasonable firing distance
            if let Some(next_pos) = current_pos.follow(orientation, world_size) {
                danger_set.insert(next_pos.clone());
                current_pos = next_pos;
            } else {
                break;
            }
        }
    }

    fn actions_to_position(
        &self,
        ctx: &Context,
        target_orientation: &Orientation
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let current_orientation = ctx.player_details().orientation;

        if &current_orientation != target_orientation {
            let (rotation, steps) = current_orientation.quick_turn(target_orientation);
            for _ in 0..steps {
                actions.push(Action::Rotate(rotation.clone()));
            }
        }

        actions.push(Action::Move(Direction::Forward));
        actions
    }

    fn find_tactical_position(
        &self,
        world: &WorldModel,
        ctx: &Context,
        enemies: &[TankInfo]
    ) -> Option<Vec<Action>> {
        if enemies.is_empty() {
            return None;
        }

        let my_pos = ctx.position();
        let world_size = ctx.world_size();

        let enemy_positions: Vec<Position> = enemies
            .iter()
            .filter_map(|e| e.known_positions.back().map(|(pos, _, _)| pos.clone()))
            .collect();

        let mut best_orientation = None;
        let mut best_score = f32::MIN;

        for &orientation in &Orientation::all() {
            if let Some(next_pos) = my_pos.follow(&orientation, &world_size) {
                let tile = &world.map.peek_terrain()[next_pos.y][next_pos.x];
                // Lower danger threshold for better safety
                if !tile.is_transitable() || tile.danger_score > 0.4 {
                    continue;
                }

                let mut score = tile.strategic_value * 2.0 - tile.danger_score * 5.0;

                for enemy_pos in &enemy_positions {
                    // Can we hit them from this position?
                    if next_pos.could_hit_positionally(enemy_pos) {
                        score += 5.0;
                    }

                    let dist = next_pos.pythagorean_distance(enemy_pos);

                    if dist >= 4.0 && dist <= 8.0 {
                        score += 4.0;
                    } else if dist < 3.0 {
                        score -= 6.0; // Much bigger penalty for being too close
                    }

                    if enemy_pos.could_hit_positionally(&next_pos) {
                        score -= 8.0;
                    }
                }

                // Cover
                let has_forest_cover = world.map.has_adjacent_forest(&next_pos, &world_size);
                if has_forest_cover {
                    score += 6.0;
                }

                if world.map.is_choke_point(&next_pos) {
                    score -= 7.0;
                }

                if score > best_score {
                    best_score = score;
                    best_orientation = Some(orientation);
                }
            }
        }

        if let Some(target_orientation) = best_orientation {
            return Some(self.actions_to_position(ctx, &target_orientation));
        }

        None
    }
}

pub struct StrategyManager {
    current_strategy: Box<dyn Strategy + Send>,
    model: WorldModel,
    consecutive_moves: usize,
    consecutive_scans: usize,
    action_cache: Vec<Action>,
}

impl StrategyManager {
    pub fn new(model: WorldModel) -> Self {
        Self {
            current_strategy: Box::new(CombatStrategy::new()),
            model: model,
            consecutive_moves: 0,
            consecutive_scans: 0,
            action_cache: Vec::new(),
        }
    }

    pub fn process(&mut self, context: &Context) {
        if context.scanned_data().is_some() {
            self.handle_scan(context);
        }

        // FIXME: Just a default action for now. Running out of time
        self.switch_to(Box::new(CombatStrategy::new()));
        return;

        // let situation = self.assess_context(context);
        // match situation {
        //     Situation::UnderAttack | Situation::InDanger => {
        //         self.switch_to(Box::new(EvasionStrategy::new()));
        //         // Clear cache when in danger - need fresh decisions
        //         self.action_cache.clear();
        //     }
        //     Situation::EnemyNearby => {
        //         if context.max_turns() - context.turn() > 10 {
        //             self.switch_to(Box::new(EvasionStrategy::new()));
        //         } else {
        //             self.switch_to(Box::new(CombatStrategy::new()));
        //         }
        //         // Clear cache when encountering enemies - need fresh decisions
        //         self.action_cache.clear();
        //     }
        //     Situation::AtChokePoint => {
        //         self.switch_to(Box::new(CombatStrategy::new()));
        //     }
        //     Situation::Exploring | Situation::Safe => {
        //         self.switch_to(Box::new(ExploreStrategy::new()));
        //     }
        // }
    }

    pub fn switch_to(&mut self, strategy: Box<dyn Strategy>) {
        self.current_strategy = strategy;
    }

    pub fn decide_action(&mut self, ctx: &Context) -> Action {
        // Maybe first turn
        if self.model.map.is_unexplored(ctx.position()) {
            // Scan all cardinal directions
            let mut scan_actions = Vec::new();
            for direction in Orientation::cardinals() {
                scan_actions.push(Action::Scan(ScanType::Mono(direction)));
            }
            self.action_cache = scan_actions;
            return self.action_cache.pop().unwrap();
        }

        self.model.tanks.update_own_id(ctx.position(), ctx.turn());

        if self.model.tanks.enemies_found(ctx.turn()).is_empty()
            && !self.action_cache.is_empty() {
            let action = self.action_cache.remove(0);
            self.update_consecutive_counters(&action);

            // Override due to limits
            if let Some(override_action) = self.check_action_limits(&action, ctx) {
                self.action_cache.clear(); // Invalidating cache on override
                return override_action;
            }

            return action;
        }

        // No cached actions, ask strategy for action and potential cache
        let (action, future_actions) = self
            .current_strategy
            .decide_action_with_plan(&mut self.model, ctx);

        // Cache a number of future actions and scan, let's see how the prdictions go
        if !future_actions.is_empty() {
            let mut cached_actions = if future_actions.len() <= 2 {
                future_actions.clone()
            } else {
                future_actions[0..2].to_vec()
            };

            let current_orientation = ctx.player_details().orientation;
            let left_orthogonal = current_orientation.rotated_counter_clockwise().rotated_counter_clockwise();
            let right_orthogonal = current_orientation.rotated_clockwise().rotated_clockwise();

            cached_actions.push(Action::Scan(ScanType::Mono(left_orthogonal)));
            cached_actions.push(Action::Scan(ScanType::Mono(right_orthogonal)));
            self.action_cache = cached_actions;
        }

        if let Some(override_action) = self.check_action_limits(&action, ctx) {
                self.action_cache.clear();
                return override_action;
            }

        // Fixme: Remove these cached counters. Is broken
        self.update_consecutive_counters(&action);
        action
    }
}

impl StrategyManager {
    fn handle_scan(&mut self, context: &Context) {
        let mut scan = ScanProces::new(context.player_details().id, context.position().clone());
        scan.process(
            &mut self.model,
            context.scanned_data().as_ref().unwrap(),
            context.turn(),
            context.position(),
        );
    }

    #[allow(dead_code)]
    fn assess_context(&self, ctx: &Context) -> Situation {
        let pos = ctx.position();
        let tile = &self.model.map.peek_terrain()[pos.y][pos.x];

        if tile.last_damage_turn == ctx.turn() {
            return Situation::UnderAttack;
        }

        if tile.danger_score > 0.5 || tile.times_hit > 0 {
            return Situation::InDanger;
        }

        let enemies = self.model.tanks.enemies_found(ctx.turn());
        if !enemies.is_empty() {
            return Situation::EnemyNearby;
        }

        if self.model.map.is_choke_point(pos) {
            return Situation::AtChokePoint;
        }

        if self.model.map.is_unexplored(pos) {
            return Situation::Exploring;
        }

        // Default to safe?
        Situation::Safe
    }

    fn update_consecutive_counters(&mut self, action: &Action) {
        // FIXME: It is broken!!!!
        match action {
            Action::Move(_) | &Action::Rotate(_) => {
                self.consecutive_moves += 1;
                self.consecutive_scans = 0;
            }
            Action::Scan(_) => {
                self.consecutive_scans += 1;
                self.consecutive_moves = 0;
            }
            _ => {
                self.consecutive_moves = 0;
                self.consecutive_scans = 0;
            }
        }
    }

    fn check_action_limits(&mut self, action: &Action, ctx: &Context) -> Option<Action> {
        // Hack hack hack
        match action {
            Action::Move(dir) if self.consecutive_moves > 2 => {
                let scan_orientation = if *dir == Direction::Backward {
                    ctx.player_details().orientation.opposite()
                } else {
                    ctx.player_details().orientation
                };
                self.consecutive_moves = 0;
                self.consecutive_scans = 0;
                Some(Action::Scan(ScanType::Mono(scan_orientation)))
            },
            Action::Rotate(_) if self.consecutive_moves > 2 => {
                self.consecutive_moves = 0;
                self.consecutive_scans = 0;
                Some(Action::Scan(ScanType::Omni))
            },
            Action::Scan(_) if self.consecutive_scans > 1 =>{
                self.consecutive_moves = 0;
                self.consecutive_scans = 0;
                Some(self.current_strategy.decide_action(&self.model, ctx))},
            _ => None,
        }
    }
}
