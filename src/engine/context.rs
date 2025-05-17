use crate::{
    api::{
        action::Action, context::Context as ApiContext, map_cell::Terrain, player::Details,
        position::Position, rotation::Rotation, scan::ScanResult, world_size::WorldSize,
    },
    engine::game::DEAD_AVATAR,
};

/// Private consts

const DAMAGE_SINKING_INTO_LAKE: u16 = 100;
const DAMAGE_DIRECT_HIT: u16 = 75;
const DAMAGE_INDIRECT_HIT: u16 = 25;
const DAMAGE_COLLISION_WITH_PLAYER: u16 = 25;
const DAMAGE_COLLISION_WITH_FOREST: u16 = 10;

const SCORE_INDIRECT_HIT_BONUS: u16 = 1;
const SCORE_DIRECT_HIT_BONUS: u16 = 2;
const SCORE_KILLING_BONUS: u16 = 3;
const SCORE_SURVIVOR_BONUS: u16 = 5;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    value: u16,
}

impl Score {
    fn increment(&mut self, points: u16) {
        self.value += points;
    }
}

/// Represents the player context that the game engine is using for storing players state
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    cumulated_cpu_time: u128,
    health: u8,
    mobile: bool,
    previous_action: Action,
    player_details: Details,
    position: Position,
    scan: Option<ScanResult>,
    score: Score,
    turn: usize,
    world_size: WorldSize,
}

#[allow(dead_code)]
impl Context {
    pub fn new(player_details: Details, position: Position, world_size: WorldSize) -> Self {
        Self {
            health: 100,
            mobile: true,
            previous_action: Action::default(),
            player_details,
            position,
            scan: None,
            score: Score { value: 0 },
            cumulated_cpu_time: 0,
            turn: 0,
            world_size,
        }
    }

    pub fn damage_collision_forest(&mut self) {
        self.generic_damage(DAMAGE_COLLISION_WITH_FOREST);
    }

    pub fn damage_collision_player(&mut self, other: &mut Self) {
        self.generic_damage(DAMAGE_COLLISION_WITH_PLAYER);
        other.generic_damage(DAMAGE_COLLISION_WITH_PLAYER);
    }

    pub fn damage_direct_hit(&mut self, shooter_id: u8) -> u16 {
        self.hit_damage(shooter_id, DAMAGE_DIRECT_HIT, SCORE_DIRECT_HIT_BONUS)
    }

    pub fn damage_indirect_hit(&mut self, shooter_id: u8) -> u16 {
        self.hit_damage(shooter_id, DAMAGE_INDIRECT_HIT, SCORE_INDIRECT_HIT_BONUS)
    }

    pub fn reward_survivor(&mut self) {
        if self.health > 0 {
            self.score.increment(SCORE_SURVIVOR_BONUS);
        }
    }

    pub fn reward_hits(&mut self, amount: u16) {
        if self.health > 0 {
            self.score.increment(amount);
        }
    }

    pub fn health(&self) -> u8 {
        self.health
    }

    pub fn average_cpu_time_per_turn(&self) -> u128 {
        if self.turn > 0 {
            self.cumulated_cpu_time / self.turn as u128
        } else {
            0
        }
    }

    pub fn increase_cpu_time(&mut self, time: u128) {
        self.cumulated_cpu_time += time;
    }

    pub fn is_mobile(&self) -> bool {
        self.mobile
    }

    pub fn previous_action(&self) -> &Action {
        &self.previous_action
    }

    pub fn player_details(&self) -> &Details {
        &self.player_details
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn relocate(&mut self, new_position: &Position, walk_on: Terrain) {
        self.position = new_position.clone();

        match walk_on {
            Terrain::Lake => self.generic_damage(DAMAGE_SINKING_INTO_LAKE),
            Terrain::Swamp => self.mobile = false,
            _ => {}
        }
    }

    pub fn rotate(&mut self, rotation: &Rotation) {
        self.player_details.orientation = match rotation {
            Rotation::Clockwise => self.player_details.orientation.rotated_clockwise(),
            Rotation::CounterClockwise => {
                self.player_details.orientation.rotated_counter_clockwise()
            }
        }
    }

    pub fn scanned_data(&self) -> &Option<ScanResult> {
        &self.scan
    }

    pub fn score(&self) -> u16 {
        self.score.value
    }

    pub fn set_previous_action(&mut self, action: Action) {
        self.previous_action = action
    }

    pub fn set_scanned_data(&mut self, scan: Option<ScanResult>) {
        self.scan = scan;
    }

    pub fn set_turn(&mut self, turn: usize) {
        self.turn = turn;
    }

    pub fn turn(&self) -> usize {
        self.turn
    }

    pub fn world_size(&self) -> &WorldSize {
        &self.world_size
    }
}

impl Context {
    fn generic_damage(&mut self, amount: u16) {
        self.health = self.health.saturating_sub(amount as u8);
        if self.health == 0 {
            self.player_details.avatar = DEAD_AVATAR;
            self.player_details.alive = false;
        }
    }

    fn hit_damage(&mut self, shooter_id: u8, damage_amount: u16, reward_amount: u16) -> u16 {
        let mut reward = 0;

        if self.health > 0 {
            self.generic_damage(damage_amount);

            if self.player_details.id != shooter_id {
                reward += reward_amount;

                if self.health == 0 {
                    reward += SCORE_KILLING_BONUS;
                }
            }
        }

        reward
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if self.scan.is_some() {
            format!(
                "{{\n   player_details: {},\n   health: {},\n   mobile: {},\n   previous_action: \"{}\",\n   position: {},\n   scanned_data: present\n}}",
                self.player_details, self.health, self.mobile, self.previous_action, self.position
            )
        } else {
            format!(
                "{{\n   player_details: {},\n   health: {},\n   mobile: {},\n   previous_action: \"{}\",\n   position: {},\n   scanned_data: absent\n}}",
                self.player_details, self.health, self.mobile, self.previous_action, self.position,
            )
        };
        write!(f, "{text}")
    }
}

impl Into<ApiContext> for Context {
    fn into(self) -> ApiContext {
        ApiContext::new(
            self.health,
            self.previous_action.clone(),
            self.player_details.clone(),
            self.position.clone(),
            self.scan.clone(),
            self.turn,
            self.world_size.clone(),
        )
    }
}
