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
    pub inputs: HashMap<String, Input>,
    pub fields: HashMap<String, Field>,
}

pub struct Input {
    pub shadow_id: Option<String>,
    pub block_id: Option<String>,
}

pub struct Field {
    pub text: String,
    pub id: Option<String>,
}
