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

        for i in 0..rounds {
            let game_id = i + 1;
            let players = self.get_players();
            let (quit, game_outcome) =
                self.run_single_game(game_id, players, self.world_size.clone());

            championship_outcome.add_game_result(game_outcome);
            println!("Game {game_id} finished");

            if quit {
                break;
            }
        }

        dbg!(championship_outcome);
    }
}

impl Championship {
    fn run_single_game(
        &self,
        game_id: u32,
        players: Vec<Box<dyn Player>>,
        world_size: WorldSize,
    ) -> (bool, GameOutcome) {
        let mut game = Game::new(world_size);
        game.spawn_players(players);

        game.start(game_id)
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
    fn test_only_one_player_is_ready() {
        let championship = Championship::new(League::Academy, WorldSize { x: 60, y: 45 });
        let players = championship.get_players();

        let ready: i32 = players
            .iter()
            .map(|player| if player.is_ready() { 1 } else { 0 })
            .collect::<Vec<i32>>()
            .iter()
            .sum();

        assert_eq!(1, ready);
    }

    #[test]
    fn test_foxy_player_is_ready() {
        let fox = TwentyCenturyFox::new();
        assert!(fox.is_ready())
    }
}
