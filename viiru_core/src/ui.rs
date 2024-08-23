use std::io::stdout;

use crossterm::{
    terminal::{
        disable_raw_mode, enable_raw_mode, window_size, EnterAlternateScreen, LeaveAlternateScreen,
        WindowSize,
    },
    ExecutableCommand,
};

use crate::result::ViiruResult;

pub fn in_terminal_scope<F>(f: F) -> ViiruResult
where
    F: FnOnce(u16, u16) -> ViiruResult,
{
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let WindowSize { columns, rows, .. } = window_size()?;
    f(columns, rows)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
