use super::types::*;

use std::collections::{HashMap, VecDeque};
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct TankInfo {
    pub id: PId,
    pub alive: bool,
    pub known_positions: VecDeque<(Position, Distance, Turn)>,
    pub orientation: Orientation,
    pub prediction: Option<Position>,
}

impl TankInfo {
    // Copy from Anubias ;)
    pub fn find_alignment(&self, from: &Position, to: &Position) -> Option<Orientation> {
        let (dx, dy) = from.manhattan_distance(to);

        match (dx, dy) {
            (0, ..0) => Some(Orientation::South),
            (0, 0..) => Some(Orientation::North),
            (..0, 0) => Some(Orientation::East),
            (0.., 0) => Some(Orientation::West),
            _ => {
                if dx.abs() == dy.abs() {
                    if dx < 0 && dy < 0 {
                        Some(Orientation::SouthEast)
                    } else if dx < 0 && dy > 0 {
                        Some(Orientation::NorthEast)
                    } else if dx > 0 && dy < 0 {
                        Some(Orientation::SouthWest)
                    } else {
                        Some(Orientation::NorthWest)
                    }
                } else {
                    None
                }
            }
        }
    }
}

pub struct TanksTracker {
    tanks: HashMap<PId, TankInfo>,
    own_id: Option<PId>,
}

impl Default for TanksTracker {
    fn default() -> Self {
        Self {
            tanks: HashMap::new(),
            own_id: None,
        }
    }
}

impl Index<PId> for TanksTracker {
    type Output = TankInfo;

    fn index(&self, id: PId) -> &Self::Output {
        self.tanks.get(&id).expect("No enemy with that ID")
    }
}

impl IndexMut<PId> for TanksTracker {
    fn index_mut(&mut self, id: PId) -> &mut Self::Output {
        self.tanks.get_mut(&id).expect("No enemy with that ID")
    }
}

impl TanksTracker {
    pub fn new(own_id: PId) -> Self {
        Self {
            tanks: HashMap::new(),
            own_id: Some(own_id),
        }
    }

    pub fn update_own_id(&mut self, position: &Position, turn: Turn) {
        if let Some(own_id) = self.own_id {
            if let Some(tank) = self.tanks.get_mut(&own_id) {
                if tank.known_positions.is_empty() ||
                   (tank.known_positions.back().map(|&(_, _, t)| t == turn - 1).unwrap_or(false)) {
                    tank.known_positions.push_back((position.clone(), 0.0, turn));
                }
            }
        }
    }

    pub fn track(&mut self, details: &Details, pos: Position, turn: Turn, ref_point: &Position) {
        let distance: Distance = ref_point.pythagorean_distance(&pos);

        // This is a new tank
        if !self.tanks.contains_key(&details.id) {
            let enemy = TankInfo {
                id: details.id,
                known_positions: VecDeque::from(vec![(pos, distance, turn)]),
                orientation: details.orientation.clone(),
                prediction: None,
                alive: details.alive,
            };

            self.tanks.insert(details.id, enemy);
            return;
        }

        let prediction = self.predict_next_position(&details.id, &turn);

        if let Some(tank) = self.tanks.get_mut(&details.id) {
            tank.alive = details.alive;

            tank.known_positions.push_back((pos, distance, turn));
            if tank.known_positions.len() > 10 {
                tank.known_positions.pop_front();
            }
            tank.orientation = details.orientation.clone();
            tank.prediction = prediction
        }
    }

    pub fn enemies_found(&self, turn: Turn) -> Vec<TankInfo> {
        self.tanks
            .iter()
            .filter_map(|(_, enemy)| {
                if enemy.id != self.own_id.unwrap() && enemy.alive {
                    if let Some(&(_, _, last_turn)) = enemy.known_positions.back() {
                        if last_turn == turn {
                            return Some(enemy.clone());
                        }
                    }
                }
                None
            })
        .collect()
    }
}

impl TanksTracker {
    fn predict_next_position(&self, enemy_id: &PId, turn: &Turn) -> Option<Position> {
        if let Some(enemy) = self.tanks.get(enemy_id) {
            if let Some(seen) = self.last_consecutive_tracks(enemy_id, turn) {
                if seen.len() >= 2 {
                    let latest_pos = &seen[0].0;
                    let previous_pos = &seen[1].0;
                    let dx = latest_pos.x - previous_pos.x;
                    let dy = latest_pos.y - previous_pos.y;

                    return Some(Position {
                        x: latest_pos.x + dx,
                        y: latest_pos.y + dy,
                    });
                }
            }

            if let Some(&(ref pos, _, _)) = enemy.known_positions.front() {
                // If we only have one position, use the orientation
                match enemy.orientation {
                    Orientation::North => {
                        return Some(Position {
                            x: pos.x,
                            y: pos.y - 1,
                        })
                    }
                    Orientation::South => {
                        return Some(Position {
                            x: pos.x,
                            y: pos.y + 1,
                        })
                    }
                    Orientation::East => {
                        return Some(Position {
                            x: pos.x + 1,
                            y: pos.y,
                        })
                    }
                    Orientation::West => {
                        return Some(Position {
                            x: pos.x - 1,
                            y: pos.y,
                        })
                    }
                    Orientation::NorthEast => {
                        return Some(Position {
                            x: pos.x + 1,
                            y: pos.y - 1,
                        })
                    }
                    Orientation::SouthEast => {
                        return Some(Position {
                            x: pos.x + 1,
                            y: pos.y + 1,
                        })
                    }
                    Orientation::SouthWest => {
                        return Some(Position {
                            x: pos.x - 1,
                            y: pos.y + 1,
                        })
                    }
                    Orientation::NorthWest => {
                        return Some(Position {
                            x: pos.x - 1,
                            y: pos.y - 1,
                        })
                    }
                }
            }
        }
        None
    }

    fn last_consecutive_tracks(
        &self,
        enemy_id: &PId,
        from_turn: &Turn,
    ) -> Option<Vec<(Position, Turn)>> {
        self.tanks.get(enemy_id).map(|enemy| {
            if let Some(start_index) = enemy
                .known_positions
                .iter()
                .position(|(_, _, turn)| *turn == *from_turn)
            {
                let mut result = Vec::new();
                let mut expected_turn = *from_turn;
                let mut current_index = start_index;

                loop {
                    let (pos, _, turn) = &enemy.known_positions[current_index];

                    if *turn != expected_turn {
                        // Caring only about consecutive turns
                        break;
                    }

                    result.push((pos.clone(), *turn));
                    expected_turn -= 1;

                    if current_index == 0 {
                        break;
                    }

                    current_index -= 1;
                }
                result
            } else {
                Vec::new()
            }
        })
    }
}
