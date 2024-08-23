mod api;
mod block;
mod blocks;
mod result;
mod spec;
mod state;
mod ui;

use std::{collections::HashMap, io::stdout};

use api::*;
use block::{
    Block,
    Throption::{Given, Void},
};
use crossterm::{
    event::{read, KeyCode, KeyEventKind},
    execute,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use result::undefined_or_throw;
use state::State;
use ui::{draw_block, in_terminal_scope, print_size};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;

    let result = in_terminal_scope(|| {
        load_project(&mut cx, api, "example/empty.sb3")?;

        execute!(stdout(), Clear(ClearType::All))?;

        for (i, opcode) in blocks::OPCODES.iter().enumerate() {
            let id = i.to_string();
            create_block_template(&mut cx, api, opcode, Some(&id))?;
            slide_block(&mut cx, api, &id, 100.0, (i as f64) * 100.0)?;
        }
        save_project(&mut cx, api, "example/output.sb3")?;

        let WindowSize {
            mut columns,
            mut rows,
            ..
        } = window_size()?;

        // print_size(columns, rows)?;

        let mut state = State {
            blocks: HashMap::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
        };

        state.blocks.insert(
            "child".into(),
            Block {
                id: "child".into(),
                opcode: "math_number".into(),
                parent_id: Given("parent".into()),
                fields: HashMap::from_iter([("NUM".into(), "12.3".into())]),
                ..Default::default()
            },
        );

        state.blocks.insert(
            "parent".into(),
            Block {
                id: "parent".into(),
                opcode: "motion_movesteps".into(),
                input_ids: vec![(Some("child".into()), None)],
                ..Default::default()
            },
        );

        draw_block(&state, "parent", 5, 6)?;

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
                    // print_size(columns, rows)?;
                }
                _ => (),
            }
        }

        Ok(())
    });

    undefined_or_throw(&mut cx, result)
}
