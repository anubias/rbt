use super::super::player::*;
use super::Arola;
use super::PlayerState;

impl Arola {
    pub(super) fn attack(&mut self, context: &Context) -> (PlayerState, Action) {
        if let Some(scan_result) = context.scanned_data() {
            let other_players =
                scan_result.find_other_players(context.player_details(), context.position());
            if !other_players.is_empty() {
                return (
                    PlayerState::Attack,
                    Action::Fire(Aiming::Positional(other_players[0].1.clone())),
                );
            }
        }

        return (
            PlayerState::Explore,
            Action::Scan(ScanType::Mono(context.orientation().clone())),
        );
    }
}
