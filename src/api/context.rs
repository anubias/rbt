use super::{
    action::Action, player::Details, position::Position, scan::ScanResult, world_size::WorldSize,
};

/// Represents the context that the game engine is sharing
/// with the player logic in every interaction.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    health: u8,
    max_turns: usize,
    previous_action: Action,
    player_details: Details,
    position: Position,
    scan: Option<ScanResult>,
    turn: usize,
    world_size: WorldSize,
}

#[allow(dead_code)]
impl Context {
    pub fn new(
        health: u8,
        max_turns: usize,
        previous_action: Action,
        player_details: Details,
        position: Position,
        scan: Option<ScanResult>,
        turn: usize,
        world_size: WorldSize,
    ) -> Self {
        Self {
            health,
            max_turns,
            player_details,
            position,
            previous_action,
            scan,
            turn,
            world_size,
        }
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

    pub fn scanned_data(&self) -> &Option<ScanResult> {
        &self.scan
    }

    pub fn max_turns(&self) -> usize {
        self.max_turns
    }

    pub fn turn(&self) -> usize {
        self.turn
    }

    pub fn world_size(&self) -> &WorldSize {
        &self.world_size
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if self.scan.is_some() {
            format!(
                "{{\n   player_details: {},\n   health: {},\n   previous_action: \"{}\",\n   position: {},\n   scanned_data: present\n}}",
                self.player_details, self.health, self.previous_action, self.position
            )
        } else {
            format!(
                "{{\n   player_details: {},\n   health: {},\n   previous_action: \"{}\",\n   position: {},\n   scanned_data: absent\n}}",
                self.player_details, self.health, self.previous_action, self.position,
            )
        };
        write!(f, "{text}")
    }
}
