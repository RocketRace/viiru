mod api;
mod block;
mod blocks;
mod result;
mod spec;
mod state;
mod ui;
mod util;

use std::{
    collections::HashMap,
    io::{stdout, Write},
};

use api::*;
use block::{Block, Throption::Given};
use crossterm::{
    event::{read, KeyCode, KeyEventKind},
    execute, queue,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use state::Runtime;
use ui::{draw_block, in_terminal_scope};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;
    let mut state = Runtime::init(&mut cx, api);

    let result = in_terminal_scope(|| {
        // load_project(&mut cx, api, "example/empty.sb3")?;
        execute!(stdout(), Clear(ClearType::All))?;

        // for (i, opcode) in blocks::OPCODES.iter().enumerate() {
        //     let id = i.to_string();
        //     create_block_template(&mut cx, api, opcode, Some(&id))?;
        //     slide_block(&mut cx, api, &id, 100.0, (i as f64) * 100.0)?;
        // }
        // save_project(&mut cx, api, "example/output.sb3")?;

        let WindowSize {
            mut columns,
            mut rows,
            ..
        } = window_size()?;

        let mut cursor_x = 0;
        let mut cursor_y = 0;

        state.blocks.insert(
            "start".into(),
            Block {
                id: "start".into(),
                opcode: "event_whenflagclicked".into(),
                next_id: Given("if".into()),
                ..Default::default()
            },
        );

        state.blocks.insert(
            "if".into(),
            Block {
                id: "if".into(),
                parent_id: Given("start".into()),
                opcode: "control_if_else".into(),
                input_ids: vec![
                    (None, Some("cond".into())),
                    (None, Some("parent".into())),
                    (None, None),
                ],
                ..Default::default()
            },
        );

        state.blocks.insert(
            "cond".into(),
            Block {
                id: "cond".into(),
                parent_id: Given("if".into()),
                opcode: "sensing_touchingcolor".into(),
                input_ids: vec![(None, Some("color".into()))],
                ..Default::default()
            },
        );

        state.blocks.insert(
            "color".into(),
            Block {
                id: "color".into(),
                parent_id: Given("cond".into()),
                opcode: "colour_picker".into(),
                fields: HashMap::from_iter([("COLOUR".into(), "FF0000".into())]),
                ..Default::default()
            },
        );

        state.blocks.insert(
            "parent".into(),
            Block {
                id: "parent".into(),
                parent_id: Given("if".into()),
                opcode: "motion_movesteps".into(),
                next_id: Given("hide".into()),
                input_ids: vec![(Some("op".into()), None)],
                ..Default::default()
            },
        );

        state.blocks.insert(
            "hide".into(),
            Block {
                id: "hide".into(),
                parent_id: Given("parent".into()),
                opcode: "looks_hide".into(),
                ..Default::default()
            },
        );

        state.blocks.insert(
            "op".into(),
            Block {
                id: "op".into(),
                parent_id: Given("parent".into()),
                opcode: "operator_add".into(),
                input_ids: vec![(Some("child".into()), None), (Some("empty".into()), None)],
                ..Default::default()
            },
        );

        state.blocks.insert(
            "child".into(),
            Block {
                id: "child".into(),
                parent_id: Given("op".into()),
                opcode: "math_number".into(),
                fields: HashMap::from_iter([("NUM".into(), "12.3".into())]),
                ..Default::default()
            },
        );

        state.blocks.insert(
            "empty".into(),
            Block {
                id: "empty".into(),
                parent_id: Given("op".into()),
                opcode: "text".into(),
                fields: HashMap::from_iter([("TEXT".into(), "".into())]),
                ..Default::default()
            },
        );

        loop {
            queue!(stdout(), Clear(ClearType::All))?;
            draw_block(&state, "start", cursor_x, cursor_y)?;
            stdout().flush()?;
            match read()? {
                crossterm::event::Event::Key(event) => {
                    if event.kind == KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('h') => cursor_x -= 1,
                            KeyCode::Char('j') => cursor_y += 1,
                            KeyCode::Char('k') => cursor_y -= 1,
                            KeyCode::Char('l') => cursor_x += 1,
                            _ => (),
                        }
                    }
                }
                crossterm::event::Event::Resize(w, h) => {
                    columns = w;
                    rows = h;
                }
                _ => (),
            }
        }

        Ok(())
    });

    state.undefined_or_throw(result)
}
