mod magellan;

use magellan::Magellan;

use crate::{
    api::{
        action::Action,
        aiming::Aiming,
        context::Context,
        direction::Direction,
        map_cell::{MapCell, Terrain},
        orientation::Orientation,
        path_finder::{MapReader, PathFinder},
        player::{Details, Player},
        position::{Position, CARDINAL_SHOT_DISTANCE, SCANNING_DISTANCE},
        rotation::Rotation,
        scan::{ScanResult, ScanType},
        world_size::WorldSize,
    },
    terminal::Terminal,
};

const MAX_CONSECUTIVE_FORWARD_SCANS: usize = 4;
const MAX_TURNS_BETWEEN_FORWARD_SCANS: usize = 4;
const MAX_AMBUSH_TURNS: usize = MAX_TURNS_BETWEEN_FORWARD_SCANS * 4;
const MAX_SNEAK_TURNS: usize = MAX_AMBUSH_TURNS * 2;

pub struct Aurelian {
    active_enemy_turn: usize,
    ambush_turn: usize,
    forward_scans_count: usize,
    full_scan_done: bool,
    map_reader: Magellan,
    mono_scans: Vec<Action>,
    previous_enemy_positions: Vec<Position>,
    scan_turn: usize,
    shot_queue: Vec<Action>,
    terminal: Terminal,
    unreachable_positions: Vec<Position>,
    walking_direction: Direction,
    walking_path: Vec<Position>,
}

// Public functions
impl Aurelian {
    pub fn new() -> Self {
        Self {
            active_enemy_turn: 0,
            ambush_turn: 0,
            forward_scans_count: 0,
            full_scan_done: false,
            map_reader: Magellan::new(),
            mono_scans: Vec::new(),
            previous_enemy_positions: Vec::new(),
            scan_turn: 0,
            shot_queue: Vec::new(),
            terminal: Terminal::new(),
            unreachable_positions: Vec::new(),
            walking_direction: Direction::default(),
            walking_path: Vec::new(),
        }
    }
}

// Private functions
impl Aurelian {
    fn ai_logic(&mut self, context: &Context) -> Action {
        let mut intention = self.try_to_shoot(context);

        if intention.is_none() {
            intention = self.try_to_scan(context);
        }
        if intention.is_none() {
            intention = self.try_to_ambush_or_sneak(context);
        }
        if intention.is_none() {
            intention = self.try_to_explore(context);
        }
        if intention.is_none() {
            intention = self.try_to_walk(context);
        }
        if intention.is_none() {
            intention = self.forget_everything();
        }

        let action = match intention {
            Some(a) => a,
            None => Action::default(),
        };

        action
    }

    fn try_to_shoot(&mut self, context: &Context) -> Option<Action> {
        if self.shot_queue.is_empty() {
            let my_position = context.position();

            let positional_enemies: Vec<Position> = self
                .map_reader
                .find_all_enemies(context.player_details().id, context.world_size())
                .into_iter()
                .filter(|p| my_position.could_hit_positionally(p))
                .collect();
            for position in &positional_enemies {
                self.shot_queue.push(Action::Scan(ScanType::Omni));
                self.shot_queue
                    .push(Action::Fire(Aiming::Positional(position.clone())));
                self.shot_queue
                    .push(Action::Fire(Aiming::Positional(position.clone())));
                break;
            }

            if self.shot_queue.is_empty() {
                let cardinal_enemies: Vec<Position> = self
                    .map_reader
                    .find_all_enemies(context.player_details().id, context.world_size())
                    .into_iter()
                    .filter(|p| my_position.could_hit_cardinally(p))
                    .collect();

                if let Some(closest) = self
                    .map_reader
                    .find_closest_position(my_position, &cardinal_enemies)
                {
                    if let Some(cardinal) = self.map_reader.find_alignment(my_position, &closest) {
                        self.shot_queue.push(Action::Scan(ScanType::Mono(cardinal)));
                        self.shot_queue
                            .push(Action::Fire(Aiming::Cardinal(cardinal)));
                    }
                }
            }
        }

        // None or something to shoot at
        self.shot_queue.pop()
    }

    fn try_to_scan(&mut self, context: &Context) -> Option<Action> {
        if context.turn() == 0 {
            Some(Action::Scan(ScanType::Omni))
        } else if self.ambush_turn != 0 {
            None
        } else {
            if self.mono_scans.is_empty() {
                if !self.full_scan_done {
                    self.full_scan_done = true;
                    self.mono_scans = self.get_360_scans();
                } else {
                    let turn_delta = context.turn() - self.scan_turn;
                    if turn_delta >= MAX_TURNS_BETWEEN_FORWARD_SCANS {
                        self.scan_turn = context.turn();

                        let mut moving_orientation = context.player_details().orientation;
                        if self.walking_direction == Direction::Backward {
                            moving_orientation = moving_orientation.opposite();
                        }

                        let actions = self
                            .map_reader
                            .find_forward_scans(moving_orientation)
                            .into_iter()
                            .map(|o| Action::Scan(ScanType::Mono(o)))
                            .collect();

                        let mut backward_scans = self.get_backward_scans(&actions);
                        self.mono_scans = actions;

                        if self.forward_scans_count >= MAX_CONSECUTIVE_FORWARD_SCANS {
                            self.mono_scans.append(&mut backward_scans);
                            self.forward_scans_count = 0;
                        }
                    }
                }
            } else {
                self.walking_path.clear();
                if self.mono_scans.len() == 1 {
                    self.scan_turn = context.turn();
                    self.forward_scans_count += 1;
                }
            }

            self.mono_scans.pop()
        }
    }

    fn try_to_ambush_or_sneak(&mut self, context: &Context) -> Option<Action> {
        if !self.walking_path.is_empty() {
            return None;
        }

        let my_position = context.position();
        let enemy_positions: Vec<Position> = self
            .map_reader
            .find_all_enemies(context.player_details().id, context.world_size())
            .into_iter()
            .filter(|x| my_position.within_distance(x, CARDINAL_SHOT_DISTANCE))
            .collect();

        if enemy_positions.is_empty() {
            // no enemies found
            self.active_enemy_turn = 0;
            self.ambush_turn = 0;
            None
        } else {
            if self.ambush_turn == 0 {
                self.ambush_turn = context.turn();
            }
            let enemy_movement = self.have_enemies_moved(&enemy_positions);
            if enemy_movement && self.active_enemy_turn == 0 {
                self.active_enemy_turn = context.turn();
            }

            let ambush_delta = context.turn() - self.ambush_turn;
            let active_enemy_delta = context.turn() - self.active_enemy_turn;
            let sneaking_decision = (self.active_enemy_turn != 0
                && active_enemy_delta >= MAX_SNEAK_TURNS)
                || (self.active_enemy_turn == 0 && ambush_delta >= MAX_AMBUSH_TURNS);

            if sneaking_decision {
                self.ambush_turn = context.turn();

                let mut firing_positions = Vec::new();
                let world_size = context.world_size();

                for position in enemy_positions {
                    let mut fire_at = self
                        .map_reader
                        .list_safe_firing_positions(&position, world_size)
                        .into_iter()
                        .filter(|x| !firing_positions.contains(x))
                        .collect();
                    firing_positions.append(&mut fire_at);
                }

                if let Some(closest) = self
                    .map_reader
                    .find_closest_position(my_position, &firing_positions)
                {
                    let mut path =
                        PathFinder::new(self.map_reader.clone(), context.world_size().clone())
                            .compute_shortest_path(
                                my_position,
                                &closest,
                                &context.player_details().orientation,
                            )
                            .to_path();
                    if enemy_movement {
                        self.active_enemy_turn = 0;
                        if let Some(next) = path.pop() {
                            self.walking_path.clear();
                            self.walking_path.push(next);
                        }
                    } else {
                        if path.len() <= MAX_TURNS_BETWEEN_FORWARD_SCANS {
                            self.walking_path = path;
                        } else {
                            let at = path.len() - MAX_TURNS_BETWEEN_FORWARD_SCANS;
                            self.walking_path = path.split_off(at);
                        }
                    }
                }

                None
            } else {
                let mut scan_quadrants = Vec::new();
                for position in enemy_positions {
                    let quadrant = self.map_reader.compute_quadrant(my_position, &position);
                    if !scan_quadrants.contains(&quadrant) {
                        scan_quadrants.push(quadrant);
                    }
                }

                if let Some(orientation) = scan_quadrants.pop() {
                    Some(Action::Scan(ScanType::Mono(orientation)))
                } else {
                    None
                }
            }
        }
    }

    fn try_to_explore(&mut self, context: &Context) -> Option<Action> {
        if !self.walking_path.is_empty() {
            return None;
        }

        let unallocated_cells_count = self
            .map_reader
            .count_unallocated_cells(context.world_size());

        loop {
            if self.unreachable_positions.len() >= unallocated_cells_count {
                break None;
            }

            let unexplored = self
                .map_reader
                .list_all_unexplored(&self.unreachable_positions, context.world_size());
            if let Some(destination) = self
                .map_reader
                .find_closest_position(context.position(), &unexplored)
            {
                self.walking_path =
                    PathFinder::new(self.map_reader.clone(), context.world_size().clone())
                        .compute_shortest_path(
                            context.position(),
                            &destination,
                            &context.player_details().orientation,
                        )
                        .to_path();

                if self.walking_path.is_empty() {
                    // no path to destination, remember it
                    self.unreachable_positions.push(destination);
                } else {
                    break self.try_to_walk(context);
                }
            } else {
                // nothing left unexplored
                break None;
            }
        }
    }

    fn try_to_walk(&mut self, context: &Context) -> Option<Action> {
        if self.walking_path.is_empty() {
            None
        } else {
            if let Some(next_position) = self.walking_path.pop() {
                let my_position = context.position();
                let my_orientation = context.player_details().orientation;
                if let Some(orientation) =
                    self.map_reader.find_alignment(my_position, &next_position)
                {
                    let action = self.map_reader.reorient(&my_orientation, &orientation);
                    match action {
                        Action::Move(direction) => self.try_to_step(
                            my_position,
                            &my_orientation,
                            &direction,
                            context.world_size(),
                        ),
                        Action::Rotate(_) => {
                            // putting back the next position, as we didn't walk to it
                            self.walking_path.push(next_position);
                            Some(action)
                        }
                        _ => None,
                    }
                } else {
                    panic!(
                        "Unable to compute alignment between neighbors {} and {}",
                        my_position, &next_position
                    );
                }
            } else {
                // no path to unexplored data
                None
            }
        }
    }

    fn try_to_step(
        &mut self,
        my_position: &Position,
        my_orientation: &Orientation,
        direction: &Direction,
        world_size: &WorldSize,
    ) -> Option<Action> {
        let step_orientation = if *direction == Direction::Backward {
            my_orientation.opposite()
        } else {
            *my_orientation
        };

        if let Some(next_pos) = my_position.follow(&step_orientation, world_size) {
            if self.map_reader.read_at(&next_pos) == MapCell::Terrain(Terrain::Field) {
                self.walking_direction = direction.clone();
                Some(Action::Move(direction.clone()))
            } else {
                None
            }
        } else {
            Some(Action::Rotate(Rotation::Clockwise))
        }
    }

    fn forget_everything(&mut self) -> Option<Action> {
        // forgetting everything causes a map rediscovery quest
        self.forward_scans_count = 0;
        self.full_scan_done = false;
        self.map_reader = Magellan::new();
        self.mono_scans.clear();
        self.ambush_turn = 0;
        self.scan_turn = 0;
        self.walking_path.clear();

        Some(Action::Scan(ScanType::Omni))
    }

    fn update_internal_state(&mut self, context: &Context) {
        if let Some(scan_result) = context.scanned_data() {
            if let Some(my_pos) =
                self.locate_myself_in_scanned_map(context.player_details(), scan_result)
            {
                let (start_x, start_y) = context.position().manhattan_distance(&my_pos);
                self.map_reader.update_map(scan_result, start_x, start_y);
            }

            // Once the map is (potentially) changed,
            // the path needs to be re-validated
            for p in self.walking_path.iter() {
                let cell = self.map_reader.read_at(p);

                match cell {
                    MapCell::Terrain(terrain) => match terrain {
                        Terrain::Field => {}
                        _ => {
                            // if path takes us over known obstacles, avoid them
                            self.walking_path.clear();
                            break;
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn get_backward_scans(&self, forward_scans: &Vec<Action>) -> Vec<Action> {
        self.get_360_scans()
            .into_iter()
            .filter(|s| !forward_scans.contains(s))
            .collect()
    }

    fn get_360_scans(&self) -> Vec<Action> {
        let mut result = Vec::new();

        result.push(Action::Scan(ScanType::Mono(Orientation::NorthWest)));
        result.push(Action::Scan(ScanType::Mono(Orientation::SouthWest)));
        result.push(Action::Scan(ScanType::Mono(Orientation::SouthEast)));
        result.push(Action::Scan(ScanType::Mono(Orientation::NorthEast)));

        result
    }

    fn store_previous_internal_state(&mut self, context: &Context) {
        self.previous_enemy_positions = self
            .map_reader
            .find_all_enemies(context.player_details().id, context.world_size());
    }

    fn locate_myself_in_scanned_map(
        &self,
        player_details: &Details,
        scan_result: &ScanResult,
    ) -> Option<Position> {
        for i in 0..SCANNING_DISTANCE {
            for j in 0..SCANNING_DISTANCE {
                if let MapCell::Player(details, _) = scan_result.data[i][j] {
                    if details == *player_details {
                        return Some(Position { x: j, y: i });
                    }
                }
            }
        }

        None
    }

    fn have_enemies_moved(&self, new_positions: &Vec<Position>) -> bool {
        let mut moved = false;

        for new_pos in new_positions {
            if !self.previous_enemy_positions.contains(new_pos) {
                moved = true;
                break;
            }
        }

        moved
    }

    #[allow(dead_code)]
    fn print(&mut self, context: &Context, highlights: Vec<Position>) {
        self.terminal.println("+++++++++++++++++++++++");

        for i in 0..context.world_size().y {
            let mut line = String::new();
            for j in 0..context.world_size().x {
                let pos = Position { x: j, y: i };
                let cell = self.map_reader.read_at(&pos);
                if pos == *context.position() {
                    if matches!(cell, MapCell::Terrain(_)) {
                        line = format!("{line}🟨");
                    } else {
                        line = format!("{line}{cell}");
                    }
                } else if highlights.contains(&pos) {
                    line = format!("{line}🔶");
                } else if self.walking_path.contains(&pos) {
                    line = format!("{line}⬜");
                } else {
                    line = format!("{line}{cell}");
                }
            }
            self.terminal.println(line);
        }
        self.terminal.println("-----------------------");
    }
}

impl Player for Aurelian {
    fn act(&mut self, context: Context) -> Action {
        self.update_internal_state(&context);
        // self.print(&context, Vec::new());

        let action = self.ai_logic(&context);
        self.store_previous_internal_state(&context);

        action
    }

    fn name(&self) -> String {
        "Kiimainen Apina".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
