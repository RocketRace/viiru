mod api;
mod blocks;
mod result;
mod spec;
mod ui;

use std::{io::stdout, thread, time::Duration};

use api::*;
use crossterm::{
    cursor::MoveTo,
    event::{read, KeyCode, KeyEventKind},
    execute,
    style::Print,
    terminal::{window_size, Clear, ClearType, WindowSize},
    ExecutableCommand,
};
use neon::prelude::*;
use result::return_or_throw;
use ui::in_terminal_scope;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;

    let result = in_terminal_scope(|| {
        load_project(&mut cx, api, "example/empty.sb3")?;

        let WindowSize {
            mut columns,
            mut rows,
            ..
        } = window_size()?;
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        for (i, opcode) in blocks::OPCODES.iter().enumerate() {
            let id = i.to_string();
            create_block_template(&mut cx, api, opcode, Some(&id))?;
            slide_block(&mut cx, api, &id, 100.0, (i as f64) * 100.0)?;
        }
        save_project(&mut cx, api, "example/output.sb3")?;

        loop {
            match read()? {
                crossterm::event::Event::Key(event) => {
                    if event.kind == KeyEventKind::Press && event.code == KeyCode::Char('q') {
                        break;
                    }
                }
                crossterm::event::Event::Resize(w, h) => {
                    columns = w;
                    rows = h;
                }
                _ => (),
            }
        }

        stdout().execute(Print(format!("hi there, {columns}x{rows} terminal!")))?;
        thread::sleep(Duration::from_secs(2));
        Ok(())
    });

    return_or_throw(&mut cx, result)
}
