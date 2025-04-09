use crate::{
    api::{player::Player, world_size::WorldSize},
    engine::{
        game::Game,
        outcome::{ChampionshipOutcome, GameOutcome},
    },
    players::{
        alvarez::Luis, armholt::Swede, arola::Arola, fox::TwentyCenturyFox, laurikainen::PlayerOne,
        moykkynen::Joonas, niemisto::Niemisto, pop::Aurelian, rahtu::Rahtu, rantala::PlayerTeemu,
        reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi, terava::PlAgiAntti,
    },
};

pub enum League {
    Academy,
    Open,
}

pub struct Championship {
    league: League,
    world_size: WorldSize,
}

impl Championship {
    pub fn new(league: League, world_size: WorldSize) -> Self {
        Championship { league, world_size }
    }

    pub fn run(&mut self, rounds: u32) {
        let mut championship_outcome = ChampionshipOutcome::new();
        let players = self.get_players();
        for (rank, player) in players.into_iter().enumerate() {
            championship_outcome.add_player(rank as u8 + 1, player.name());
        }

        for _ in 0..rounds {
            let players = self.get_players();
            let game_outcome = self.run_single_game(players, self.world_size.clone());

            championship_outcome.add_game_result(game_outcome);
        }
    }
}

impl Championship {
    fn run_single_game(&self, players: Vec<Box<dyn Player>>, world_size: WorldSize) -> GameOutcome {
        let mut game = Game::new(world_size);
        game.spawn_players(players);

        game.start()
    }

    fn get_players(&self) -> Vec<Box<dyn Player>> {
        let mut result: Vec<Box<dyn Player>> = vec![
            Box::new(Luis::new()),
            Box::new(Swede::new()),
            Box::new(Arola::new()),
            Box::new(PlayerOne::new()),
            Box::new(Joonas::new()),
            Box::new(Niemisto::new()),
            Box::new(Aurelian::new()),
            Box::new(Rahtu::new()),
            Box::new(PlayerTeemu::new()),
            Box::new(Samuli::new()),
            Box::new(Es::new()),
            Box::new(Siimesjarvi::new()),
            Box::new(PlAgiAntti::new()),
            Box::new(TwentyCenturyFox::new()),
        ];

        let mut open_players: Vec<Box<dyn Player>> = vec![Box::new(Aurelian::new())];

        match self.league {
            League::Academy => {}
            League::Open => result.append(&mut open_players),
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_player_is_ready() {
        let championship = Championship::new(League::Academy, WorldSize { x: 60, y: 45 });
        let players = championship.get_players();

        for player in players {
            assert!(!player.is_ready());
        }
    }
}
