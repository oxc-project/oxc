use crate::ir::Doc;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Break,
    Flat,
}

impl Mode {
    pub fn is_break(self) -> bool {
        self == Self::Break
    }

    pub fn is_flat(self) -> bool {
        self == Self::Flat
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Indent {
    pub root: bool,
    pub length: usize,
}

impl Indent {
    pub fn root() -> Self {
        Self { root: true, length: 0 }
    }

    pub fn new(length: usize) -> Self {
        Self { root: false, length }
    }
}
