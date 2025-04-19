use std::{collections::HashMap, usize};

use crate::api::{
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    position::Position,
    world_size::WorldSize,
};

/// Navigator is a trait that provides the necessary information for the pathfinding algorithm.
/// It is used to determine the cost of moving from one position to another, as well as the
/// cost of turning the tank to a desired orientation.
///
/// It provides default implementations for some methods, but players may override them. The only
/// method for which default implementation cannot be provded is `cell_at`, as fetching the discovered
/// map information is specific to each player.
pub trait Navigator {
    /// Returns the map cell at `position`, as the player knows it
    fn cell_at(&self, position: &Position) -> MapCell;

    /// Returns the cost of rotating the tank to the desired orientation
    /// The cost is the number of steps needed to turn the tank
    /// to the desired orientation.
    ///
    /// Used as a cost heuristic by the pathfinding algorithm.
    fn turning_steps(
        &self,
        curr_orientation: &Orientation,
        next_orientation: &Orientation,
    ) -> usize {
        let (_, steps) = curr_orientation.quick_turn(&next_orientation);

        steps
    }

    /// Computes the distance between two positions, used as a cost heuristic by the pathfinding algorithm.
    fn distance(&self, from: &Position, to: &Position) -> usize {
        from.pythagorean_distance(to) as usize
    }
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
pub struct PathFinder {}

impl PathFinder {
    /// Finds the shortest path between two positions on the map.
    pub fn find_path(
        navigator: &impl Navigator,
        from: &Position,
        to: &Position,
        curr_orientation: &Orientation,
        world_size: &WorldSize,
    ) -> Vec<Position> {
        let mut came_from = HashMap::new();
        let mut open_set = PriorityQueue::new();
        let mut f_score = HashMap::new();
        let mut g_score = HashMap::new();

        let start = from.clone();
        f_score.insert(start.clone(), navigator.distance(from, to));
        g_score.insert(start.clone(), 0);
        open_set.enqueue(start, &f_score);

        loop {
            if open_set.is_empty() {
                // we didn't find a path
                break Vec::new();
            }

            if let Some(curr) = open_set.dequeue() {
                if curr == *to {
                    return Self::build_reverse_path(came_from, curr);
                }

                for next in Self::neighbors(&curr, navigator, world_size) {
                    let rotating_steps = if let Some(prev) = came_from.get(&curr) {
                        let prev_orientation =
                            Self::find_alignment(prev, &curr).unwrap_or_default();
                        let next_orientation =
                            Self::find_alignment(&curr, &next).unwrap_or_default();

                        navigator.turning_steps(&prev_orientation, &next_orientation)
                    } else {
                        if let Some(new_orientation) = Self::find_alignment(&curr, &next) {
                            navigator.turning_steps(curr_orientation, &new_orientation)
                        } else {
                            usize::MAX
                        }
                    };

                    let tentative_g_score =
                        Self::infinity_scores(&g_score, &curr) + rotating_steps + 1;

                    if tentative_g_score < Self::infinity_scores(&g_score, &next) {
                        came_from.insert(next.clone(), curr.clone());
                        g_score.insert(next.clone(), tentative_g_score);
                        f_score.insert(
                            next.clone(),
                            tentative_g_score + navigator.distance(&next, to),
                        );

                        open_set.enqueue(next, &f_score);
                    }
                }
            }
        }
    }
}

impl PathFinder {
    fn build_reverse_path(
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

    fn find_alignment(from: &Position, to: &Position) -> Option<Orientation> {
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

    fn infinity_scores(scores: &HashMap<Position, usize>, position: &Position) -> usize {
        if let Some(s) = scores.get(position) {
            *s
        } else {
            usize::MAX
        }
    }

    fn neighbors(
        position: &Position,
        navigator: &impl Navigator,
        size: &WorldSize,
    ) -> Vec<Position> {
        let mut result = Vec::new();

        if let Some(north) = position.follow(&Orientation::North, size) {
            if Self::walkable(&north, navigator) {
                result.push(north);
            }
        }
        if let Some(north_east) = position.follow(&Orientation::NorthEast, size) {
            if Self::walkable(&north_east, navigator) {
                result.push(north_east);
            }
        }
        if let Some(east) = position.follow(&Orientation::East, size) {
            if Self::walkable(&east, navigator) {
                result.push(east);
            }
        }
        if let Some(south_east) = position.follow(&Orientation::SouthEast, size) {
            if Self::walkable(&south_east, navigator) {
                result.push(south_east);
            }
        }
        if let Some(south) = position.follow(&Orientation::South, size) {
            if Self::walkable(&south, navigator) {
                result.push(south);
            }
        }
        if let Some(south_west) = position.follow(&Orientation::SouthWest, size) {
            if Self::walkable(&south_west, navigator) {
                result.push(south_west);
            }
        }
        if let Some(west) = position.follow(&Orientation::West, size) {
            if Self::walkable(&west, navigator) {
                result.push(west);
            }
        }
        if let Some(north_west) = position.follow(&Orientation::NorthWest, size) {
            if Self::walkable(&north_west, navigator) {
                result.push(north_west);
            }
        }

        result
    }

    fn walkable(position: &Position, navigator: &impl Navigator) -> bool {
        let cell = navigator.cell_at(position);

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
