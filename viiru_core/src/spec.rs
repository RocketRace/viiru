
pub struct Spec {
    pub head: Vec<Fragment>,
    pub mouths: Vec<(&'static str, Vec<Fragment>)>
}

pub enum Fragment {
    Text(&'static str),
    Input(InputKind, &'static str),
    Special(Special),
}

pub enum Special {
    Flag,
}

pub enum InputKind {
    Strumber,
    Boolean,
    Stack,
}

/// panics on invalid input so be careful
pub fn spec(s: &str) -> Spec {
    todo!()
}
