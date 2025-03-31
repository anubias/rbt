use crate::{
    api::{player::Player, world_size::WorldSize},
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
    world_size: WorldSize,
}

impl Championship {
    pub fn new(league: League, world_size: WorldSize) -> Self {
        Championship { league, world_size }
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

            run_game(players, self.world_size.clone());
        }
    }
}

fn run_game(players: Vec<Box<dyn Player>>, world_size: WorldSize) {
    let mut game = Game::new(world_size);
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
