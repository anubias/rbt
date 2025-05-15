pub(super) struct Evaluator;

use crate::api::position::Position;
use crate::api::position::SCANNING_DISTANCE;
use crate::api::world_size::WorldSize;

// Evaluator is used to give the AI hints about the current position
// The AI will always try to choose the move which has the bigger evaluation.
impl Evaluator {
    pub(super) fn evaluate_position(world_size: &WorldSize, target_position: &Position) -> i64 {
        const BIAS: i64 = 10;

        // furthest you can be from an edge
        let world_centre = (world_size.x / 2, world_size.y / 2);
        let furthest_dist = world_centre.0.max(world_centre.1);

        // Compute distances to the edges
        let left_dist = target_position.x;
        let right_dist = world_size.x - target_position.x;
        let top_dist = target_position.y;
        let bottom_dist = world_size.y - target_position.y;
        let dist_to_edge = left_dist.min(right_dist).min(top_dist).min(bottom_dist) as i64;

        let ideal_dist = (dist_to_edge - (SCANNING_DISTANCE as i64)).abs();

        // To prioritize edge closeness, we invert the score
        return (furthest_dist as i64 - ideal_dist) * BIAS;

        // Staying closer to the edge allows the AI to not check those directions after the initial check.
        // So be atleast a scanning distance away from the edge.
        // TODO: add a penalty being to close to the edge aswell.
    }
}
