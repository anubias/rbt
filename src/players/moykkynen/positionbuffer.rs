use super::Position;

const BUFFER_SIZE: usize = 10;

// Very simple ring buffer for storing a limited amount of position data
#[derive(Clone, Debug)]
pub struct PositionBuffer {
    buffer: [Option<Position>; BUFFER_SIZE],
    index: usize,
    count: usize,
}

impl PositionBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [const { None }; BUFFER_SIZE], // const needed as Position does not implement the Copy trait
            index: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, position: Position) {
        self.buffer[self.index] = Some(position);
        self.index += 1;
        if self.index >= self.buffer.len() {
            self.index = 0;
        }
        if self.count < BUFFER_SIZE {
            self.count += 1;
        }
    }

    pub fn _last(&self) -> Option<Position> {
        if self.index == 0 {
            self.buffer[BUFFER_SIZE - 1].clone()
        } else {
            self.buffer[self.index - 1].clone()
        }
    }

    pub fn is_full(&self) -> bool {
        if self.count >= BUFFER_SIZE {
            return true;
        }
        return false;
    }

    pub fn all_equal(&self) -> bool {
        if self.count < 2 {
            return false;
        }
        let first_value = self.buffer[0].clone();
        self.buffer.iter().take(self.count).all(|x| *x == first_value)
    }

    pub fn _print(&self) {
        for entry in &self.buffer {
            if let Some(pos) = entry {
                print!("{}, ", pos);
            }
        }
        println!();
    }
}
