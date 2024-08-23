use std::io::stdout;

use crossterm::{
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, window_size, EnterAlternateScreen, LeaveAlternateScreen,
        WindowSize,
    },
};
use neon::prelude::*;

pub fn in_terminal_scope<'a, F>(f: F)
where
    F: FnOnce(u16, u16) -> JsResult<'a, JsUndefined>,
{
    execute!(stdout(), EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let WindowSize { columns, rows, .. } = window_size().unwrap();
    f(columns, rows).unwrap();
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen).unwrap();
}
