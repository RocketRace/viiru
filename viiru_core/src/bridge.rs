use std::collections::HashMap;

use neon::{prelude::*, types::function::Arguments};

use crate::block::{Block, Field, Input};

pub fn string_of(cx: &mut FunctionContext, s: Handle<JsString>) -> String {
    String::from_utf16(&s.to_utf16(cx)).unwrap()
}

fn str_value<'js>(
    cx: &mut FunctionContext<'js>,
    object: Handle<'js, JsObject>,
    key: &str,
) -> NeonResult<String> {
    let handle: Handle<JsString> = object.get(cx, key)?;
    Ok(string_of(cx, handle))
}

// funny that this is the exact same implementation, but `.get` isn't polymorphic over self
fn str_index<'js>(
    cx: &mut FunctionContext<'js>,
    array: Handle<'js, JsArray>,
    i: u32,
) -> NeonResult<String> {
    let handle: Handle<JsString> = array.get(cx, i)?;
    Ok(string_of(cx, handle))
}

fn optional_str_value<'js>(
    cx: &mut FunctionContext<'js>,
    object: Handle<'js, JsObject>,
    key: &str,
) -> NeonResult<Option<String>> {
    let value = object.get_value(cx, key)?;
    if value.is_a::<JsString, _>(cx) {
        str_value(cx, object, key).map(Some)
    } else {
        Ok(None)
    }
}

fn num_value<'js>(
    cx: &mut FunctionContext<'js>,
    object: Handle<'js, JsObject>,
    key: &str,
) -> NeonResult<f64> {
    let value: Handle<JsNumber> = object.get(cx, key)?;
    Ok(value.value(cx))
}

pub fn map_each_value<'js, F, V, R>(
    cx: &mut FunctionContext<'js>,
    object: Handle<'js, JsObject>,
    mut f: F,
) -> NeonResult<HashMap<String, R>>
where
    F: FnMut(&mut FunctionContext<'js>, Handle<'js, V>) -> NeonResult<R>,
    V: Value,
{
    let keys = object.get_own_property_names(cx)?;
    let length = keys.len(cx);
    let mut output = HashMap::new();
    for i in 0..length {
        let key = str_index(cx, keys, i)?;
        let value: Handle<V> = object.get(cx, key.as_str())?;
        let result = f(cx, value)?;
        output.insert(key, result);
    }
    Ok(output)
}

pub fn to_block<'js>(
    cx: &mut FunctionContext<'js>,
    object: Handle<'js, JsObject>,
) -> NeonResult<Block> {
    let id = str_value(cx, object, "id")?;
    let opcode = str_value(cx, object, "opcode")?;
    let parent_id = optional_str_value(cx, object, "parent")?;
    let next_id = optional_str_value(cx, object, "next")?;

    let inputs_obj: Handle<JsObject> = object.get(cx, "inputs")?;
    let inputs = map_each_value(cx, inputs_obj, |cx, input| {
        let block_id = optional_str_value(cx, input, "block")?;
        let shadow_id = optional_str_value(cx, input, "shadow")?;
        Ok(Input {
            shadow_id,
            block_id,
        })
    })?;

    let fields_obj: Handle<JsObject> = object.get(cx, "fields")?;
    let fields = map_each_value(cx, fields_obj, |cx, field| {
        // used for vars / lists / bcs
        let id = optional_str_value(cx, field, "id")?;
        let text = str_value(cx, field, "value")?;
        Ok(Field { text, id })
    })?;

    // 50px per cell
    let x = num_value(cx, object, "x")? / 50.0;
    let y = num_value(cx, object, "y")? / 50.0;

    Ok(Block {
        x: x as i32,
        y: y as i32,
        offset_x: 0,
        offset_y: 0,
        id,
        opcode,
        parent_id,
        next_id,
        inputs,
        fields,
    })
}

// neon seems to be a pretty barebones library with not a lot of sugar.
// but it's too late to change now
// oh wait I just noticed there's a serde feature. oh well too late now
fn api_call<'js, A, R>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    function_name: &str,
    args: A,
) -> JsResult<'js, R>
where
    A: Arguments<'js>,
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

pub fn load_project<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    path: &str,
) -> JsResult<'js, JsUndefined> {
    let args = args!(cx; cx.string(path));
    let result = api_call(cx, api, "loadProject", args)?;
    // what's a race condition
    Ok(result)
}

pub fn save_project<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    path: &str,
) -> JsResult<'js, JsUndefined> {
    let args = args!(cx; cx.string(path));
    let result = api_call(cx, api, "saveProject", args)?;
    Ok(result)
}

// Returns the block ID on success.
pub fn create_block<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    opcode: &str,
    is_shadow: bool,
    id: Option<&str>,
) -> JsResult<'js, JsString> {
    let args = id.map_or(
        // this is the wrong way to do it, comment "uwu" below if you agree
        args!(cx; cx.string(opcode), cx.boolean(is_shadow), cx.undefined()),
        |id| args!(cx; cx.string(opcode), cx.boolean(is_shadow), cx.string(id)),
    );
    api_call(cx, api, "createBlock", args)
}

pub fn delete_block<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'js, JsUndefined> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "deleteBlock", args)
}

pub fn slide_block<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    id: &str,
    x: i32,
    y: i32,
) -> JsResult<'js, JsUndefined> {
    // 50px per cell
    let x = x as f64 * 50.0;
    let y = y as f64 * 50.0;
    let args = args!(cx; cx.string(id), cx.number(x), cx.number(y));
    api_call(cx, api, "slideBlock", args)
}

pub fn attach_block<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    id: &str,
    parent_id: &str,
    input_name: Option<&str>,
    is_shadow: bool,
) -> JsResult<'js, JsUndefined> {
    let args = args!(
        cx; cx.string(id), cx.string(parent_id),
        // this is the right way, comment "awa" if you prefer this one
        if let Some(input_name) = input_name {
            cx.string(input_name).as_value(cx)
        } else {
            cx.undefined().as_value(cx)
        },
        cx.boolean(is_shadow)
    );
    api_call(cx, api, "attachBlock", args)
}

pub fn detach_block<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    id: &str,
) -> JsResult<'js, JsUndefined> {
    let args = args!(cx; cx.string(id));
    api_call(cx, api, "detachBlock", args)
}

pub fn change_field<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    block_id: &str,
    field: &str,
    text: &str,
    data_id: Option<&str>,
) -> JsResult<'js, JsUndefined> {
    let args = args!(cx; cx.string(block_id), cx.string(field), cx.string(text), if let Some(data_id) = data_id {
        cx.string(data_id).as_value(cx)
    } else {
        cx.null().as_value(cx)
    });
    api_call(cx, api, "changeField", args)
}

// todo: ChangeMutation(String, ())

pub fn get_all_blocks<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
) -> JsResult<'js, JsObject> {
    api_call(cx, api, "getAllBlocks", ())
}

pub enum VariableType {
    Scalar,
    List,
    Broadcast,
}

pub fn get_variables_of_type<'js>(
    cx: &mut FunctionContext<'js>,
    api: Handle<JsObject>,
    variable_type: VariableType,
) -> JsResult<'js, JsObject> {
    let s = match variable_type {
        VariableType::Scalar => "",
        VariableType::List => "list",
        VariableType::Broadcast => "broadcast_msg",
    };
    let args = args!(cx; cx.string(s));
    api_call(cx, api, "getVariablesOfType", args)
}
