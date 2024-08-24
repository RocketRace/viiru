use std::{collections::HashMap, default};

use crate::opcodes::BLOCKS;

#[derive(Default)]
pub enum Throption<T> {
    #[default]
    Void,
    Missing,
    Given(T),
}

#[derive(Default)]
pub struct Block {
    pub id: String,
    pub opcode: String,
    pub parent_id: Option<String>,
    pub next_id: Option<String>,
    pub input_ids: HashMap<String, (Option<String>, Option<String>)>,
    pub fields: HashMap<String, (String, Option<String>)>,
}

impl Block {
    pub fn new_from_template(opcode: &str) -> Block {
        let spec = &BLOCKS[opcode];

        todo!()
    }
}
