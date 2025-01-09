mod arola;
mod pop;
mod rahtu;
mod reponen;
mod salonen;
mod siimesjarvi;

mod utils;

use arola::Arola;
use pop::Aurelian;
use rahtu::Rahtu;
use reponen::Samuli;
use salonen::Es;
use siimesjarvi::Siimesjarvi;

use utils::{Player, Position};

fn main() {
    let arola = Arola::new();
    let pop = Aurelian::new(Position { x: 0, y: 0 });
    let rahtu = Rahtu::new();
    let reponen = Samuli::new();
    let salonen = Es::new();
    let siimesjarvi = Siimesjarvi::new();

    println!(
        "player: {}: health: {}, score:{}",
        arola.get_name(),
        arola.get_health(),
        arola.get_score()
    );
    println!(
        "player: {}: health: {}, score:{}",
        pop.get_name(),
        pop.get_health(),
        pop.get_score()
    );
    println!(
        "player: {}: health: {}, score:{}",
        rahtu.get_name(),
        rahtu.get_health(),
        rahtu.get_score()
    );
    println!(
        "player: {}: health: {}, score:{}",
        reponen.get_name(),
        reponen.get_health(),
        reponen.get_score()
    );
    println!(
        "player: {}: health: {}, score:{}",
        salonen.get_name(),
        salonen.get_health(),
        salonen.get_score()
    );
    println!(
        "player: {}: health: {}, score:{}",
        siimesjarvi.get_name(),
        siimesjarvi.get_health(),
        siimesjarvi.get_score()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health() {
        let arola = Arola::new();
        let pop = Aurelian::new();
        let rahtu = Rahtu::new();
        let reponen = Samuli::new();
        let salonen = Es::new();
        let siimesjarvi = Siimesjarvi::new();

        assert_eq!(100, arola.get_health());
        assert_eq!(100, pop.get_health());
        assert_eq!(100, rahtu.get_health());
        assert_eq!(100, reponen.get_health());
        assert_eq!(100, salonen.get_health());
        assert_eq!(100, siimesjarvi.get_health());
    }
}
