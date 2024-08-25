use std::collections::HashMap;

use neon::prelude::*;

use crate::{
    block::{Block, Field, Input},
    bridge::{self, map_each_value, string_of, to_block, VariableType},
    opcodes::BLOCKS,
    result::{undefined_or_throw, ViiruResult},
    spec::Fragment,
};

pub struct Runtime<'js, 'a> {
    next_usable_id: usize,
    cx: &'a mut FunctionContext<'js>,
    api: Handle<'js, JsObject>,
    pub blocks: HashMap<String, Block>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
    pub broadcasts: HashMap<String, String>,
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
            broadcasts: HashMap::new(),
        }
    }

    pub fn generate_id(&mut self) -> String {
        let mut n = self.next_usable_id;
        let mut id = format!("viiru-{n}");
        self.next_usable_id += 1;
        while self.blocks.contains_key(&id) {
            n = self.next_usable_id;
            id = format!("viiru-{n}");
            self.next_usable_id += 1;
        }
        id
    }

    // These methods are convenient wrappers around the raw `api::*` function calls

    /// be sure to clear the screen afterwards, as this creates some spam from the JS side
    pub fn load_project(&mut self, path: &str) -> NeonResult<()> {
        bridge::load_project(self.cx, self.api, path)?;
        self.blocks = self.get_all_blocks()?;
        self.variables = self.get_variables_of_type(VariableType::Scalar)?;
        self.lists = self.get_variables_of_type(VariableType::List)?;
        self.broadcasts = self.get_variables_of_type(VariableType::Broadcast)?;
        Ok(())
    }

    pub fn save_project(&mut self, path: &str) -> NeonResult<()> {
        bridge::save_project(self.cx, self.api, path)?;
        Ok(())
    }

    pub fn create_single_block(&mut self, opcode: &str) -> JsResult<'js, JsString> {
        let spec = &BLOCKS[opcode];
        let is_shadow = spec.is_shadow;
        let id = self.generate_id();
        let inputs: HashMap<_, _> = spec
            .lines
            .iter()
            .flatten()
            .filter_map(|frag| {
                if let Fragment::StrumberInput(input, _)
                | Fragment::BooleanInput(input)
                | Fragment::BlockInput(input) = frag
                {
                    Some((
                        input.clone(),
                        Input {
                            shadow_id: None,
                            block_id: None,
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();
        let fields: HashMap<_, _> = spec
            .lines
            .iter()
            .flatten()
            .filter_map(|frag| {
                if let Fragment::FieldText(field)
                | Fragment::CustomColour(field)
                | Fragment::Dropdown(field) = frag
                {
                    Some((
                        field.clone(),
                        Field {
                            text: "".into(),
                            id: None,
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();
        bridge::create_block(self.cx, self.api, opcode, is_shadow, None)?;
        self.blocks.insert(
            id.clone(),
            Block {
                id,
                opcode: opcode.to_string(),
                parent_id: None,
                next_id: None,
                inputs,
                fields,
            },
        );
        todo!()
    }

    // special
    pub fn create_block_template(opcode: &str) -> JsResult<'js, JsString> {
        // let id_handle = create_block(cx, api, opcode, false, id)?;
        // let id = string_of(cx, id_handle);
        let spec = &BLOCKS[opcode];
        for line in &spec.lines {
            for frag in line {
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
        }
        // Ok(id_handle)
        todo!()
    }

    fn delete_blocks_recursively(&mut self, id: &str) {
        let block = self.blocks.remove(id).unwrap();
        for input in block.inputs.values() {
            if let Some(id) = &input.block_id {
                self.delete_blocks_recursively(id);
            }
            if let Some(id) = &input.shadow_id {
                self.delete_blocks_recursively(id);
            }
        }
    }

    pub fn delete_block(&mut self, id: &str) -> NeonResult<()> {
        self.delete_blocks_recursively(id);
        bridge::delete_block(self.cx, self.api, id)?;
        Ok(())
    }

    pub fn slide_block(&mut self, id: &str, x: f64, y: f64) -> NeonResult<()> {
        bridge::slide_block(self.cx, self.api, id, x, y)?;
        todo!()
    }

    pub fn attach_block(
        &mut self,
        id: &str,
        parent_id: &str,
        input_name: &str,
        is_shadow: bool,
    ) -> NeonResult<()> {
        let parent = self.blocks.get_mut(parent_id).unwrap();
        parent.set_input(input_name, id, is_shadow);
        self.blocks.get_mut(id).unwrap().parent_id = Some(parent_id.to_string());

        bridge::attach_block(
            self.cx,
            self.api,
            id,
            parent_id,
            Some(input_name),
            is_shadow,
        )?;

        Ok(())
    }

    pub fn detach_block(&mut self, id: &str) -> NeonResult<()> {
        let parent_id = self.blocks[id].parent_id.clone();
        if let Some(parent_id) = parent_id {
            let parent = self.blocks.get_mut(&parent_id).unwrap();
            let (input_name, is_shadow) = parent
                .inputs
                .iter()
                .filter_map(|(input_name, input)| {
                    if let Some(bid) = &input.block_id {
                        (bid == id).then(|| (input_name.clone(), false))
                    } else if let Some(bid) = &input.shadow_id {
                        (bid == id).then(|| (input_name.clone(), true))
                    } else {
                        None
                    }
                })
                .next()
                .unwrap();
            parent.remove_input(&input_name, is_shadow);
        }
        self.blocks.get_mut(id).unwrap().parent_id = None;
        bridge::detach_block(self.cx, self.api, id)?;
        Ok(())
    }

    pub fn change_field(
        &mut self,
        block_id: &str,
        field_name: &str,
        text: &str,
        data_id: Option<&str>,
    ) -> JsResult<'js, JsUndefined> {
        let block = self.blocks.get_mut(block_id).unwrap();
        block.set_field_text(field_name, text);
        if let Some(id) = data_id {
            block.set_field_id(field_name, id);
        } else {
            block.remove_field_id(field_name);
        }

        bridge::change_field(self.cx, self.api, block_id, field_name, text, data_id)?;
        todo!()
    }

    // todo: ChangeMutation(String, ()),

    // internal use, only needed for synchronization
    fn get_all_blocks(&mut self) -> NeonResult<HashMap<String, Block>> {
        let blocks: Handle<JsObject> = bridge::get_all_blocks(self.cx, self.api)?;
        map_each_value(self.cx, blocks, |cx, obj| to_block(cx, obj))
    }

    fn get_variables_of_type(
        &mut self,
        variable_type: VariableType,
    ) -> NeonResult<HashMap<String, String>> {
        let vars: Handle<JsObject> =
            bridge::get_variables_of_type(self.cx, self.api, variable_type)?;
        map_each_value(self.cx, vars, |cx, obj| Ok(string_of(cx, obj)))
    }

    /// Finalize results
    pub fn undefined_or_throw(&mut self, result: ViiruResult) -> JsResult<'js, JsUndefined> {
        undefined_or_throw(self.cx, result)
    }
}
