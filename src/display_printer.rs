use std::io::Write;

use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    terminal::Clear,
    ExecutableCommand, QueueableCommand,
};

pub struct DisplayPrinter {}

impl DisplayPrinter {
    pub fn clear() {
        let mut stdout = std::io::stdout();
        let _ = stdout.execute(Clear(crossterm::terminal::ClearType::All));
        let _ = stdout.execute(MoveTo(0, 0));
    }

    pub fn println(text: String) {
        let _ = Self::print(format!("{text}\n"));
    }

    pub fn println_str(str: &str) {
        Self::println(str.to_string());
    }
}

// Private functions
impl DisplayPrinter {
    fn print(text: String) {
        let mut stdout = std::io::stdout();

        let mut lines = text.split('\n').collect::<Vec<&str>>();
        let last = lines.pop();

        for line in lines {
            let _ = write!(stdout, "{line}");
            let _ = stdout.queue(MoveToNextLine(1));
        }

        if let Some(line) = last {
            let _ = write!(stdout, "{line}");
        }

        let _ = stdout.flush();
    }
}
