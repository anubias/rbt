use crate::{api::{action::Action, aiming::Aiming, context::Context, orientation::Orientation, path_finder::PathFinder, position::Position, scan::ScanType}, WORLD_SIZE};
use super::shared::Data;

pub fn get_next_action(data: &mut Data, context: &Context) -> Action {
    data.map.actions_since_last_scan_northwest += 1;
    data.map.actions_since_last_scan_northeast += 1;
    data.map.actions_since_last_scan_southeast += 1;
    data.map.actions_since_last_scan_southwest += 1;
    
    if let Some(action) = danger_close(data, context) {
        return action;
    }
    else if let Some(action) = keep_scans_fresh(data, context) {
        return action;
    }
    else if let Some(action) = explore(data, context) {
        return action;
    }
    else if let Some(action) = move_towards_center(data, context) {
        return action;
    }
    else {
        return Action::Idle;
    }
}

fn danger_close(data: &mut Data, context: &Context) -> Option<Action> {
    let _ = context;
    let mut found_recent_track = false;
    for track in &data.map.tracks {
        if track.timestamp == context.turn() as i32 && context.position().could_hit_positionally(&Position { x: track.x, y: track.y }) {
            // Current track -> fire on it!
            return Some(Action::Fire(Aiming::Positional(Position {
                x: track.x,
                y: track.y,
            })));
        } else if track.timestamp > context.turn() as i32 - 2 && context.position().could_hit_positionally(&Position { x: track.x, y: track.y }) {
            // Recent track - check if it's still there to fire on..
            found_recent_track = true;
        }
    }

    if found_recent_track {
        return Some(Action::Scan(ScanType::Omni));
    }
    else {
        data.map.tracks.clear();
    }
    return None;
}

fn keep_scans_fresh(data: &mut Data, context: &Context) -> Option<Action> {
    let _ = context;
    if !data.map.initial_scan_done
    {
        return Some(Action::Scan(ScanType::Omni));
    }
    if data.map.actions_since_last_scan_northwest > 8 {
        return Some(Action::Scan(ScanType::Mono(Orientation::NorthWest)));
    }
    if data.map.actions_since_last_scan_northeast > 8 {
        return Some(Action::Scan(ScanType::Mono(Orientation::NorthEast)));
    }
    if data.map.actions_since_last_scan_southeast > 8 {
        return Some(Action::Scan(ScanType::Mono(Orientation::SouthEast)));
    }
    if data.map.actions_since_last_scan_southwest > 8 {
        return Some(Action::Scan(ScanType::Mono(Orientation::SouthWest)));
    }

    None
}

fn explore(data: &mut Data, context: &Context) -> Option<Action> {
    // TODO: This feels _incredibly_ inefficient...
    if let Some(any_unexplored_tile) = data.map.get_any_unexplored_tile() {
        return PathFinder::new(data.map.clone(), WORLD_SIZE).compute_shortest_path(context.position(), &any_unexplored_tile, &context.player_details().orientation).to_actions().pop();
    }
    None
}

fn move_towards_center(data: &mut Data, context: &Context) -> Option<Action> {
    // TODO: This feels _incredibly_ inefficient...
    // TODO: Also doesn't seem to actually go towards the center...
    return PathFinder::new(data.map.clone(), WORLD_SIZE).compute_shortest_path(context.position(), &Position { x: (context.world_size().x / 2), y: (context.world_size().x / 2) }, &context.player_details().orientation).to_actions().pop();
}
