use super::player::*;

#[derive(Default)]
pub struct Aurelian {
    iteration: usize,
    world_map: Vec<Vec<MapCell>>,
}

impl Aurelian {
    fn update_world_map(&mut self, context: &Context) {
        if let Some(scan_result) = context.scanned_data() {
            if let Some((x, y)) =
                self.locate_myself_in_scanned_map(context.player_id(), scan_result)
            {
                let (start_x, start_y) = (
                    context.position().y as isize - y,
                    context.position().x as isize - x,
                );
            }
        }
    }

    fn locate_myself_in_scanned_map(
        &self,
        player_id: u8,
        scan_result: &ScanResult,
    ) -> Option<(isize, isize)> {
        for x in 0..SCANNING_DISTANCE {
            for y in 0..SCANNING_DISTANCE {
                if scan_result.data[x][y] == MapCell::Player(player_id) {
                    return Some((x as isize, y as isize));
                }
            }
        }

        None
    }

    fn copy_scanned_data(&mut self, scan_result: &ScanResult, start_x: usize, start_y: usize) {
        // for i in 0..SCANNING_DISTANCE {
        //     for j in 0..SCANNING_DISTANCE
        // }
    }
}

impl Player for Aurelian {
    fn act(&mut self, context: &Context) -> Action {
        println!("{}: context:{}", self.name(), context);

        self.update_world_map(context);
        // if let Some(data) = context.scanned_data() {
        //     println!("{data}");
        // }

        // if context.is_mobile() {
        //     Action::Move(Direction::Forward)
        // } else {
        //     Action::Move(Direction::Backward)
        // }
        // Action::Rotate(Rotation::Clockwise)

        self.iteration += 1;

        match self.iteration % 2 {
            1 => Action::Scan(ScanType::Omni),
            _ => Action::Move(Direction::Forward),
        }
    }

    fn name(&self) -> String {
        "Aurelian".to_string()
    }

    fn is_ready(&self) -> bool {
        true
    }
}
