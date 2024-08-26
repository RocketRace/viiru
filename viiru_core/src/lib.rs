mod block;
mod bridge;
mod opcodes;
mod result;
mod runtime;
mod spec;
mod ui;
mod util;

use std::io::stdout;

use crossterm::{
    event::{read, KeyCode, KeyEventKind},
    execute,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use runtime::{Runtime, State};
use ui::{draw_toolbox, in_terminal_scope, refresh_screen};

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

        runtime.viewport_offset_x = 3;
        runtime.viewport_offset_y = 1;
        runtime.toolbox_width = 45;
        runtime.status_height = 5;

        let WindowSize { columns, rows, .. } = window_size()?;

        runtime.set_viewport(columns, rows);
        runtime.initialize_scroll();

        refresh_screen(&mut runtime)?;
        let mut needs_refresh = false;
        loop {
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
                        } else if let State::Inline = runtime.state {
                            // ugly implementation
                            match event.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    runtime.state = State::Move;
                                    runtime.status_message = "".into();
                                    needs_refresh = true;
                                }
                                KeyCode::Backspace => {
                                    let mut field =
                                        runtime.get_strumber_field(&runtime.editing_shadow);
                                    field.pop();
                                    runtime.set_strumber_field(
                                        &runtime.editing_shadow.clone(),
                                        &field,
                                    )?;
                                    needs_refresh = true;
                                }
                                KeyCode::Char(c) => {
                                    let mut field =
                                        runtime.get_strumber_field(&runtime.editing_shadow);
                                    field.push(c);
                                    runtime.set_strumber_field(
                                        &runtime.editing_shadow.clone(),
                                        &field,
                                    )?;
                                    needs_refresh = true;
                                }
                                _ => (),
                            }
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
                                            let vox = runtime.viewport_offset_x;
                                            let voy = runtime.viewport_offset_y;
                                            draw_toolbox(&mut runtime, vox, voy, true)?;
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
                                            } else if let Some(shadow_id) = runtime
                                                .writable_points
                                                .get(&(runtime.cursor_x, runtime.cursor_y))
                                            {
                                                runtime.editing_shadow = shadow_id.clone();
                                                runtime.state = State::Inline;
                                                runtime.status_message = "Editing field".into();
                                                needs_refresh = true;
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
                                                        if !runtime.blocks[&parent_id].is_boot() {
                                                            runtime.attach_next(
                                                                &cursor_id, &parent_id,
                                                            )?;
                                                        }
                                                    } else {
                                                        runtime.status_message =
                                                            "Can't place between stacks yet".into()
                                                    }
                                                }
                                            } else {
                                                runtime.cursor_block.take().unwrap();
                                            }
                                            needs_refresh = true;
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
                    runtime.set_viewport(new_columns, new_rows);
                    needs_refresh = true;
                }
                _ => (),
            }
            // TODO: implement some form of culling & per-component refresh
            if needs_refresh {
                refresh_screen(&mut runtime)?;
                needs_refresh = false;
            }
        }

        Ok(())
    });

    runtime.undefined_or_throw(result)
}
