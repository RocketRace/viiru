mod block;
mod bridge;
mod opcodes;
mod result;
mod runtime;
mod spec;
mod ui;
mod util;

use std::io::{stdout, Write};

use crossterm::{
    event::{read, KeyCode, KeyEventKind},
    execute, queue,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use opcodes::OPCODES;
use runtime::Runtime;
use ui::{draw_block, in_terminal_scope};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;
    let mut state = Runtime::new(&mut cx, api);

    let result = in_terminal_scope(|| {
        state.load_project("example/empty.sb3")?;
        execute!(stdout(), Clear(ClearType::All))?;

        for (i, opcode) in OPCODES.iter().enumerate() {
            let (id, _) = state.create_block_template(opcode)?;
            state.slide_block(&id, 100, (i as i32) * 100)?;
        }

        let WindowSize { columns, rows, .. } = window_size()?;

        state.viewport.x_max = columns as i32;
        state.viewport.y_max = rows as i32;

        let mut cursor_x = 0;
        let mut cursor_y = 0;

        let (start, _) = state.create_block_template("event_whenflagclicked")?;

        let (iff, _) = state.create_block_template("control_if_else")?;
        state.attach_next(&iff, &start)?;

        let (cond, cond_children) = state.create_block_template("sensing_touchingcolor")?;
        state.attach_input(&cond, &iff, "CONDITION", false)?;
        state.set_field(&cond_children[0], "COLOUR", "#FF0000", None)?;

        let (motion, _) = state.create_block_template("motion_movesteps")?;
        state.attach_input(&motion, &iff, "SUBSTACK", false)?;

        let (op, op_children) = state.create_block_template("operator_add")?;
        state.attach_input(&op, &motion, "STEPS", false)?;
        state.set_strumber_field(&op_children[0], "12.3")?;

        let (hide, _) = state.create_block_template("looks_hide")?;
        state.attach_next(&hide, &motion)?;

        let (show, _) = state.create_block_template("looks_show")?;
        state.attach_next(&show, &hide)?;

        loop {
            queue!(stdout(), Clear(ClearType::All))?;
            draw_block(&state, &start, cursor_x, cursor_y)?;
            stdout().flush()?;
            match read()? {
                crossterm::event::Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char('q') => {
                                state.save_project("example/output.sb3")?;
                                break;
                            }
                            KeyCode::Char('h') => cursor_x -= 1,
                            KeyCode::Char('j') => cursor_y += 1,
                            KeyCode::Char('k') => cursor_y -= 1,
                            KeyCode::Char('l') => cursor_x += 1,
                            _ => (),
                        }
                    }
                }
                crossterm::event::Event::Resize(w, h) => {
                    state.viewport.x_max = w as i32;
                    state.viewport.y_max = h as i32;
                }
                _ => (),
            }
        }

        Ok(())
    });

    state.undefined_or_throw(result)
}
