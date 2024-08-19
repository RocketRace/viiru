use std::{thread, time::Duration};

use neon::{prelude::*, types::function::Arguments};

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("main", tui_main)?;
    Ok(())
}

// neon seems to be a pretty barebones library with not a lot of sugar.
// but it's too late to change now
// oh wait I just noticed there's a serde feature. oh well too late now
fn api_call<'a, A, R>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    function_name: &str,
    args: A,
) -> JsResult<'a, R>
where
    A: Arguments<'a>,
    R: Value,
{
    api.get::<JsFunction, _, _>(cx, function_name)?
        .call_with(cx)
        .args(args)
        .apply(cx)
}

// hide the .as_value(cx) calls everywhere
macro_rules! args {
    ($cx: expr; $($arg: expr),*$(,)?) => {
        ($($arg.as_value($cx)),*,)
    };
}

fn load_project<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    path: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(path));
    let result = api_call(cx, api, "loadProject", args)?;
    // what's a race condition
    Ok(result)
}

fn save_project<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    path: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(path));
    let result = api_call(cx, api, "saveProject", args)?;
    Ok(result)
}

fn create_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    opcode: &str,
    id: Option<&str>,
) -> JsResult<'a, JsUndefined> {
    let args = id.map_or(
        // this is one way to do it, comment "uwu" if you prefer this one
        args!(cx; cx.string(opcode), cx.undefined()),
        |id| args!(cx; cx.string(opcode), cx.string(id)),
    );
    api_call(cx, api, "createBlock", args)
}

fn delete_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "deleteBlock", args)
}

fn slide_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
    x: f64,
    y: f64,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id), cx.number(x), cx.number(y));
    api_call(cx, api, "slideBlock", args)
}

fn attach_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
    parent_id: &str,
    input_name: Option<&str>,
) -> JsResult<'a, JsUndefined> {
    let args = args!(
        cx; cx.string(id), cx.string(parent_id),
        // this is another way, comment "awa" if you prefer this one
        if let Some(input_name) = input_name {
            cx.string(input_name).as_value(cx)
        } else {
            cx.undefined().as_value(cx)
        }
    );
    api_call(cx, api, "attachBlock", args)
}

fn detach_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "detachBlock", args)
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

    load_project(&mut cx, api, "example/cg.sb3")?;
    create_block(&mut cx, api, "event_whenflagclicked", Some("starting"))?;
    slide_block(&mut cx, api, "starting", 35.0, 35.0)?;
    create_block(&mut cx, api, "control_if", Some("if"))?;
    create_block(&mut cx, api, "looks_sayforsecs", Some("speak"))?;
    attach_block(&mut cx, api, "if", "starting", None)?;
    attach_block(&mut cx, api, "speak", "if", Some("SUBSTACK"))?;
    create_block(&mut cx, api, "operator_equals", Some("cond!!"))?;
    attach_block(&mut cx, api, "cond!!", "if", Some("CONDITION"))?;
    save_project(&mut cx, api, "example/cg2.sb3")?;


    Ok(cx.undefined())
}
