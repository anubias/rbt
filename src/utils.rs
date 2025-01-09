pub trait Player {
    /// This is the player's own decision computation and desired action
    fn act() {}

    /// Returns the health of the player, valid values are 0..=100
    fn get_health(&self) -> u8 {
        0
    }

    /// Returns the player's name
    fn get_name(&self) -> String {
        "Unnamed".to_string()
    }

    /// This indicates whether the player is ready to battle
    fn is_ready(&self) -> bool {
        false
    }

    /// Returns the accumulated battle score of the player
    fn get_score(&self) -> u32 {
        0
    }
}

pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub enum Action {
    Fire,
    Move(Direction),
    Rotate(Rotation),
    Scan(ScanType),
}

pub enum Direction {
    Forward,
    Backward,
}

pub enum Rotation {
    Clockwise,
    CounterClockwise,
}

pub enum ScanType {
    Omni,
    Directional,
}
