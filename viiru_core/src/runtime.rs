use std::collections::{HashMap, HashSet};

use neon::prelude::*;

use crate::{
    block::{Block, Field, Input},
    bridge::{self, map_each_value, string_of, to_block, VariableType},
    opcodes::{BLOCKS, NUMBERS_ISH},
    result::{undefined_or_throw, ViiruResult},
    spec::Fragment,
};

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x_min: i32,
    /// exclusive
    pub x_max: i32,
    pub y_min: i32,
    /// exclusive
    pub y_max: i32,
}

pub struct Runtime<'js, 'a> {
    // internals
    next_usable_id: usize,
    cx: &'a mut FunctionContext<'js>,
    api: Handle<'js, JsObject>,
    // ui
    pub viewport: Viewport,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub placement_grid: HashMap<(i32, i32), Vec<String>>,
    pub cursor_block: Option<String>,
    pub state: State,
    // data
    pub blocks: HashMap<String, Block>,
    pub top_level: HashSet<String>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
    pub broadcasts: HashMap<String, String>,
}

#[derive(Clone, Copy)]
pub enum State {
    Move,
    Hold,
    Toolbox,
    Command,
    Inline,
}

impl<'js, 'rt> Runtime<'js, 'rt> {
    pub fn new(cx: &'rt mut FunctionContext<'js>, api: Handle<'js, JsObject>) -> Self {
        Runtime {
            cx,
            api,
            next_usable_id: 0,
            viewport: Viewport {
                x_min: 0,
                x_max: 0,
                y_min: 0,
                y_max: 0,
            },
            scroll_x: 0,
            scroll_y: 0,
            cursor_x: 0,
            cursor_y: 0,
            placement_grid: HashMap::new(),
            cursor_block: None,
            state: State::Move,
            top_level: HashSet::new(),
            blocks: HashMap::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            broadcasts: HashMap::new(),
        }
    }

    pub fn is_in_view(&self, x: i32, y: i32) -> bool {
        x - self.scroll_x >= self.viewport.x_min
            && x - self.scroll_x < self.viewport.x_max
            && y - self.scroll_y >= self.viewport.y_min
            && y - self.scroll_y < self.viewport.y_max
    }

    pub fn move_x(&mut self, dx: i32) -> NeonResult<()> {
        self.cursor_x += dx;
        if let Some(id) = self.cursor_block.clone() {
            let x = self.blocks[&id].x + dx;
            let y = self.blocks[&id].y;
            self.slide_block(&id, x, y)?;
        }
        Ok(())
    }

    pub fn move_y(&mut self, dy: i32) -> NeonResult<()> {
        self.cursor_y += dy;
        if let Some(id) = self.cursor_block.clone() {
            let x = self.blocks[&id].x;
            let y = self.blocks[&id].y + dy;
            self.slide_block(&id, x, y)?;
        }
        Ok(())
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

    pub fn create_single_block(&mut self, opcode: &str) -> NeonResult<String> {
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
        self.blocks.insert(
            id.clone(),
            Block {
                x: 0,
                y: 0,
                id: id.clone(),
                opcode: opcode.to_string(),
                parent_id: None,
                next_id: None,
                inputs,
                fields,
            },
        );
        self.top_level.insert(id.clone());
        // todo: perhaps we can let the vm generate the ID
        bridge::create_block(self.cx, self.api, opcode, is_shadow, Some(&id))?;
        Ok(id)
    }

    // special!
    pub fn create_block_template(&mut self, opcode: &str) -> NeonResult<(String, Vec<String>)> {
        let id = self.create_single_block(opcode)?;
        let spec = &BLOCKS[opcode];
        let mut child_ids = vec![];
        for line in &spec.lines {
            for frag in line {
                if let crate::spec::Fragment::StrumberInput(input_name, Some(default)) = frag {
                    let child_id = match default {
                        crate::spec::DefaultValue::Block(child_opcode) => {
                            self.create_single_block(child_opcode)?
                        }
                        crate::spec::DefaultValue::Str(s) => {
                            let text_id = self.create_single_block("text")?;
                            self.set_field(&text_id, "TEXT", s, None)?;
                            text_id
                        }
                        crate::spec::DefaultValue::Num(n, visible) => {
                            let num_id = self.create_single_block("math_number")?;
                            let value = if *visible { &n.to_string() } else { "" };
                            self.set_field(&num_id, "NUM", value, None)?;
                            num_id
                        }
                        crate::spec::DefaultValue::Color((r, g, b)) => {
                            let color_id = self.create_single_block("colour_picker")?;
                            let rgb_string = format!("#{r:X}{g:X}{b:X}");
                            self.set_field(&color_id, "COLOUR", &rgb_string, None)?;
                            color_id
                        }
                    };
                    self.attach_input(&child_id, &id, input_name, true)?;
                    child_ids.push(child_id);
                }
            }
        }
        Ok((id, child_ids))
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
        self.top_level.remove(id);
        self.delete_blocks_recursively(id);
        bridge::delete_block(self.cx, self.api, id)?;
        Ok(())
    }

    pub fn slide_block(&mut self, id: &str, x: i32, y: i32) -> NeonResult<()> {
        let block = self.blocks.get_mut(id).unwrap();
        block.x = x;
        block.y = y;

        bridge::slide_block(self.cx, self.api, id, x, y)?;
        Ok(())
    }

    pub fn attach_input(
        &mut self,
        id: &str,
        parent_id: &str,
        input_name: &str,
        is_shadow: bool,
    ) -> NeonResult<()> {
        let parent = self.blocks.get_mut(parent_id).unwrap();
        parent.set_input(input_name, id, is_shadow);
        self.blocks.get_mut(id).unwrap().parent_id = Some(parent_id.to_string());
        self.top_level.remove(id);

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

    pub fn attach_next(&mut self, id: &str, parent_id: &str) -> NeonResult<()> {
        let parent = self.blocks.get_mut(parent_id).unwrap();

        let mut old_next_id = Some(id.to_string());
        std::mem::swap(&mut old_next_id, &mut parent.next_id);

        self.blocks.get_mut(id).unwrap().parent_id = Some(parent_id.to_string());
        self.top_level.remove(id);

        // TODO handle attaching to the middle of a stack
        if let Some(next_id) = old_next_id {
            todo!()
        }

        bridge::attach_block(self.cx, self.api, id, parent_id, None, false)?;

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
        self.top_level.insert(id.to_string());
        bridge::detach_block(self.cx, self.api, id)?;
        Ok(())
    }

    pub fn set_field(
        &mut self,
        block_id: &str,
        field_name: &str,
        text: &str,
        data_id: Option<&str>,
    ) -> NeonResult<()> {
        let block = self.blocks.get_mut(block_id).unwrap();
        block.set_field_text(field_name, text);
        if let Some(id) = data_id {
            block.set_field_id(field_name, id);
        } else {
            block.remove_field_id(field_name);
        }

        bridge::change_field(self.cx, self.api, block_id, field_name, text, data_id)?;
        Ok(())
    }

    pub fn set_strumber_field(&mut self, id: &str, text: &str) -> NeonResult<()> {
        let block = self.blocks.get_mut(id).unwrap();
        if block.opcode == "text" {
            self.set_field(id, "TEXT", text, None)?;
        } else if NUMBERS_ISH.contains(&block.opcode.as_str()) {
            self.set_field(id, "NUM", text, None)?;
        }
        Ok(())
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
