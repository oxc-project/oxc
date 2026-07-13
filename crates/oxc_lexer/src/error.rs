pub mod diag_code {
    pub const OK: u16 = 0;
    pub const UNTERMINATED_STRING: u16 = 1;
    pub const UNTERMINATED_TEMPLATE: u16 = 2;
    pub const UNTERMINATED_BLOCK_COMMENT: u16 = 3;
    pub const UNTERMINATED_REGEXP: u16 = 4;
    pub const LINE_TERMINATOR_IN_REGEXP: u16 = 5;
    pub const INVALID_UTF8: u16 = 6;
    pub const INVALID_UNICODE_ESCAPE: u16 = 7;
    pub const INVALID_IDENTIFIER_ESCAPE: u16 = 8;
    pub const INVALID_NUMERIC_SEPARATOR: u16 = 9;
    pub const INVALID_BIGINT: u16 = 10;
    pub const INVALID_NUMERIC_LITERAL: u16 = 11;
    pub const INVALID_HASHBANG_POSITION: u16 = 12;
    pub const INVALID_REGEXP_FLAG: u16 = 13;
    pub const DUPLICATE_REGEXP_FLAG: u16 = 14;
    pub const INVALID_REGEXP_GRAMMAR: u16 = 15;
    pub const ORACLE_DEPTH_EXCEEDED: u16 = 16;
    pub const ALLOCATION_LIMIT_EXCEEDED: u16 = 17;
    pub const UNEXPECTED_CHARACTER: u16 = 18;
    pub const LINE_TERMINATOR_IN_STRING: u16 = 19;
    pub const HTML_COMMENT_IN_MODULE: u16 = 20;
}

pub mod diag_severity {
    pub const ERROR: u16 = 0;
    pub const WARNING: u16 = 1;
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Diagnostic {
    pub off: u32,
    pub len: u32,
    pub code: u16,
    pub severity: u16,
}
