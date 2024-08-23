use std::io::stdout;

use crossterm::{
    cursor::{Hide, Show},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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
