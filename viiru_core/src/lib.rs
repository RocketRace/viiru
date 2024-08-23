mod api;
mod blocks;
mod spec;
mod ui;

use std::{thread, time::Duration};

use neon::prelude::*;
use api::*;
use ui::in_terminal_scope;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;

    in_terminal_scope(|w, h| {
        load_project(&mut cx, api, "example/empty.sb3")?;
        for (i, opcode) in blocks::OPCODES.iter().enumerate() {
            let id = i.to_string();
            create_block_template(&mut cx, api, opcode, Some(&id))?;
            slide_block(&mut cx, api, &id, 100.0, (i as f64) * 100.0)?;
        }
        save_project(&mut cx, api, "example/output.sb3")?;
        thread::sleep(Duration::from_millis(5000));
        Ok(cx.undefined())
    });


    Ok(cx.undefined())
}
