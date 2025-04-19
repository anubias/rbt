use std::collections::HashMap;

use crate::api::{
    action::Action,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    position::Position,
    world_size::WorldSize,
};

pub struct NotComputed;
pub struct Computed;

/// MapReader is a trait that provides necessary map data for the PathFinder.
pub trait MapReader: Clone {
    /// Returns the MapCell found at `position`, as known by the player.
    fn read_at(&self, position: &Position) -> MapCell;
}

struct PathDetails {
    pub from: Position,
    pub to: Position,
    pub orientation: Orientation,
    pub path: Vec<Position>,
}

/// The `A*` pathfinding algorithm
///
/// Implemented by following the reference implementation at
/// https://en.wikipedia.org/wiki/A*_search_algorithm
///
/// The algorithm is designed to find the shortest path between two positions on the map.
///
/// It uses a heuristic function to estimate the cost of reaching a position from another
/// position. The heuristics are implemented by the players, and provided via the `Navigator` trait.
///
pub struct PathFinder<M, C> {
    map_reader: M,
    world_size: WorldSize,
    path_details: Option<PathDetails>,
    _computed: C,
}

impl<M: MapReader> PathFinder<M, NotComputed> {
    pub fn new(map_reader: M, world_size: WorldSize) -> Self {
        Self {
            map_reader,
            world_size,
            path_details: None,
            _computed: NotComputed,
        }
    }

    /// Computes the shortest path between two positions on the map.
    ///
    /// Parameters:
    /// - `from`: The starting position.
    /// - `to`: The target position.
    /// - `orientation`: The current orientation of the player.
    ///
    pub fn compute_shortest_path(
        &mut self,
        from: &Position,
        to: &Position,
        orientation: &Orientation,
    ) -> PathFinder<M, Computed> {
        let mut came_from = HashMap::new();
        let mut open_set = PriorityQueue::new();
        let mut f_score = HashMap::new();
        let mut g_score = HashMap::new();

        let start = from.clone();
        f_score.insert(start.clone(), self.distance(from, to));
        g_score.insert(start.clone(), 0);
        open_set.enqueue(start, &f_score);

        let route = loop {
            if open_set.is_empty() {
                // we didn't find a path
                break Vec::new();
            }

            if let Some(curr) = open_set.dequeue() {
                if curr == *to {
                    break self.build_reverse_path(came_from, curr);
                }

                for next in self.neighbors(&curr) {
                    let rotating_steps = if let Some(prev) = came_from.get(&curr) {
                        let prev_orientation =
                            Self::compute_alignment(prev, &curr).unwrap_or_default();
                        let next_orientation =
                            Self::compute_alignment(&curr, &next).unwrap_or_default();

                        Self::turning_steps(&prev_orientation, &next_orientation)
                    } else {
                        if let Some(new_orientation) = Self::compute_alignment(&curr, &next) {
                            Self::turning_steps(orientation, &new_orientation)
                        } else {
                            usize::MAX
                        }
                    };

                    let tentative_g_score =
                        Self::infinity_scores(&g_score, &curr) + rotating_steps + 1;

                    if tentative_g_score < Self::infinity_scores(&g_score, &next) {
                        came_from.insert(next.clone(), curr.clone());
                        g_score.insert(next.clone(), tentative_g_score);
                        f_score.insert(next.clone(), tentative_g_score + self.distance(&next, to));

                        open_set.enqueue(next, &f_score);
                    }
                }
            }
        };

        PathFinder {
            map_reader: self.map_reader.clone(),
            world_size: self.world_size.clone(),
            path_details: Some(PathDetails {
                orientation: orientation.clone(),
                from: from.clone(),
                to: to.clone(),
                path: route,
            }),
            _computed: Computed,
        }
    }
}

impl<M: MapReader> PathFinder<M, Computed> {
    /// Returns the path from `from` to `to` as a vector of positions.
    ///
    /// Note: The returned path is in 'reverse' order, meaning that the next position to follow
    /// is always at the end of the vector, which makes consumming the path easier (via the `pop()` method).

    pub fn to_path(&self) -> Vec<Position> {
        if let Some(ref pd) = self.path_details {
            pd.path.clone()
        } else {
            Vec::new()
        }
    }

    /// Creates a list of `Actions` to be taken in order to navigate the path.
    ///
    /// Note: The path is in reverse order, meaning that the next Action to perform
    /// is always at the end of the vector, which makes consuming the path easier (via the `pop()` method).
    pub fn to_actions(&mut self) -> Vec<Action> {
        if let Some(ref mut pd) = self.path_details {
            let mut result = Vec::new();
            let mut prev_pos = pd.from.clone();
            let mut prev_ori = pd.orientation.clone();

            while !pd.path.is_empty() {
                if let Some(next_pos) = pd.path.pop() {
                    if let Some(next_orientation) = Self::compute_alignment(&prev_pos, &next_pos) {
                        let (rotation, steps) = prev_ori.quick_turn(&next_orientation);

                        for _ in 0..steps {
                            result.push(Action::Rotate(rotation.clone()));
                        }
                        prev_ori = next_orientation.clone();
                    }

                    result.push(Action::Move(Direction::Forward));

                    prev_pos = next_pos;
                }
            }

            result.reverse();
            result
        } else {
            Vec::new()
        }
    }
}

impl<M: MapReader, C> PathFinder<M, C> {
    fn build_reverse_path(
        &self,
        came_from: HashMap<Position, Position>,
        mut current: Position,
    ) -> Vec<Position> {
        let mut result = Vec::new();

        result.push(current.clone());
        loop {
            if let Some(position) = came_from.get(&current) {
                current = position.clone();
                result.push(current.clone());
            } else {
                break;
            }
        }

        // removing the (unecessary) "starting" point from the reverse path
        result.pop();

        result
    }

    fn compute_alignment(from: &Position, to: &Position) -> Option<Orientation> {
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

    fn distance(&self, from: &Position, to: &Position) -> usize {
        from.pythagorean_distance(to) as usize
    }

    fn infinity_scores(scores: &HashMap<Position, usize>, position: &Position) -> usize {
        if let Some(s) = scores.get(position) {
            *s
        } else {
            usize::MAX
        }
    }

    fn neighbors(&self, position: &Position) -> Vec<Position> {
        let mut result = Vec::new();

        if let Some(north) = position.follow(&Orientation::North, &self.world_size) {
            if self.walkable(&north) {
                result.push(north);
            }
        }
        if let Some(north_east) = position.follow(&Orientation::NorthEast, &self.world_size) {
            if self.walkable(&north_east) {
                result.push(north_east);
            }
        }
        if let Some(east) = position.follow(&Orientation::East, &self.world_size) {
            if self.walkable(&east) {
                result.push(east);
            }
        }
        if let Some(south_east) = position.follow(&Orientation::SouthEast, &self.world_size) {
            if self.walkable(&south_east) {
                result.push(south_east);
            }
        }
        if let Some(south) = position.follow(&Orientation::South, &self.world_size) {
            if self.walkable(&south) {
                result.push(south);
            }
        }
        if let Some(south_west) = position.follow(&Orientation::SouthWest, &self.world_size) {
            if self.walkable(&south_west) {
                result.push(south_west);
            }
        }
        if let Some(west) = position.follow(&Orientation::West, &self.world_size) {
            if self.walkable(&west) {
                result.push(west);
            }
        }
        if let Some(north_west) = position.follow(&Orientation::NorthWest, &self.world_size) {
            if self.walkable(&north_west) {
                result.push(north_west);
            }
        }

        result
    }

    fn turning_steps(curr_orientation: &Orientation, next_orientation: &Orientation) -> usize {
        let (_, steps) = curr_orientation.quick_turn(&next_orientation);

        steps
    }

    fn walkable(&self, position: &Position) -> bool {
        let cell = self.map_reader.read_at(position);

        cell == MapCell::Unallocated || matches!(cell, MapCell::Terrain(Terrain::Field))
    }
}

/// A simple priority queue implementation
#[derive(Debug)]
struct PriorityQueue {
    queue: Vec<Position>,
}

impl PriorityQueue {
    fn new() -> Self {
        Self { queue: Vec::new() }
    }

    fn enqueue(&mut self, position: Position, f_scores: &HashMap<Position, usize>) {
        if self.queue.is_empty() {
            self.queue.push(position);
        } else {
            if !self.queue.contains(&position) {
                let index = self.queue.partition_point(|x| {
                    f_scores.contains_key(x)
                        && f_scores.contains_key(&position)
                        && f_scores.get(x).unwrap() > f_scores.get(&position).unwrap()
                });
                self.queue.insert(index, position);
            }
        }
    }

    fn dequeue(&mut self) -> Option<Position> {
        self.queue.pop()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
