use std::io::stdout;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};

use crate::result::ViiruResult;

pub fn in_terminal_scope<F>(f: F) -> ViiruResult
where
    F: FnOnce() -> ViiruResult,
{
    stdout().execute(EnterAlternateScreen)?.execute(Hide)?;
    enable_raw_mode()?;
    f()?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?.execute(Show)?;
    Ok(())
}

pub fn print_size(columns: u16, rows: u16) -> ViiruResult {
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print(format!("hi there, {columns}x{rows} terminal!"))
    )?;
    Ok(())
}
