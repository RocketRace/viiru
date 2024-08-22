mod api;
mod blocks;
mod spec;

use std::sync::LazyLock;

use neon::prelude::*;
use api::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;
    dbg!(LazyLock::force(&blocks::BLOCKS));

    load_project(&mut cx, api, "example/empty.sb3")?;

    for (i, opcode) in blocks::OPCODES.iter().enumerate() {
        let id = i.to_string();
        create_block(&mut cx, api, opcode, Some(&id))?;
        slide_block(&mut cx, api, &id, 100.0, (i as f64) * 100.0)?;
    }
    save_project(&mut cx, api, "example/cg2.sb3")?;

    return Ok(cx.undefined());

    create_block(&mut cx, api, "event_whenflagclicked", Some("starting"))?;
    slide_block(&mut cx, api, "starting", 35.0, 35.0)?;
    create_block(&mut cx, api, "control_if", Some("if"))?;
    create_block(&mut cx, api, "looks_sayforsecs", Some("speak"))?;
    attach_block(&mut cx, api, "if", "starting", None)?;
    attach_block(&mut cx, api, "speak", "if", Some("SUBSTACK"))?;
    create_block(&mut cx, api, "operator_equals", Some("cond!!"))?;
    attach_block(&mut cx, api, "cond!!", "if", Some("CONDITION"))?;
    // accessing objects kind of sucks
    // if only there was some kind of serialization-deserialization library that could help...
    let b: Handle<JsObject> = get_block(&mut cx, api, "if")?.downcast_or_throw(&mut cx)?;
    let parent: Handle<JsString> = b.get(&mut cx, "parent")?;
    dbg!(parent.value(&mut cx));
    save_project(&mut cx, api, "example/cg2.sb3")?;

    Ok(cx.undefined())
}
