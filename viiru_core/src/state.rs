use std::collections::HashMap;

use crate::block::Block;

pub struct State {
    pub blocks: HashMap<String, Block>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
}
