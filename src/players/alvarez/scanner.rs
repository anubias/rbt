use super::model::*;
use super::types::*;

#[derive(Clone, Debug)]
pub struct PositionDelta {
    x: isize,
    y: isize,
}

pub trait PositionResolver {
    fn position_delta(&self, scan: &ScanResult) -> PositionDelta;
    fn position_absolute(
        &self,
        delta: &PositionDelta,
        rel_pos: Position,
        world_size: &WorldSize,
    ) -> Option<Position>;
    fn player_rel_pos(&self, scan: &ScanResult) -> Position;
}

pub trait Scanner: PositionResolver {
    fn process(
        &mut self,
        world: &mut WorldModel,
        scan: &ScanResult,
        turn: usize,
        reference_pos: &Position,
    );
}

pub struct ScanProces {
    id: PId,           // Own Player ID
    abs_pos: Position, // Own absolute position
}

impl<'a> ScanProces {
    pub fn new(id: PId, abs_pos: Position) -> Self {
        Self { id, abs_pos }
    }
}

// A wrapper for private methods
impl PositionResolver for ScanProces {
    fn position_delta(&self, scan: &ScanResult) -> PositionDelta {
        let rel_pos = self.player_rel_pos(scan);
        PositionDelta {
            x: self.abs_pos.x as isize - rel_pos.x as isize,
            y: self.abs_pos.y as isize - rel_pos.y as isize,
        }
    }

    fn position_absolute(
        &self,
        delta: &PositionDelta,
        rel_pos: Position,
        world_size: &WorldSize,
    ) -> Option<Position> {
        let abs_x = (delta.x + rel_pos.x as isize).max(0) as usize;
        let abs_y = (delta.y + rel_pos.y as isize).max(0) as usize;
        if abs_y == 0 || abs_x == 0 || abs_y >= world_size.y - 1 || abs_x >= world_size.x - 1 {
            return None;
        }
        let abs_pos = Position { x: abs_x, y: abs_y };
        Some(abs_pos)
    }

    fn player_rel_pos(&self, scan: &ScanResult) -> Position {
        let pos: Position = match scan.scan_type {
            ScanType::Omni => {
                let center = scan.data.len() / 2;
                Position {
                    x: center,
                    y: center,
                }
            }
            ScanType::Mono(orientation) => {
                let center = scan.data.len() / 2;
                let max = scan.data.len() - 1;
                let (x, y) = match orientation {
                    Orientation::North => (center, max),
                    Orientation::East => (0, center),
                    Orientation::South => (center, 0),
                    Orientation::West => (max, center),
                    Orientation::NorthEast => (0, max),
                    Orientation::SouthEast => (0, 0),
                    Orientation::SouthWest => (max, 0),
                    Orientation::NorthWest => (max, max),
                };
                Position { x: x, y: y }
            }
        };

        if let MapCell::Player(Details { id, .. }, _) = scan.data[pos.y][pos.x] {
            if id == self.id {
                return pos;
            }
        }

        println!("Guess failed {pos}, searching for player");
        for (y, row) in scan.data.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let MapCell::Player(player, _) = cell {
                    if player.id == self.id {
                        return Position { x, y };
                    }
                }
            }
        }
        assert!(false, "Player not found! This is a bug.");
        Position { x: 0, y: 0 }
    }
}

impl Scanner for ScanProces {
    fn process(
        &mut self,
        world: &mut WorldModel,
        scan: &ScanResult,
        turn: usize,
        ref_pos: &Position,
    ) {
        let delta = self.position_delta(scan);

        for (y, row) in scan.data.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let abs_pos = match self.position_absolute(
                    &delta,
                    Position { x, y },
                    &world.map.world_size(),
                ) {
                    Some(value) => value,
                    None => continue,
                };

                match cell {
                    MapCell::Player(d, t) => {
                        if d.alive {
                            world.tanks.track(d, abs_pos.clone(), turn, &ref_pos);
                            world.map.update_tile(
                                &abs_pos,
                                &MapCell::Player(d.clone(), t.clone()),
                                turn,
                            );
                        } else {
                            world.map.update_tile(
                                &abs_pos,
                                &MapCell::Terrain(Terrain::Forest(TreeType::default())),
                                turn,
                            );
                        }
                    }
                    MapCell::Explosion(_, t) | MapCell::Shell(_, t) => {
                        world
                            .map
                            .update_tile(&abs_pos, &MapCell::Terrain(t.clone()), turn);
                    }
                    _ => {
                        world.map.update_tile(&abs_pos, &cell.clone(), turn);
                    }
                }
            }
        }
    }
}
