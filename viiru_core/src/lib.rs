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
use runtime::{Runtime, State};
use ui::{
    draw_block, draw_cursor, draw_cursor_lines, draw_marker_dots, draw_toolbox,
    draw_viewport_border, in_terminal_scope,
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

        // initialize toolbox
        runtime.do_sync = false;
        for opcode in OPCODES.iter() {
            let (id, _) = runtime.create_block_template(opcode)?;
            runtime.remove_top_level(&id);
            runtime.toolbox.push(id);
        }
        runtime.do_sync = true;

        let (add, _) = runtime.create_block_template("operator_add")?;
        let (child, _) = runtime.create_block_template("operator_subtract")?;
        runtime.attach_input(&child, &add, "NUM1", false)?;

        let viewport_offset_x = 3;
        let viewport_offset_y = 1;
        let toolbox_width = 45;
        let status_height = 5;

        runtime.scroll_x = -viewport_offset_x;
        runtime.scroll_y = -viewport_offset_y;

        let WindowSize { columns, rows, .. } = window_size()?;

        runtime.window_cols = columns;
        runtime.window_rows = columns;
        runtime.viewport.x_min = viewport_offset_x;
        runtime.viewport.x_max = columns as i32 - toolbox_width;
        runtime.viewport.y_min = viewport_offset_y;
        runtime.viewport.y_max = rows as i32 - status_height;

        // center the view on 0, 0
        runtime.scroll_x -= (runtime.viewport.x_max - runtime.viewport.x_min) / 2;
        runtime.scroll_y -= (runtime.viewport.y_max - runtime.viewport.y_min) / 2;

        let mut needs_refresh = true;
        loop {
            if needs_refresh {
                queue!(stdout(), Clear(ClearType::All))?;
                draw_viewport_border(&runtime)?;
                draw_marker_dots(&runtime)?;
                draw_cursor_lines(&runtime)?;
                // TODO: implement some form of culling
                let mut placement_grid = HashMap::new();
                for top_id in &runtime.top_level {
                    // draw the cursor block last so it's always on top
                    let is_cursor = if let Some(cursor_id) = &runtime.cursor_block {
                        cursor_id == top_id
                    } else {
                        false
                    };
                    if !is_cursor {
                        draw_block(
                            &runtime,
                            top_id,
                            runtime.blocks[top_id].x,
                            runtime.blocks[top_id].y,
                            &mut placement_grid,
                            false,
                        )?;
                    }
                }
                if let Some(cursor_id) = &runtime.cursor_block {
                    draw_block(
                        &runtime,
                        cursor_id,
                        runtime.blocks[cursor_id].x,
                        runtime.blocks[cursor_id].y,
                        &mut placement_grid,
                        false,
                    )?;
                }
                runtime.placement_grid = placement_grid;
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

                draw_toolbox(&mut runtime, viewport_offset_x, viewport_offset_y, false)?;

                stdout().flush()?;
                needs_refresh = false;
            }

            match read()? {
                crossterm::event::Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char('q') => {
                                runtime.save_project("example/output.sb3")?;
                                break;
                            }
                            KeyCode::Char('h') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.move_x(-1)?;
                                    if runtime.cursor_x - runtime.scroll_x
                                        == runtime.viewport.y_min - 1
                                    {
                                        runtime.scroll_x -= 1;
                                    }
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('j') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.move_y(1)?;
                                    if runtime.cursor_y - runtime.scroll_y == runtime.viewport.y_max
                                    {
                                        runtime.scroll_y += 1;
                                    }
                                    needs_refresh = true;
                                }
                                State::Toolbox => {
                                    runtime.toolbox_cursor =
                                        (runtime.toolbox_cursor + 1).min(runtime.toolbox.len() - 1);
                                    while runtime.toolbox_cursor > runtime.toolbox_visible_max {
                                        runtime.toolbox_scroll = (runtime.toolbox_scroll + 1)
                                            .min(runtime.toolbox.len() - 1);
                                        draw_toolbox(
                                            &mut runtime,
                                            viewport_offset_x,
                                            viewport_offset_y,
                                            true,
                                        )?;
                                    }
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('k') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.move_y(-1)?;
                                    if runtime.cursor_y - runtime.scroll_y < runtime.viewport.y_min
                                    {
                                        runtime.scroll_y -= 1;
                                    }
                                    needs_refresh = true;
                                }
                                State::Toolbox => {
                                    runtime.toolbox_cursor =
                                        runtime.toolbox_cursor.saturating_sub(1);
                                    if runtime.toolbox_cursor == runtime.toolbox_scroll - 1 {
                                        runtime.toolbox_scroll = runtime.toolbox_cursor;
                                    }
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('l') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.move_x(1)?;
                                    if runtime.cursor_x - runtime.scroll_x == runtime.viewport.x_max
                                    {
                                        runtime.scroll_x += 1;
                                    }
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('H') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.scroll_x -= 1;
                                    runtime.move_x(-1)?;
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('J') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.scroll_y += 1;
                                    runtime.move_y(1)?;
                                    needs_refresh = true;
                                }
                                State::Toolbox => {
                                    runtime.toolbox_cursor =
                                        (runtime.toolbox_cursor + 1).min(runtime.toolbox.len() - 1);
                                    runtime.toolbox_scroll =
                                        (runtime.toolbox_scroll + 1).min(runtime.toolbox.len() - 1);
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('K') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.scroll_y -= 1;
                                    runtime.move_y(-1)?;
                                    needs_refresh = true;
                                }
                                State::Toolbox => {
                                    runtime.toolbox_cursor =
                                        runtime.toolbox_cursor.saturating_sub(1);
                                    runtime.toolbox_scroll =
                                        runtime.toolbox_scroll.saturating_sub(1);
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('L') => match runtime.state {
                                State::Move | State::Hold => {
                                    runtime.scroll_x += 1;
                                    runtime.move_x(1)?;
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('t') => match runtime.state {
                                State::Move => {
                                    runtime.state = State::Toolbox;
                                    needs_refresh = true;
                                }
                                State::Toolbox => {
                                    runtime.state = State::Move;
                                    needs_refresh = true;
                                }
                                _ => (),
                            },
                            KeyCode::Char('s') => match runtime.state {
                                State::Move => {
                                    if let Some(a) = runtime
                                        .placement_grid
                                        .get(&(runtime.cursor_x, runtime.cursor_y))
                                    {
                                        let selected = a.last().unwrap().clone();
                                        let stamp_id = runtime.stamp_block(&selected, true)?;
                                        runtime.put_to_cursor(&stamp_id)?;
                                        needs_refresh = true;
                                        runtime.state = State::Hold;
                                    }
                                }
                                State::Hold => {
                                    if let Some(cursor_id) = runtime.cursor_block.clone() {
                                        runtime.stamp_block(&cursor_id, true)?;
                                        needs_refresh = true;
                                    }
                                }
                                _ => (),
                            },
                            KeyCode::Char('D') => match runtime.state {
                                State::Move => {
                                    if let Some(a) = runtime
                                        .placement_grid
                                        .get(&(runtime.cursor_x, runtime.cursor_y))
                                    {
                                        let selected = a.last().unwrap().clone();
                                        runtime.delete_block(&selected)?;
                                        needs_refresh = true;
                                    }
                                }
                                State::Hold => {
                                    if let Some(cursor_id) = runtime.cursor_block.take() {
                                        runtime.delete_block(&cursor_id)?;
                                        needs_refresh = true;
                                        runtime.state = State::Move;
                                    }
                                }
                                _ => (),
                            },
                            KeyCode::Char(' ') => {
                                // interaction!
                                match runtime.state {
                                    State::Move => {
                                        if let Some(a) = runtime
                                            .placement_grid
                                            .get(&(runtime.cursor_x, runtime.cursor_y))
                                        {
                                            let selected = a.last().unwrap().clone();
                                            runtime.put_to_cursor(&selected)?;
                                            needs_refresh = true;
                                            runtime.state = State::Hold;
                                        }
                                    }
                                    State::Hold => {
                                        // TODO: attach to drop-off points
                                        runtime.cursor_block.take().unwrap();
                                        runtime.state = State::Move;
                                    }
                                    State::Toolbox => {
                                        let toolbox_id =
                                            runtime.toolbox[runtime.toolbox_cursor].clone();
                                        let spawned_id = runtime.stamp_block(&toolbox_id, true)?;
                                        runtime.put_to_cursor(&spawned_id)?;
                                        needs_refresh = true;
                                        runtime.state = State::Hold;
                                    }
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                }
                crossterm::event::Event::Resize(new_columns, new_rows) => {
                    runtime.window_cols = new_columns;
                    runtime.window_rows = new_rows;
                    runtime.viewport.x_min = viewport_offset_x;
                    runtime.viewport.x_max = new_columns as i32 - toolbox_width;
                    runtime.viewport.y_min = viewport_offset_y;
                    runtime.viewport.y_max = new_rows as i32 - status_height;
                    needs_refresh = true;
                }
                _ => (),
            }
        }

        Ok(())
    });

    runtime.undefined_or_throw(result)
}
