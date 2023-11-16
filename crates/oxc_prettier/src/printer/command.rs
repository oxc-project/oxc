use crate::doc::Doc;
use serde::Serialize;

#[derive(Serialize)]
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

#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
pub enum Mode {
    Break,
    Flat,
}

#[derive(Clone, Copy, Serialize)]
pub struct Indent {
    pub length: usize,
}

impl Indent {
    pub fn root() -> Self {
        Self { length: 0 }
    }
}
