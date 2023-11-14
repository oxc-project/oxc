use crate::{doc::Doc, PrettierOptions};

pub struct Command<'a, 'b> {
    pub indent: Indent,
    pub mode: Mode,
    pub doc: &'b Doc<'a>,
}

impl<'a, 'b> Command<'a, 'b> {
    pub fn new(indent: Indent, mode: Mode, doc: &'a Doc<'b>) -> Self {
        Self { indent, mode, doc }
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    Break,
    Flat,
}

#[derive(Clone, Copy)]
pub struct Indent {
    pub value: &'static str,
    pub length: usize,
}

impl Indent {
    pub fn root() -> Self {
        Self { value: "", length: 0 }
    }
}
