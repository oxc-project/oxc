use crate::doc::Doc;

pub struct Command<'a> {
    pub indent: Indent,
    pub mode: Mode,
    pub doc: Doc<'a>,
}

impl<'a> Command<'a> {
    pub fn new(indent: Indent, mode: Mode, doc: Doc<'a>) -> Self {
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
