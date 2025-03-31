use crate::{
    api::player::Player,
    engine::game::Game,
    players::{
        alvarez::Luis, armholt::Swede, arola::Arola, fox::TwentyCenturyFox, laurikainen::PlayerOne,
        moykkynen::Joonas, niemisto::Niemisto, pop::Aurelian, rahtu::Rahtu, rantala::PlayerTeemu,
        reponen::Samuli, salonen::Es, siimesjarvi::Siimesjarvi, terava::PlAgiAntti,
    },
};

pub enum League {
    Academy,
    // Open,
}

pub struct Championship {
    //results: Vec
    league: League,
}

impl Championship {
    pub fn new(league: League) -> Self {
        Championship { league }
    }

    pub fn build_academy_championship() -> Vec<Box<dyn Player>> {
        vec![
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
        ]
    }

    pub fn run(&mut self, rounds: u32) {
        for _ in 0..rounds {
            let players = match self.league {
                League::Academy => Championship::build_academy_championship(),
            };

            run_game(players);
        }
    }
}

fn run_game(players: Vec<Box<dyn Player>>) {
    let mut game = Game::new();
    for player in players {
        game.spawn_player(player);
    }

    game.start();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_player_is_ready() {
        let players = Championship::build_academy_championship();

        for player in players {
            assert!(!player.is_ready());
        }
    }
}
