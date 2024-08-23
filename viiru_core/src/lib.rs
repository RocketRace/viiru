mod api;
mod blocks;
mod result;
mod spec;
mod ui;

use std::{io::{stdout, Write}, thread, time::Duration};

use api::*;
use crossterm::{
    cursor::{Hide, MoveTo}, queue, style::Print, terminal::{Clear, ClearType},
};
use neon::prelude::*;
use result::ViiruError;
use ui::in_terminal_scope;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;

    let result = in_terminal_scope(|w, h| {
        load_project(&mut cx, api, "example/empty.sb3")?;

        queue!(stdout(), Clear(ClearType::All), Hide, MoveTo(0, 0), Print("hi there!"))?;
        stdout().flush()?;

        for (i, opcode) in blocks::OPCODES.iter().enumerate() {
            let id = i.to_string();
            create_block_template(&mut cx, api, opcode, Some(&id))?;
            slide_block(&mut cx, api, &id, 100.0, (i as f64) * 100.0)?;
        }
        save_project(&mut cx, api, "example/output.sb3")?;
        thread::sleep(Duration::from_millis(5000));
        Ok(())
    });

    match result {
        Ok(()) => Ok(cx.undefined()),
        Err(ViiruError::JsThrow(throw)) => Err(throw),
        Err(ViiruError::IoError(err)) => {
            cx.throw_error(err.to_string())
        }
    }
}
