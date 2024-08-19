use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

enum EditorOp {
    LoadProject(String),
    SaveProject(String),
    CreateBlock((), Option<String>),
    DeleteBlock(String),
    SlideBlock(String, i32, i32),
    AttachBlock(String, String, Option<String>),
    DetachBlock(String),
    ChangeField(String, String, String),
    ChangeMutation(String, ()),
    ChangeCheckbox(String, bool),
}

fn tui_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let api = cx.argument::<JsObject>(0)?;

    println!("testing testing");

    Ok(cx.undefined())
}
