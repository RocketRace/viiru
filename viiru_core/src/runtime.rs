use std::collections::HashMap;

use neon::prelude::*;

use crate::{
    block::Block,
    bridge,
    opcodes::BLOCKS,
    result::{undefined_or_throw, ViiruResult},
};

pub struct Runtime<'js, 'a> {
    next_usable_id: usize,
    cx: &'a mut FunctionContext<'js>,
    api: Handle<'js, JsObject>,
    pub blocks: HashMap<String, Block>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
}

impl<'js, 'rt> Runtime<'js, 'rt> {
    pub fn new(cx: &'rt mut FunctionContext<'js>, api: Handle<'js, JsObject>) -> Self {
        Runtime {
            next_usable_id: 0,
            cx,
            api,
            blocks: HashMap::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
        }
    }

    pub fn generate_id(&mut self) -> String {
        let n = self.next_usable_id;
        self.next_usable_id += 1;
        format!("viiru-{n}")
    }

    // These methods are convenient wrappers around the raw `api::*` function calls

    /// Be sure to clear the screen afterwards, as this creates some spam from JS!
    pub fn load_project(&mut self, path: &str) -> JsResult<'js, JsUndefined> {
        bridge::load_project(self.cx, self.api, path)?;
        todo!()
    }

    pub fn save_project(&mut self, path: &str) -> JsResult<'js, JsUndefined> {
        bridge::save_project(self.cx, self.api, path)
    }

    pub fn create_single_block(&mut self, opcode: &str) -> JsResult<'js, JsString> {
        let is_shadow = BLOCKS[opcode].is_shadow;
        let id = self.generate_id();
        bridge::create_block(self.cx, self.api, opcode, is_shadow, Some(&id));
        todo!()
    }

    // special
    pub fn create_block_template(opcode: &str) -> JsResult<'js, JsString> {
        // let id_handle = create_block(cx, api, opcode, false, id)?;
        // let id = string_of(cx, id_handle);
        let spec = &BLOCKS[opcode];
        for frag in &spec.head {
            if let crate::spec::Fragment::StrumberInput(value, Some(default)) = frag {
                let child_id = match default {
                    crate::spec::DefaultValue::Block(child_opcode) => {
                        // let id_handle = create_block(cx, api, child_opcode, true, None)?;
                        // string_of(cx, id_handle)
                        23
                    }
                    crate::spec::DefaultValue::Str(s) => {
                        // let id_handle = create_block(cx, api, "text", true, None)?;
                        // let id = string_of(cx, id_handle);
                        // change_field(cx, api, &id, "TEXT", s)?;
                        // id
                        todo!()
                    }
                    crate::spec::DefaultValue::Num(n) => {
                        // let id_handle = create_block(cx, api, "math_number", true, None)?;
                        // let id = string_of(cx, id_handle);
                        // change_field(cx, api, &id, "NUM", &n.to_string())?;
                        // id
                        todo!()
                    }
                    crate::spec::DefaultValue::Color((r, g, b)) => {
                        // let id_handle = create_block(cx, api, "colour_picker", true, None)?;
                        // let id = string_of(cx, id_handle);
                        // let rgb_string = format!("#{r:X}{g:X}{b:X}");
                        // change_field(cx, api, &id, "COLOUR", &rgb_string)?;
                        // id
                        todo!()
                    }
                };
                // attach_block(cx, api, &child_id, &id, Some(value))?;
            }
        }
        // Ok(id_handle)
        todo!()
    }

    pub fn delete_block(&mut self, id: &str) -> JsResult<'js, JsUndefined> {
        bridge::delete_block(self.cx, self.api, id);
        todo!()
    }

    pub fn slide_block(&mut self, id: &str, x: f64, y: f64) -> JsResult<'js, JsUndefined> {
        bridge::slide_block(self.cx, self.api, id, x, y);
        todo!()
    }

    pub fn attach_block(
        &mut self,
        id: &str,
        parent_id: &str,
        input_name: &str,
    ) -> JsResult<'js, JsUndefined> {
        bridge::attach_block(self.cx, self.api, id, parent_id, Some(input_name));
        todo!()
    }

    pub fn detach_block(&mut self, id: &str) -> JsResult<'js, JsUndefined> {
        bridge::detach_block(self.cx, self.api, id);
        todo!()
    }

    pub fn change_field(
        &mut self,
        id: &str,
        field: &str,
        value: &str,
    ) -> JsResult<'js, JsUndefined> {
        bridge::change_field(self.cx, self.api, id, field, value);
        todo!()
    }

    // todo: ChangeMutation(String, ()),

    // internal use, only needed for synchronization
    fn get_all_blocks(&mut self) -> NeonResult<Handle<'js, JsObject>> {
        bridge::get_all_blocks(self.cx, self.api)
    }

    fn get_variables_of_type(&mut self, kind: &str) -> JsResult<'js, JsObject> {
        bridge::get_variables_of_type(self.cx, self.api, kind)
    }

    /// Finalize results
    pub fn undefined_or_throw(&mut self, result: ViiruResult) -> JsResult<'js, JsUndefined> {
        undefined_or_throw(self.cx, result)
    }
}
