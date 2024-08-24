use std::collections::HashMap;

use neon::prelude::*;

use crate::{
    block::Block,
    result::{undefined_or_throw, ViiruResult},
};

pub struct Runtime<'js, 'a> {
    current_id_number: usize,
    cx: &'a mut FunctionContext<'js>,
    api: Handle<'js, JsObject>,
    pub blocks: HashMap<String, Block>,
    pub variables: HashMap<String, String>,
    pub lists: HashMap<String, String>,
}

impl<'js, 'a> Runtime<'js, 'a> {
    pub fn init(cx: &'a mut FunctionContext<'js>, api: Handle<'js, JsObject>) -> Self {
        Runtime {
            current_id_number: 0,
            cx,
            api,
            blocks: HashMap::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
        }
    }

    pub fn undefined_or_throw(&mut self, result: ViiruResult) -> JsResult<'js, JsUndefined> {
        undefined_or_throw(self.cx, result)
    }
}
