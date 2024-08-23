use std::collections::HashMap;

use crate::blocks::BLOCKS;

pub enum Kind {
    Expression(Expression),
    Stack(Stack),
}

pub enum Throption<T> {
    Given(T),
    Missing,
    Void,
}

pub struct Block {
    pub id: String,
    pub colour: Colour,
    pub opcode: String,
    pub parent_id: Throption<String>,
    pub input_ids: Vec<Option<String>>,
    pub fields: HashMap<String, String>,
    pub kind: Kind,
}

impl Block {
    pub fn new_from_template(opcode: &str) -> Block {
        let spec = &BLOCKS[opcode];

        todo!()
    }
}

pub struct Expression {
    pub shadow: bool,
}

pub struct Stack {
    pub is_hat: bool,
    pub next_id: Throption<String>,
}

pub struct Colour(pub u8, pub u8, pub u8);
