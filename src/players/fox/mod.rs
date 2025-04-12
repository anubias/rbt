use rand::{rngs::ThreadRng, thread_rng, Rng};

use crate::{
    api::{
        action::Action,
        aiming::Aiming,
        context::Context,
        direction::Direction,
        map_cell::{MapCell, Terrain},
        orientation::Orientation,
        player::Player,
        position::{Position, SCANNING_DISTANCE},
        scan::{ScanResult, ScanType},
    },
    terminal::Terminal,
};

const MAX_STEPS: u8 = 3;

pub struct TwentyCenturyFox {
    move_action: Action,
    scan_data: ScanResult,
    scan_pos: Position,
    steps: u8,
    target_orientation: Orientation,
    terminal: Terminal,
}

impl TwentyCenturyFox {
    pub fn new() -> Self {
        Self {
            move_action: Action::default(),
            scan_data: ScanResult::default(),
            scan_pos: Position { x: 0, y: 0 },
            steps: 0,
            target_orientation: Orientation::default(),
            terminal: Terminal::new(),
        }
    }
}

impl TwentyCenturyFox {
    fn random_orientation(&mut self, rng: &mut ThreadRng) -> Orientation {
        Orientation::from(rng.gen_range(0..Orientation::get_cardinal_direction_count()))
    }

    fn next_state(&mut self, context: Context, rng: &mut ThreadRng) -> Action {
        match context.previous_action() {
            Action::Idle => Action::Scan(ScanType::Omni),
            Action::Scan(_) => {
                self.scan_data = context.scanned_data().clone().unwrap_or_default();
                self.scan_pos = context.position().clone();
                self.move_or_rotate(&context, rng);
                self.move_action.clone()
            }
            Action::Fire(_) => self.move_action.clone(),
            Action::Move(_) => {
                self.move_or_rotate(&context, rng);
                self.next_step(context)
            }
            Action::Rotate(_) => {
                if self.target_orientation == context.player_details().orientation {
                    self.move_action = Action::Move(Direction::Forward);
                }
                self.next_step(context)
            }
        }
    }

    fn move_or_rotate(&mut self, context: &Context, rng: &mut ThreadRng) {
        let (mut valid_pos, mut walkable_pos) = self.check_next_pos(context);

        let mut x = 0;
        while !(valid_pos && walkable_pos) {
            // select random orientation
            self.target_orientation = self.random_orientation(rng);

            // check if valid and walkable
            (valid_pos, walkable_pos) = self.check_next_pos(context);

            if x >= 1000 {
                break;
            }
            x += 1;
        }

        if self.target_orientation == context.player_details().orientation {
            self.move_action = Action::Move(Direction::Forward);
        } else {
            let (rot, _) = context
                .player_details()
                .orientation
                .quick_turn(&self.target_orientation);

            self.move_action = Action::Rotate(rot);
        }
    }

    fn check_next_pos(&mut self, context: &Context) -> (bool, bool) {
        let mut valid_pos = false;
        let mut walkable_pos = false;

        if let Some(next_pos) = context
            .position()
            .follow(&self.target_orientation, context.world_size())
        {
            valid_pos = true;
            walkable_pos = self.can_walk(&next_pos)
        }

        (valid_pos, walkable_pos)
    }

    fn next_step(&mut self, context: Context) -> Action {
        if self.steps >= MAX_STEPS {
            self.steps = 0;
            Action::Scan(ScanType::Omni)
        } else {
            self.steps += 1;
            Action::Fire(Aiming::Cardinal(context.player_details().orientation))
        }
    }

    fn can_walk(&self, next_pos: &Position) -> bool {
        let scan_pos = self.translate_to_scan_position(next_pos);

        match self.scan_data.data[scan_pos.y][scan_pos.x] {
            MapCell::Terrain(Terrain::Field) | MapCell::Player(_, _) => true,
            _ => false,
        }
    }

    fn translate_to_scan_position(&self, new_position: &Position) -> Position {
        let (new_x, new_y) = (new_position.x, new_position.y);

        Position {
            x: SCANNING_DISTANCE / 2 + new_x - self.scan_pos.x,
            y: SCANNING_DISTANCE / 2 + new_y - self.scan_pos.y,
        }
    }
}

impl Player for TwentyCenturyFox {
    fn act(&mut self, context: Context) -> Action {
        // self.terminal.println(&self.scan_data);

        let mut rng: ThreadRng = thread_rng();
        self.next_state(context, &mut rng)
    }

    fn name(&self) -> String {
        "Twenty Century Fox".to_string()
    }
}
