use std::io;

use crossterm::{
    ExecutableCommand,
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct Tui {
    stdout: io::Stdout,
    restored: bool,
}

impl Tui {
    pub fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;
        stdout.execute(cursor::Hide)?;

        Ok(Self {
            stdout,
            restored: false,
        })
    }

    pub fn stdout(&mut self) -> &mut io::Stdout {
        &mut self.stdout
    }

    pub fn restore(&mut self) -> io::Result<()> {
        if self.restored {
            return Ok(());
        }

        self.stdout.execute(cursor::Show)?;
        self.stdout.execute(DisableMouseCapture)?;
        self.stdout.execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        self.restored = true;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
