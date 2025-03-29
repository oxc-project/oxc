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
    #[expect(clippy::struct_field_names)]
    pub group_id: Option<GroupId>,
}

#[derive(Debug)]
pub struct Fill<'a> {
    pub parts: Vec<'a, Doc<'a>>,
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
