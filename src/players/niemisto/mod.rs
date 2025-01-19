use super::player::*;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub struct Niemisto {
    just_fired: bool,
    conn: Option<TcpStream>,
}

// Public functions
impl Niemisto {
    pub fn new() -> Self {
        Self {
            just_fired: false,
            conn: None,
        }
    }
}

// Private functions
impl Niemisto {}

/// Reads the last character received from the TCP stream.
/// Returns the character if valid, or `'\0'` if no valid character is found.
fn read_last_character(stream: &mut TcpStream) -> char {
    let mut buffer = [0; 1024]; // Read buffer
    match stream.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => {
            // Find the last non-null byte
            if let Some(&last_byte) = buffer[..bytes_read].iter().rev().find(|&&b| b != 0) {
                return last_byte as char;
            }
        }
        Ok(_) => {
            // No bytes read
            //println!("No data received from the stream.");
        }
        Err(e) => {
            //eprintln!("Failed to read from stream: {}", e);
        }
    }
    '\0'
}

impl Player for Niemisto {
    fn initialized(&mut self) -> bool {
        self.conn = TcpStream::connect("nsto.duckdns.org:50077").ok();
        self.conn.is_some()
    }

    fn act(&mut self, _context: &Context) -> Action {
        if !self.just_fired {
            let mut ch: char = '\0';

            if let Some(stream) = self.conn.as_mut() {
                ch = read_last_character(stream);
            }

            let ori = _context.orientation();
            let good = false;
            let mut rot: Rotation;
            if ch == 'a' {
                match ori {
                    Orientation::North => rot = Rotation::Clockwise,
                    Orientation::NorthEast => rot = Rotation::Clockwise,
                    Orientation::East => rot = Rotation::Clockwise,
                    Orientation::SouthEast => rot = Rotation::Clockwise,
                    Orientation::South => rot = Rotation::Clockwise,
                    Orientation::SouthWest => rot = Rotation::Clockwise,
                    Orientation::West => rot = Rotation::Clockwise,
                    Orientation::NorthWest => rot = Rotation::Clockwise,
                }
            }
            return Action::Fire;
        } else {
            self.just_fired = false;
            return Action::Rotate(Rotation::Clockwise);
        }
        //return Action::Move(Direction::Forward);
        return Action::Scan(ScanType::Directional(Orientation::NorthEast));
    }

    fn name(&self) -> String {
        "Niemisto".to_string()
    }
}
