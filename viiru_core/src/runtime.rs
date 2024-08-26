use std::collections::HashMap;

use neon::prelude::*;

use crate::{
    block::{Block, Field, Input},
    bridge::{self, map_each_value, string_of, to_block, VariableType},
    opcodes::{BLOCKS, NUMBERS_ISH, TOOLBOX},
    result::{undefined_or_throw, ViiruResult},
    spec::Fragment,
    ui::{Accumulators, DropPoint},
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

impl Viewport {
    pub fn width(&self) -> i32 {
        self.x_max - self.x_min
    }
    pub fn height(&self) -> i32 {
        self.y_max - self.y_min
    }
}

pub struct Runtime<'js, 'a> {
    // internals
    next_usable_id: usize,
    cx: &'a mut FunctionContext<'js>,
    api: Handle<'js, JsObject>,
    do_sync: bool,
    pub state: State,
    is_dirty: bool,
    // ui
    pub viewport: Viewport,
    pub window_cols: u16,
    pub window_rows: u16,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub cursor_x: i32,
    pub cursor_y: i32,
    pub toolbox_cursor: usize,
    pub toolbox_scroll: usize,
    pub toolbox_visible_max: usize,
    pub last_command: char,
    pub command_buffer: String,
    pub status_message: String,
    pub editing_shadow: String,
    // constant data
    pub viewport_offset_x: i32,
    pub viewport_offset_y: i32,
    pub toolbox_width: i32,
    pub status_height: i32,
    pub toolbox: Vec<String>,
    // ephemeral data
    pub block_positions: HashMap<(i32, i32), Vec<String>>,
    pub cursor_block: Option<String>,
    pub drop_points: HashMap<(i32, i32), DropPoint>,
    pub writable_points: HashMap<(i32, i32), String>,
    // synchronized data
    pub blocks: HashMap<String, Block>,
    pub top_level: Vec<String>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
    pub broadcasts: HashMap<String, String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
            // internals
            cx,
            api,
            next_usable_id: 0,
            state: State::Move,
            do_sync: true,
            is_dirty: false,
            // ui
            viewport: Viewport {
                x_min: 0,
                x_max: 0,
                y_min: 0,
                y_max: 0,
            },
            window_cols: 0,
            window_rows: 0,
            scroll_x: 0,
            scroll_y: 0,
            cursor_x: 0,
            cursor_y: 0,
            toolbox_cursor: 0,
            toolbox_scroll: 0,
            toolbox_visible_max: 0,
            last_command: ' ',
            command_buffer: String::new(),
            status_message: String::new(),
            editing_shadow: String::new(),
            // constant data
            viewport_offset_x: 0,
            viewport_offset_y: 0,
            toolbox_width: 0,
            status_height: 0,
            toolbox: vec![],
            // ephemeral data
            block_positions: HashMap::new(),
            cursor_block: None,
            drop_points: HashMap::new(),
            writable_points: HashMap::new(),
            // synchronized data
            blocks: HashMap::new(),
            top_level: vec![],
            variables: HashMap::new(),
            lists: HashMap::new(),
            broadcasts: HashMap::new(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn move_x(&mut self, dx: i32) -> NeonResult<()> {
        self.cursor_x += dx;
        if let Some(id) = self.cursor_block.clone() {
            self.slide_block_by(&id, dx, 0)?;
        }
        Ok(())
    }

    pub fn move_y(&mut self, dy: i32) -> NeonResult<()> {
        self.cursor_y += dy;
        if let Some(id) = self.cursor_block.clone() {
            self.slide_block_by(&id, 0, dy)?;
        }
        Ok(())
    }

    pub fn put_to_cursor(&mut self, id: &str) -> NeonResult<()> {
        self.detach_block(id)?;
        self.cursor_block = Some(id.to_string());
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

    fn generate_fake_id(&mut self) -> String {
        let mut n = usize::MAX;
        let mut id = format!("viiru-{n}");
        while self.blocks.contains_key(&id) {
            n -= 1;
            id = format!("viiru-{n}");
        }
        id
    }

    pub fn remove_top_level(&mut self, id: &str) {
        let i = self.top_level.iter().position(|p| p == id).unwrap();
        self.top_level.remove(i);
    }

    pub fn process_accumulators(&mut self, accumulators: Accumulators) {
        self.block_positions = accumulators.block_positions;
        self.drop_points = accumulators.drop_points;
        self.writable_points = accumulators.writable_points;
        for (id, (dx, dy)) in accumulators.block_offsets {
            self.blocks.get_mut(&id).unwrap().offset_x = dx;
            self.blocks.get_mut(&id).unwrap().offset_y = dy;
        }
    }

    pub fn compute_own_xy(&self, block_id: &str) -> (i32, i32) {
        let block = &self.blocks[block_id];
        if let Some(parent_id) = &block.parent_id {
            let (parent_x, parent_y) = self.compute_own_xy(parent_id);
            (parent_x + block.offset_x, parent_y + block.offset_y)
        } else {
            (block.x, block.y)
        }
    }

    /// be sure to clear the screen afterwards, as this creates some spam from the JS side
    pub fn load_project(&mut self, path: &str) -> NeonResult<bool> {
        // todo: to ensure a proper reset, move self and return a new Runtime
        let success = bridge::load_project(self.cx, self.api, path)?.value(self.cx);
        if !success {
            return Ok(false);
        }
        // synchronized + constant
        self.blocks = self.get_all_blocks()?;
        self.top_level = self
            .blocks
            .iter()
            .filter(|&(_, block)| block.parent_id.is_none())
            .map(|(id, _)| id.clone())
            .collect();
        self.initialize_toolbox_blocks()?;
        self.variables = self.get_variables_of_type(VariableType::Scalar)?;
        self.lists = self.get_variables_of_type(VariableType::List)?;
        self.broadcasts = self.get_variables_of_type(VariableType::Broadcast)?;
        // ephemeral
        self.block_positions.clear();
        self.cursor_block = None;
        self.drop_points.clear();
        self.writable_points.clear();
        // ui
        self.scroll_x = 0;
        self.scroll_y = 0;
        self.cursor_x = 0;
        self.cursor_y = 0;
        Ok(true)
    }

    fn initialize_toolbox_blocks(&mut self) -> NeonResult<()> {
        self.do_sync = false;
        for opcode in TOOLBOX.iter() {
            let (id, _) = self.create_block_template(opcode)?;
            self.remove_top_level(&id);
            self.toolbox.push(id);
        }
        self.do_sync = true;
        Ok(())
    }

    pub fn save_project(&mut self, path: &str) -> NeonResult<bool> {
        let success = bridge::save_project(self.cx, self.api, path)?.value(self.cx);
        self.is_dirty = false;
        Ok(success)
    }

    pub fn create_single_block(&mut self, opcode: &str) -> NeonResult<String> {
        let spec = &BLOCKS[opcode];
        let is_shadow = spec.is_shadow;
        let id = if self.do_sync {
            self.generate_id()
        } else {
            self.generate_fake_id()
        };
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
                | Fragment::WritableFieldText(field)
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
                offset_x: 0,
                offset_y: 0,
                opcode: opcode.to_string(),
                parent_id: None,
                next_id: None,
                inputs,
                fields,
            },
        );
        self.top_level.push(id.clone());
        // todo: perhaps we can let the vm generate the ID
        if self.do_sync {
            self.is_dirty = true;
            bridge::create_block(self.cx, self.api, opcode, is_shadow, Some(&id))?;
        }
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

    pub fn stamp_block(&mut self, block_id: &str, is_root: bool) -> ViiruResult<String> {
        let original = self.blocks[block_id].clone();
        let (new_x, new_y) = self.compute_own_xy(block_id);
        let stamp_id = self.create_single_block(&original.opcode)?;

        // block sliding is recursive and so only needs to be performed on the root block
        if is_root {
            self.slide_block_to(&stamp_id, new_x, new_y)?;
        }
        for (field_name, field) in original.fields {
            self.set_field(&stamp_id, &field_name, &field.text, field.id.as_deref())?;
        }
        if let Some(next_id) = original.next_id {
            let stamp_next_id = self.stamp_block(&next_id, false)?;
            self.attach_next(&stamp_next_id, &stamp_id)?;
        }
        for (input_name, input) in original.inputs {
            if let Some(shadow_id) = input.shadow_id {
                let stamp_next_id = self.stamp_block(&shadow_id, false)?;
                self.attach_input(&stamp_next_id, &stamp_id, &input_name, true)?;
            }
            if let Some(block_id) = input.block_id {
                let stamp_next_id = self.stamp_block(&block_id, false)?;
                self.attach_input(&stamp_next_id, &stamp_id, &input_name, false)?;
            }
        }

        Ok(stamp_id)
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
        self.detach_block(id)?;
        self.remove_top_level(id);
        self.delete_blocks_recursively(id);
        // The VM handles recursion.
        if self.do_sync {
            self.is_dirty = true;
            bridge::delete_block(self.cx, self.api, id)?;
        }
        Ok(())
    }

    pub fn slide_block_by(&mut self, id: &str, dx: i32, dy: i32) -> NeonResult<()> {
        let block = self.blocks.get_mut(id).unwrap();
        block.x += dx;
        block.y += dy;
        if self.do_sync {
            self.is_dirty = true;
            bridge::slide_block(self.cx, self.api, id, block.x, block.y)?;
        }
        for child in block.inputs.clone().values() {
            if let Some(id) = &child.block_id {
                self.slide_block_by(id, dx, dy)?;
            }
            if let Some(id) = &child.shadow_id {
                self.slide_block_by(id, dx, dy)?;
            }
        }

        Ok(())
    }

    pub fn slide_block_to(&mut self, id: &str, x: i32, y: i32) -> NeonResult<()> {
        let block = self.blocks.get_mut(id).unwrap();
        let dx = x - block.x;
        let dy = y - block.y;
        self.slide_block_by(id, dx, dy)?;
        Ok(())
    }

    pub fn current_drop_point(&self) -> Option<(String, Option<String>)> {
        if let Some(cursor_id) = &self.cursor_block {
            let shape = BLOCKS[&self.blocks[cursor_id].opcode].shape;
            // cursor blocks are top-level, so these are up to date
            let x = self.blocks[cursor_id].x;
            let y = self.blocks[cursor_id].y;
            if let Some(drop_point) = self.drop_points.get(&(x, y)) {
                if drop_point.shape == shape && !BLOCKS[&self.blocks[cursor_id].opcode].is_hat {
                    return Some((drop_point.id.clone(), drop_point.input.clone()));
                }
            }
        }
        None
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
        self.remove_top_level(id);
        if self.do_sync {
            self.is_dirty = true;
            bridge::attach_block(
                self.cx,
                self.api,
                id,
                parent_id,
                Some(input_name),
                is_shadow,
            )?;
        }
        Ok(())
    }

    pub fn attach_next(&mut self, id: &str, parent_id: &str) -> NeonResult<()> {
        let parent = self.blocks.get_mut(parent_id).unwrap();

        parent.next_id.replace(id.to_string());
        self.blocks.get_mut(id).unwrap().parent_id = Some(parent_id.to_string());
        self.remove_top_level(id);

        if self.do_sync {
            self.is_dirty = true;
            bridge::attach_block(self.cx, self.api, id, parent_id, None, false)?;
        }

        Ok(())
    }

    pub fn detach_block(&mut self, id: &str) -> NeonResult<()> {
        let (new_x, new_y) = self.compute_own_xy(id);
        let parent_id = self.blocks[id].parent_id.clone();
        if let Some(parent_id) = parent_id {
            let parent = self.blocks.get_mut(&parent_id).unwrap();
            if parent.next_id.is_some() {
                parent.next_id = None;
            } else {
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
            self.top_level.push(id.to_string());
        }
        self.blocks.get_mut(id).unwrap().parent_id = None;
        self.slide_block_to(id, new_x, new_y)?;
        if self.do_sync {
            self.is_dirty = true;
            bridge::detach_block(self.cx, self.api, id)?;
        }
        Ok(())
    }

    // todo: don't panic everywhere
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
        if self.do_sync {
            self.is_dirty = true;
            bridge::change_field(self.cx, self.api, block_id, field_name, text, data_id)?;
        }
        Ok(())
    }

    // todo: don't panic everywhere
    pub fn get_strumber_field(&self, id: &str) -> String {
        let block = &self.blocks[id];
        if block.opcode == "text" {
            block.fields["TEXT"].text.clone()
        } else if NUMBERS_ISH.contains(&block.opcode.as_str()) {
            block.fields["NUM"].text.clone()
        } else {
            // once again, use the type system please
            "".into()
        }
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
