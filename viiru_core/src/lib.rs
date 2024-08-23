mod api;
mod block;
mod blocks;
mod result;
mod spec;
mod ui;

use std::{collections::HashMap, io::stdout};

use api::*;
use block::{Block, Colour, Expression, Kind, Throption};
use crossterm::{
    event::{read, KeyCode, KeyEventKind},
    execute,
    terminal::{window_size, Clear, ClearType, WindowSize},
};
use neon::prelude::*;
use result::undefined_or_throw;
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

        print_size(columns, rows)?;

        let block = Block {
            id: "123".into(),
            colour: Colour(255, 128, 0),
            opcode: "motion_xposition".into(),
            parent_id: Throption::Void,
            input_ids: vec![],
            fields: HashMap::new(),
            kind: Kind::Expression(Expression { shadow: false }),
        };

        draw_block(&block, 5, 6)?;

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
                    print_size(columns, rows)?;
                }
                _ => (),
            }
        }

        Ok(())
    });

    undefined_or_throw(&mut cx, result)
}
