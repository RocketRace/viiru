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
    cursor::{Hide, MoveRight, MoveTo, MoveToColumn},
    event::{read, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use opcodes::TOOLBOX;
use runtime::{Runtime, State};
use ui::{
    draw_block, draw_cursor, draw_cursor_lines, draw_marker_dots, draw_toolbox,
    draw_viewport_border, in_terminal_scope, Accumulators,
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
        // todo: replace with a proper implementation of "new project"
        runtime.load_project("viiru_core/empty.sb3")?;
        execute!(stdout(), Clear(ClearType::All))?;

        // initialize toolbox
        runtime.do_sync = false;
        for opcode in TOOLBOX.iter() {
            let (id, _) = runtime.create_block_template(opcode)?;
            runtime.remove_top_level(&id);
            runtime.toolbox.push(id);
        }
        runtime.do_sync = true;

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
            // TODO: implement some form of culling & per-component refresh
            if needs_refresh {
                queue!(stdout(), Clear(ClearType::All), Hide)?;
                draw_viewport_border(&runtime)?;
                draw_marker_dots(&runtime)?;
                draw_cursor_lines(&runtime)?;
                let mut accumulators = Accumulators::default();
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
                            &mut accumulators,
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
                        &mut accumulators,
                        false,
                    )?;
                }
                runtime.process_accumulators(accumulators);
                // this must occur after accumulators are processed, i.e. drop points are registered
                if let Some((parent_id, input_name)) = runtime.current_drop_point() {
                    // todo
                }
                let position = format!("{},{}", runtime.cursor_x, runtime.cursor_y);
                queue!(
                    stdout(),
                    MoveTo(
                        runtime.viewport.x_max as u16 - position.len() as u16,
                        runtime.viewport.y_max as u16 + 1,
                    ),
                    Print(position),
                    MoveRight(1),
                    Print(runtime.last_command),
                )?;
                if let State::Command = runtime.state {
                    let command_prefix = match runtime.last_command {
                        'o' => "file path: ",
                        'w' => "output path: ",
                        _ => "",
                    };
                    queue!(
                        stdout(),
                        MoveToColumn(runtime.viewport.x_min as u16),
                        Print(command_prefix),
                        Print(&runtime.command_buffer)
                    )?;
                } else {
                    queue!(
                        stdout(),
                        MoveToColumn(runtime.viewport.x_min as u16),
                        Print(&runtime.status_message)
                    )?;
                }
                draw_toolbox(&mut runtime, viewport_offset_x, viewport_offset_y, false)?;
                needs_refresh = false;
                // cursor is always drawn last
                draw_cursor(&runtime)?;
                stdout().flush()?;
            }

            match read()? {
                crossterm::event::Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        // suboptimal ordering, probably
                        if let State::Command = runtime.state {
                            match event.code {
                                KeyCode::Enter => {
                                    let buf = std::mem::take(&mut runtime.command_buffer);
                                    match runtime.last_command {
                                        'o' => {
                                            if !runtime.load_project(&buf)? {
                                                runtime.status_message =
                                                    format!("Failed to open project file at {buf}");
                                            } else {
                                                runtime.status_message =
                                                    format!("Opened project file {buf}");
                                            }
                                        }
                                        'w' => {
                                            if !runtime.save_project(&buf)? {
                                                runtime.status_message = format!(
                                                    "Failed to write project file to {buf}"
                                                );
                                            } else {
                                                runtime.status_message =
                                                    format!("Saved project into {buf}")
                                            }
                                        }
                                        _ => (),
                                    }
                                    runtime.state = State::Move;
                                }
                                KeyCode::Esc => {
                                    runtime.command_buffer.clear();
                                    runtime.state = State::Move;
                                }
                                KeyCode::Backspace => {
                                    runtime.command_buffer.pop();
                                }
                                KeyCode::Char(c) => runtime.command_buffer.push(c),
                                _ => (),
                            }
                            needs_refresh = true;
                        } else {
                            if let KeyCode::Char(c) = event.code {
                                runtime.last_command = c;
                            }
                            match event.code {
                                KeyCode::Char('q') => {
                                    if runtime.is_dirty() {
                                        runtime.status_message =
                                            "Unsaved changes. (Q to force)".into();
                                        needs_refresh = true;
                                    } else {
                                        break;
                                    }
                                }
                                KeyCode::Char('Q') => {
                                    break;
                                }
                                KeyCode::Char('o') => {
                                    runtime.state = State::Command;
                                    needs_refresh = true;
                                }
                                KeyCode::Char('w') => {
                                    runtime.state = State::Command;
                                    needs_refresh = true;
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
                                        if runtime.cursor_y - runtime.scroll_y
                                            == runtime.viewport.y_max
                                        {
                                            runtime.scroll_y += 1;
                                        }
                                        needs_refresh = true;
                                    }
                                    State::Toolbox => {
                                        runtime.toolbox_cursor = (runtime.toolbox_cursor + 1)
                                            .min(runtime.toolbox.len() - 1);
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
                                        if runtime.cursor_y - runtime.scroll_y
                                            < runtime.viewport.y_min
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
                                        if runtime.cursor_x - runtime.scroll_x
                                            == runtime.viewport.x_max
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
                                        runtime.toolbox_cursor = (runtime.toolbox_cursor + 1)
                                            .min(runtime.toolbox.len() - 1);
                                        runtime.toolbox_scroll = (runtime.toolbox_scroll + 1)
                                            .min(runtime.toolbox.len() - 1);
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
                                            .block_positions
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
                                            .block_positions
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
                                                .block_positions
                                                .get(&(runtime.cursor_x, runtime.cursor_y))
                                            {
                                                let selected = a.last().unwrap().clone();
                                                runtime.put_to_cursor(&selected)?;
                                                needs_refresh = true;
                                                runtime.state = State::Hold;
                                            }
                                        }
                                        State::Hold => {
                                            if let Some((parent_id, input_name)) =
                                                runtime.current_drop_point()
                                            {
                                                let cursor_id =
                                                    runtime.cursor_block.take().unwrap();
                                                if let Some(input_name) = input_name {
                                                    // chuck existing inputs away to the right somewhere
                                                    if let Some(existing_id) = runtime.blocks
                                                        [&parent_id]
                                                        .inputs[&input_name]
                                                        .block_id
                                                        .clone()
                                                    {
                                                        runtime.detach_block(&existing_id)?;
                                                        // TODO: pick a more reasonable position
                                                        runtime.slide_block_by(
                                                            &existing_id,
                                                            1,
                                                            1,
                                                        )?;
                                                    }
                                                    runtime.attach_input(
                                                        &cursor_id,
                                                        &parent_id,
                                                        &input_name,
                                                        false,
                                                    )?;
                                                } else {
                                                    // todo: support sandwiching
                                                    if runtime.blocks[&parent_id].next_id.is_none()
                                                    {
                                                        runtime
                                                            .attach_next(&cursor_id, &parent_id)?;
                                                    } else {
                                                        runtime.status_message =
                                                            "Can't place between stacks yet".into()
                                                    }
                                                }
                                            } else {
                                                runtime.cursor_block.take().unwrap();
                                            }
                                            runtime.state = State::Move;
                                        }
                                        State::Toolbox => {
                                            let toolbox_id =
                                                runtime.toolbox[runtime.toolbox_cursor].clone();
                                            let spawned_id =
                                                runtime.stamp_block(&toolbox_id, true)?;
                                            runtime.put_to_cursor(&spawned_id)?;
                                            runtime.slide_block_to(
                                                &spawned_id,
                                                runtime.cursor_x,
                                                runtime.cursor_y,
                                            )?;
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
