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
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Mode {
    Break,
    Flat,
}

impl Mode {
    pub fn is_break(self) -> bool {
        matches!(self, Self::Break)
    }
}

#[derive(Clone, Copy)]
pub struct Indent {
    pub length: usize,
}

impl Indent {
    pub fn root() -> Self {
        Self { length: 0 }
    }
}
