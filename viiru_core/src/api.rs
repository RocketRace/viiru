use neon::{prelude::*, types::function::Arguments};

use crate::blocks::BLOCKS;

fn string_of(cx: &mut FunctionContext, s: Handle<JsString>) -> String {
    String::from_utf16(&s.to_utf16(cx)).unwrap()
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

pub fn load_project<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    path: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(path));
    let result = api_call(cx, api, "loadProject", args)?;
    // what's a race condition
    Ok(result)
}

pub fn save_project<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    path: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(path));
    let result = api_call(cx, api, "saveProject", args)?;
    Ok(result)
}

// Returns the block ID on success.
pub fn create_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    opcode: &str,
    is_shadow: bool,
    id: Option<&str>,
) -> JsResult<'a, JsString> {
    let args = id.map_or(
        // this is one way to do it, comment "uwu" if you prefer this one
        args!(cx; cx.string(opcode), cx.boolean(is_shadow), cx.undefined()),
        |id| args!(cx; cx.string(opcode), cx.boolean(is_shadow), cx.string(id)),
    );
    api_call(cx, api, "createBlock", args)
}

// special
pub fn create_block_template<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    opcode: &str,
    id: Option<&str>,
) -> JsResult<'a, JsString> {
    let id_handle = create_block(cx, api, opcode, false, id)?;
    let id = string_of(cx, id_handle);
    let spec = &BLOCKS[opcode];
    for frag in &spec.head {
        if let crate::spec::Fragment::StrumberInput(value, Some(default)) = frag {
            let child_id = match default {
                crate::spec::DefaultValue::Block(child_opcode) => {
                    let id_handle = create_block(cx, api, child_opcode, true, None)?;
                    string_of(cx, id_handle)
                }
                crate::spec::DefaultValue::Str(s) => {
                    let id_handle = create_block(cx, api, "text", true, None)?;
                    let id = string_of(cx, id_handle);
                    change_field(cx, api, &id, "TEXT", s)?;
                    id
                }
                crate::spec::DefaultValue::Num(n) => {
                    let id_handle = create_block(cx, api, "math_number", true, None)?;
                    let id = string_of(cx, id_handle);
                    change_field(cx, api, &id, "NUM", &n.to_string())?;
                    id
                }
                crate::spec::DefaultValue::Color((r, g, b)) => {
                    let id_handle = create_block(cx, api, "colour_picker", true, None)?;
                    let id = string_of(cx, id_handle);
                    let rgb_string = format!("#{r:X}{g:X}{b:X}");
                    change_field(cx, api, &id, "COLOUR", &rgb_string)?;
                    id
                }
            };
            attach_block(cx, api, &child_id, &id, Some(value))?;
        }
    }
    Ok(id_handle)
}

pub fn delete_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "deleteBlock", args)
}

pub fn slide_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
    x: f64,
    y: f64,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id), cx.number(x), cx.number(y));
    api_call(cx, api, "slideBlock", args)
}

pub fn attach_block<'a>(
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

pub fn detach_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "detachBlock", args)
}

pub fn change_field<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
    field: &str,
    value: &str,
) -> JsResult<'a, JsUndefined> {
    let args = args!(cx; cx.string(id), cx.string(field), cx.string(value));
    api_call(cx, api, "changeField", args)
}

// todo: ChangeMutation(String, ()),
// todo: ChangeCheckbox(String, bool),

pub fn get_block<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'a, JsValue> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "getBlock", args)
}

pub fn get_scripts<'a>(
    cx: &mut FunctionContext<'a>,
    api: Handle<JsObject>,
) -> JsResult<'a, JsArray> {
    api_call(cx, api, "getScripts", ())
}
