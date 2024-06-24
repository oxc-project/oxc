mod print;

use bitflags::bitflags;
use oxc_ast::CommentKind;

#[derive(Debug, Clone, Copy)]
pub struct Comment {
    pub start: u32,
    pub end: u32,
    pub is_block: bool,
    pub has_line_suffix: bool,
}

impl Comment {
    pub fn new(start: u32, end: u32, kind: CommentKind) -> Self {
        // The comment span is for the comment value
        // -2 for `//` and `/*`
        let start = start - 2;
        // +2 for `/*`
        let end = if kind.is_multi_line() { end + 2 } else { end };
        Self { start, end, is_block: kind.is_multi_line(), has_line_suffix: false }
    }

    pub fn with_line_suffix(mut self, yes: bool) -> Self {
        self.has_line_suffix = yes;
        self
    }

    pub fn matches_flags(self, flags: CommentFlags) -> bool {
        if flags.contains(CommentFlags::Block) && !self.is_block {
            return false;
        }
        if flags.contains(CommentFlags::Line) && self.is_block {
            return false;
        }
        true
    }
}

#[derive(Default)]
pub struct DanglingCommentsPrintOptions {
    ident: bool,
}

impl DanglingCommentsPrintOptions {
    pub(crate) fn with_ident(mut self, ident: bool) -> Self {
        self.ident = ident;
        self
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct CommentFlags: u8 {
        const Leading        = 1 << 0; // Check comment is a leading comment
        const Trailing       = 1 << 1; // Check comment is a trailing comment
        const Dangling       = 1 << 2; // Check comment is a dangling comment
        const Block          = 1 << 3; // Check comment is a block comment
        const Line           = 1 << 4; // Check comment is a line comment
        const PrettierIgnore = 1 << 5; // Check comment is a `prettier-ignore` comment
        const First          = 1 << 6; // Check comment is the first attached comment
        const Last           = 1 << 7; // Check comment is the last attached comment
    }
}
