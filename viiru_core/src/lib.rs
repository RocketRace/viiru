mod block;
mod bridge;
mod opcodes;
mod result;
mod runtime;
mod spec;
mod ui;
mod util;

use std::{
    collections::HashMap,
    io::{stdout, Write},
};

use crossterm::{
    cursor::MoveTo,
    event::{read, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use opcodes::OPCODES;
use runtime::Runtime;
use ui::{
    draw_block, draw_cursor, draw_cursor_lines, draw_marker_dots, draw_viewport_border,
    in_terminal_scope,
};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;
    let mut runtime = Runtime::new(&mut cx, api);

    let result = in_terminal_scope(|| {
        runtime.load_project("example/empty.sb3")?;
        execute!(stdout(), Clear(ClearType::All))?;

        for (i, opcode) in OPCODES.iter().enumerate() {
            let (id, _) = runtime.create_block_template(opcode)?;
            runtime.slide_block(&id, 100, (i as i32) * 100)?;
        }

        let viewport_offset_x = 3;
        let viewport_offset_y = 3;

        runtime.scroll_x = -viewport_offset_x;
        runtime.scroll_y = -viewport_offset_y;

        let WindowSize { columns, rows, .. } = window_size()?;

        runtime.viewport.x_min = viewport_offset_x;
        runtime.viewport.x_max = columns as i32 - 10;
        runtime.viewport.y_min = viewport_offset_y;
        runtime.viewport.y_max = rows as i32 - 5;

        // center the view on 0, 0
        runtime.scroll_x -= (runtime.viewport.x_max - runtime.viewport.x_min) / 2;
        runtime.scroll_y -= (runtime.viewport.y_max - runtime.viewport.y_min) / 2;

        // let (start, _) = state.create_block_template("event_whenflagclicked")?;

        // let (iff, _) = state.create_block_template("control_if_else")?;
        // state.attach_next(&iff, &start)?;

        // let (cond, cond_children) = state.create_block_template("sensing_touchingcolor")?;
        // state.attach_input(&cond, &iff, "CONDITION", false)?;
        // state.set_field(&cond_children[0], "COLOUR", "#FF0000", None)?;

        // let (motion, _) = state.create_block_template("motion_movesteps")?;
        // state.attach_input(&motion, &iff, "SUBSTACK", false)?;

        // let (op, op_children) = state.create_block_template("operator_add")?;
        // state.attach_input(&op, &motion, "STEPS", false)?;
        // state.set_strumber_field(&op_children[0], "12.3")?;

        // let (hide, _) = state.create_block_template("looks_hide")?;
        // state.attach_next(&hide, &motion)?;

        // let (show, _) = state.create_block_template("looks_show")?;
        // state.attach_next(&show, &hide)?;

        loop {
            queue!(stdout(), Clear(ClearType::All))?;
            // draw_block(&state, &start, cursor_x, cursor_y)?;
            draw_viewport_border(&runtime)?;
            draw_marker_dots(&runtime)?;
            draw_cursor_lines(&runtime)?;
            let mut placement_cache = HashMap::new();
            for top_id in &runtime.top_level {
                // if top_id != &start {
                draw_block(
                    &runtime,
                    top_id,
                    runtime.blocks[top_id].x / 50,
                    runtime.blocks[top_id].y / 50,
                    &mut placement_cache,
                )?;
                // }
            }
            runtime.placement_grid = placement_cache;
            draw_cursor(&runtime)?;
            let position = format!("{},{}", runtime.cursor_x, runtime.cursor_y);

            queue!(
                stdout(),
                MoveTo(
                    runtime.viewport.x_max as u16 - position.len() as u16 + 1,
                    runtime.viewport.y_max as u16 + 1,
                ),
                Print(position)
            )?;

            stdout().flush()?;
            match read()? {
                crossterm::event::Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char('q') => {
                                runtime.save_project("example/output.sb3")?;
                                break;
                            }
                            KeyCode::Char('h') => {
                                runtime.cursor_x -= 1;
                                if runtime.cursor_x - runtime.scroll_x == runtime.viewport.y_min - 1
                                {
                                    runtime.scroll_x -= 1;
                                }
                            }
                            KeyCode::Char('j') => {
                                runtime.cursor_y += 1;
                                if runtime.cursor_y - runtime.scroll_y == runtime.viewport.y_max {
                                    runtime.scroll_y += 1;
                                }
                            }
                            KeyCode::Char('k') => {
                                runtime.cursor_y -= 1;
                                if runtime.cursor_y - runtime.scroll_y == runtime.viewport.y_min - 1
                                {
                                    runtime.scroll_y -= 1;
                                }
                            }
                            KeyCode::Char('l') => {
                                runtime.cursor_x += 1;
                                if runtime.cursor_x - runtime.scroll_x == runtime.viewport.x_max {
                                    runtime.scroll_x += 1;
                                }
                            }
                            KeyCode::Char('H') => {
                                runtime.scroll_x -= 1;
                                runtime.cursor_x -= 1;
                            }
                            KeyCode::Char('J') => {
                                runtime.scroll_y += 1;
                                runtime.cursor_y += 1;
                            }
                            KeyCode::Char('K') => {
                                runtime.scroll_y -= 1;
                                runtime.cursor_y -= 1;
                            }
                            KeyCode::Char('L') => {
                                runtime.scroll_x += 1;
                                runtime.cursor_x += 1;
                            }
                            KeyCode::Char(' ') => {
                                // interaction!
                                if runtime
                                    .placement_grid
                                    .contains_key(&(runtime.cursor_x, runtime.cursor_y))
                                {
                                    runtime.cursor_x += 1;
                                }
                            }
                            _ => (),
                        }
                    }
                }
                crossterm::event::Event::Resize(w, h) => {
                    runtime.viewport.x_max = w as i32;
                    runtime.viewport.y_max = h as i32;
                }
                _ => (),
            }
        }

        Ok(())
    });

    runtime.undefined_or_throw(result)
}
