use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    // The offset is calculated automatically while rendering
    pub offset_x: i32,
    pub offset_y: i32,
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

    /// this needs to be a function, since control_stop's bootness is dynamic
    pub fn is_boot(&self) -> bool {
        self.opcode == "control_delete_this_clone"
            || (self.opcode == "control_stop"
                && self.fields["STOP_OPTION"].value != "other scripts in sprite")
    }

    /// returns the old value, or None on failure
    pub fn set_field_text(&mut self, field_name: &str, text: &str) -> Option<String> {
        let field = self.fields.get_mut(field_name)?;
        let mut text = text.to_string();
        std::mem::swap(&mut field.value, &mut text);
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
    pub value: String,
    pub id: Option<String>,
}
