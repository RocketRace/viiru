use std::{collections::HashMap, default};

use crate::blocks::BLOCKS;

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
    pub parent_id: Throption<String>,
    pub next_id: Throption<String>,
    pub input_ids: Vec<(Option<String>, Option<String>)>,
    pub fields: HashMap<String, String>,
}

impl Block {
    pub fn new_from_template(opcode: &str) -> Block {
        let spec = &BLOCKS[opcode];

        todo!()
    }
}
