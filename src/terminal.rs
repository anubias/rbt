use std::{
    fmt::Display,
    io::{stdout, Stdout, Write},
};

use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};

pub const CHAMPIONSHIP_MODE: bool = false;
pub const DEBUG_MODE: bool = false;

/// This structure should be used when trying to print anything to the console.
/// It encapsulates and wraps up behaviour needed to redraw the screen to avoid
/// continuous console buffer scrolling.
///
/// For debugging purposes, mostly when console output scrolling is seen as beneficial,
/// the `DEBUG_MODE` flag can be enabled, which will disable the wrapping in effect.
pub struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    pub fn new() -> Self {
        Self { stdout: stdout() }
    }

    /// This is meant for the game engine, players should avoid using it.
    pub fn enter_raw_mode() {
        if !CHAMPIONSHIP_MODE && !DEBUG_MODE {
            if enable_raw_mode().is_err() {
                println!("Unable to enter raw mode!")
            }
        }
    }

    /// This is meant for the game engine, players should avoid using it.
    pub fn exit_raw_mode() {
        if !CHAMPIONSHIP_MODE && !DEBUG_MODE {
            let _ = disable_raw_mode();
        }
    }

    pub fn clear_screen(&mut self) {
        if !CHAMPIONSHIP_MODE && !DEBUG_MODE {
            let _ = self.stdout.queue(Clear(ClearType::All));
            self.move_caret_to_origin();
        }
    }

    pub fn clear_below(&mut self) {
        if !CHAMPIONSHIP_MODE && !DEBUG_MODE {
            let _ = self.stdout.execute(Clear(ClearType::FromCursorDown));
        }
    }

    pub fn move_caret_to_origin(&mut self) {
        if !CHAMPIONSHIP_MODE && !DEBUG_MODE {
            let _ = self.stdout.execute(MoveTo(0, 0));
        }
    }

    pub fn println<T: Display>(&mut self, printable: T) {
        if !CHAMPIONSHIP_MODE {
            if DEBUG_MODE {
                println!("{printable}");
            } else {
                let _ = self.println_text(format!("{printable}"));
            }
        }
    }
}

// Private functions
impl Terminal {
    fn println_text(&mut self, text: String) {
        for line in text.split('\n').collect::<Vec<&str>>() {
            let _ = write!(self.stdout, "{line}");
            let _ = self.stdout.queue(Clear(ClearType::UntilNewLine));
            let _ = self.stdout.queue(MoveToNextLine(1));
        }

        let _ = self.stdout.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_championship_mode_is_off() {
        assert!(!CHAMPIONSHIP_MODE);
    }

    #[test]
    fn test_debug_mode_is_off() {
        assert!(!DEBUG_MODE);
    }
}
