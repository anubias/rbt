use crate::api::{
    action::Action,
    aiming::Aiming,
    context::Context,
    direction::Direction,
    map_cell::{MapCell, Terrain},
    orientation::Orientation,
    player::Player,
    rotation::Rotation,
    scan::{ScanResult, ScanType},
};

use rand::random;

pub struct Swede {
    last_rotation: Rotation,
}

impl Swede {
    pub fn new() -> Self {
        Self {
            last_rotation: Rotation::Clockwise,
        }
    }
}

impl Swede {
    fn handle_omni_scan(&mut self, _scan_result: &ScanResult, context: &Context) -> Action {
        //this shouldnt happen as the the logic is not using omni scan
        //shoot before asking any questions ;)
        Action::Fire(Aiming::Cardinal(context.player_details().orientation))
    }

    fn handle_mono_scan(&mut self, scan_result: &ScanResult, context: &Context) -> Action {
        //check what is ahead, shoot if someone is ahead and alive
        let mut looking_north_south = false;
        let mut looking_east_west = false;
        match context.player_details().orientation {
            Orientation::North | Orientation::South => looking_north_south = true,
            Orientation::East | Orientation::West => looking_east_west = true,
            _ => {}
        }

        //println!("{scan_result}");

        if looking_north_south {
            //assumption is that tank is at bottom/top center of the scan box
            //straight ahead has same x coordinate
            let row_width = scan_result.data[0].len();

            for scan_y in 0..scan_result.data.len() {
                let cell = scan_result.data[scan_y][row_width / 2];
                if (scan_y == 1 && context.player_details().orientation == Orientation::South)
                    || (scan_y == scan_result.data.len() - 2
                        && context.player_details().orientation == Orientation::North)
                {
                    let tank_row = if context.player_details().orientation == Orientation::North {
                        scan_y + 1
                    } else {
                        scan_y - 1
                    };
                    let left_cell = scan_result.data[tank_row][row_width / 2 - 1];
                    let right_cell = scan_result.data[tank_row][row_width / 2 + 1];
                    match cell {
                        MapCell::Terrain(t) => {
                            match t {
                                Terrain::Field => {}
                                _ => {
                                    if context.player_details().orientation == Orientation::North {
                                        if left_cell == MapCell::Terrain(Terrain::Field) {
                                            self.last_rotation = Rotation::CounterClockwise;
                                        } else {
                                            self.last_rotation = Rotation::Clockwise;
                                        }
                                    }
                                    if context.player_details().orientation == Orientation::South {
                                        if left_cell == MapCell::Terrain(Terrain::Field) {
                                            self.last_rotation = Rotation::Clockwise;
                                        } else {
                                            self.last_rotation = Rotation::CounterClockwise;
                                        }
                                    }
                                    //cause some randomness by sometimes turning opposite way
                                    if right_cell == MapCell::Terrain(Terrain::Field) {
                                        if random::<bool>() {
                                            self.last_rotation =
                                                if self.last_rotation == Rotation::Clockwise {
                                                    Rotation::CounterClockwise
                                                } else {
                                                    Rotation::Clockwise
                                                }
                                        }
                                    }

                                    return Action::Rotate(self.last_rotation.clone());
                                }
                            };
                        }
                        MapCell::Unallocated => {
                            self.last_rotation = Rotation::Clockwise;
                            return Action::Rotate(Rotation::Clockwise);
                        }
                        _ => {}
                    };
                } else {
                    match cell {
                        MapCell::Player(player_details, _) => {
                            if player_details.alive
                                && player_details.id != context.player_details().id
                            {
                                return Action::Fire(Aiming::Cardinal(
                                    context.player_details().orientation,
                                ));
                            }
                        }
                        _ => {}
                    };
                }
            }
        } else if looking_east_west {
            //straight ahead has same y coordinate
            let scan_height = scan_result.data.len();
            let scan_y_row = scan_result.data[scan_height / 2];
            for scan_x in 0..scan_y_row.len() {
                let cell = scan_y_row[scan_x];

                if (scan_x == 1 && context.player_details().orientation == Orientation::East)
                    || (scan_x == scan_y_row.len() - 2
                        && context.player_details().orientation == Orientation::West)
                {
                    let upper_cell = scan_result.data[scan_height / 2 - 1][scan_x];
                    let lower_cell = scan_result.data[scan_height / 2 + 1][scan_x];
                    match cell {
                        MapCell::Terrain(t) => {
                            match t {
                                Terrain::Field => {}
                                _ => {
                                    if context.player_details().orientation == Orientation::East {
                                        if upper_cell == MapCell::Terrain(Terrain::Field) {
                                            self.last_rotation = Rotation::CounterClockwise;
                                        } else {
                                            self.last_rotation = Rotation::Clockwise;
                                        }
                                    }
                                    if context.player_details().orientation == Orientation::West {
                                        if upper_cell == MapCell::Terrain(Terrain::Field) {
                                            self.last_rotation = Rotation::Clockwise;
                                        } else {
                                            self.last_rotation = Rotation::CounterClockwise;
                                        }
                                    }
                                    //cause some randomness by sometimes turning opposite way
                                    if lower_cell == MapCell::Terrain(Terrain::Field) {
                                        if random::<bool>() {
                                            self.last_rotation =
                                                if self.last_rotation == Rotation::Clockwise {
                                                    Rotation::CounterClockwise
                                                } else {
                                                    Rotation::Clockwise
                                                }
                                        }
                                    }
                                    return Action::Rotate(self.last_rotation.clone());
                                }
                            };
                        }
                        MapCell::Unallocated => {
                            self.last_rotation = Rotation::Clockwise;
                            return Action::Rotate(Rotation::Clockwise);
                        }
                        _ => {}
                    };
                }
                match cell {
                    MapCell::Player(player_details, _) => {
                        if player_details.alive && player_details.id != context.player_details().id
                        {
                            return Action::Fire(Aiming::Cardinal(
                                context.player_details().orientation,
                            ));
                        }
                    }
                    _ => {}
                };
            }
        } else {
            //not handled yet, turn 45 degrees
            return Action::Rotate(self.last_rotation.clone());
        }

        Action::Move(Direction::Forward)
    }

    fn check_scan(&mut self, context: &Context) -> Action {
        if let Some(scan_result) = context.scanned_data() {
            let next = match scan_result.scan_type {
                ScanType::Mono(_) => self.handle_mono_scan(scan_result, context),
                ScanType::Omni => self.handle_omni_scan(scan_result, context),
            };
            return next;
        } else {
            Action::Scan(ScanType::Omni)
        }
    }
}

impl Player for Swede {
    fn act(&mut self, context: Context) -> Action {
        let next = match context.previous_action() {
            Action::Idle => Action::Scan(ScanType::Mono(context.player_details().orientation)),
            Action::Fire(_) => Action::Scan(ScanType::Mono(context.player_details().orientation)),
            Action::Move(_) => Action::Fire(Aiming::Cardinal(context.player_details().orientation)),
            Action::Rotate(_) => {
                Action::Fire(Aiming::Cardinal(context.player_details().orientation))
            }
            Action::Scan(_) => self.check_scan(&context),
        };

        return next;
    }

    fn is_ready(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        "The Swede".to_string()
    }
}
