use oxc_allocator::{Box, Vec};

use crate::GroupId;

/// IR for the pretty printing.
/// Direct use is discouraged, use the macro instead.
#[derive(Debug)]
pub enum Doc<'a> {
    Str(&'a str),
    Array(Vec<'a, Doc<'a>>),
    Group(Group<'a>),
    Fill(Fill<'a>),
    IfBreak(IfBreak<'a>),
    BreakParent,
    Line(Line),
    Indent(Vec<'a, Doc<'a>>),
    IndentIfBreak(IndentIfBreak<'a>),
    LineSuffix(Vec<'a, Doc<'a>>),
    LineSuffixBoundary,
}

#[derive(Debug)]
pub struct Group<'a> {
    pub contents: Vec<'a, Doc<'a>>,
    pub should_break: bool,
    pub expanded_states: Option<Vec<'a, Doc<'a>>>,
    #[allow(clippy::struct_field_names)]
    pub group_id: Option<GroupId>,
}

#[derive(Debug)]
pub struct Fill<'a> {
    pub contents: Vec<'a, Doc<'a>>,
}

#[derive(Debug)]
pub struct IfBreak<'a> {
    pub break_contents: Box<'a, Doc<'a>>,
    pub flat_contents: Box<'a, Doc<'a>>,
    pub group_id: Option<GroupId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub hard: bool,
    pub soft: bool,
    pub literal: bool,
}

#[derive(Debug)]
pub struct IndentIfBreak<'a> {
    pub contents: Box<'a, Doc<'a>>,
    pub group_id: GroupId,
}

// Printer utils
impl<'a> Fill<'a> {
    pub fn drain_out_pair(&mut self) -> (Option<Doc<'a>>, Option<Doc<'a>>) {
        let content = if self.contents.len() > 0 { Some(self.contents.remove(0)) } else { None };
        let whitespace = if self.contents.len() > 0 { Some(self.contents.remove(0)) } else { None };
        (content, whitespace)
    }

    pub fn dequeue(&mut self) -> Option<Doc<'a>> {
        if self.contents.len() > 0 {
            Some(self.contents.remove(0))
        } else {
            None
        }
    }

    pub fn enqueue(&mut self, doc: Doc<'a>) {
        self.contents.insert(0, doc);
    }

    pub fn parts(&self) -> &[Doc<'a>] {
        &self.contents
    }

    pub fn take_parts(self) -> Vec<'a, Doc<'a>> {
        self.contents
    }
}

// NOTE: Really needed? Just use `Doc` as a separator?
#[derive(Clone, Copy)]
pub enum JoinSeparator {
    Softline,
    Hardline,
    CommaLine,  // [",", line]
    CommaSpace, // ", "
    Literalline,
}
