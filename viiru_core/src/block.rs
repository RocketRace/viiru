use std::{collections::HashMap, default};

use crate::opcodes::BLOCKS;

#[derive(Default)]
pub enum Throption<T> {
    #[default]
    Void,
    Missing,
    Given(T),
}

#[derive(Default, Clone)]
pub struct Block {
    pub id: String,
    pub opcode: String,
    pub parent_id: Option<String>,
    pub next_id: Option<String>,
    pub inputs: HashMap<String, Input>,
    pub fields: HashMap<String, Field>,
}

impl Block {
    /// returns the old value if it exists, or None on failure
    pub fn set_input(&mut self, input_name: &str, id: &str, is_shadow: bool) -> Option<String> {
        let input = self.inputs.get_mut(input_name)?;

        let mut given = Some(id.to_string());
        if is_shadow {
            std::mem::swap(&mut input.shadow_id, &mut given);
        } else {
            std::mem::swap(&mut input.block_id, &mut given);
        }
        given
    }

    /// returns the old value if it exists, or None on failure
    pub fn remove_input(&mut self, input_name: &str, is_shadow: bool) -> Option<String> {
        let input = self.inputs.get_mut(input_name)?;

        if is_shadow {
            std::mem::take(&mut input.shadow_id)
        } else {
            std::mem::take(&mut input.block_id)
        }
    }

    /// returns the old value, or None on failure
    pub fn set_field_text(&mut self, field_name: &str, text: &str) -> Option<String> {
        let field = self.fields.get_mut(field_name)?;
        let mut text = text.to_string();
        std::mem::swap(&mut field.text, &mut text);
        Some(text)
    }

    /// returns the old value, or None on failure
    pub fn set_field_id(&mut self, field_name: &str, id: &str) -> Option<String> {
        let field = self.fields.get_mut(field_name)?;
        let mut id = Some(id.to_string());
        std::mem::swap(&mut field.id, &mut id);
        id
    }

    /// returns the old value, or None on failure
    pub fn remove_field_id(&mut self, field_name: &str) -> Option<String> {
        let field = self.fields.get_mut(field_name)?;
        std::mem::take(&mut field.id)
    }
}

#[derive(Clone)]
pub struct Input {
    pub shadow_id: Option<String>,
    pub block_id: Option<String>,
}

#[derive(Clone)]
pub struct Field {
    pub text: String,
    pub id: Option<String>,
}
