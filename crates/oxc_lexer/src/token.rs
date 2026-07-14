/// Token kinds are plain `u8`s the pipeline computes them arithmetically and blends them in SIMD registers.
pub mod token_kind {
    pub const EOF: u8 = 0;

    pub const IDENT: u8 = 1;
    pub const PRIVATE_IDENT: u8 = 2;
    pub const NUMBER: u8 = 3;
    pub const BIGINT: u8 = 4;
    pub const STRING: u8 = 5;
    pub const REGEXP: u8 = 6;
    pub const TEMPLATE_NO_SUB: u8 = 7;
    pub const TEMPLATE_HEAD: u8 = 8;
    pub const TEMPLATE_MIDDLE: u8 = 9;
    pub const TEMPLATE_TAIL: u8 = 10;

    pub const LINE_COMMENT: u8 = 11;
    pub const BLOCK_COMMENT: u8 = 12;
    pub const HASHBANG: u8 = 13;
    pub const WHITESPACE: u8 = 14;
    pub const LINE_TERMINATOR: u8 = 15;

    pub const STRING_COOKED: u8 = 16;
    pub const IDENT_ESCAPED: u8 = 17;
    pub const PRIVATE_IDENT_ESCAPED: u8 = 18;
    pub const TEMPLATE_NO_SUB_COOKED: u8 = 19;
    pub const TEMPLATE_HEAD_COOKED: u8 = 20;
    pub const TEMPLATE_MIDDLE_COOKED: u8 = 21;
    pub const TEMPLATE_TAIL_COOKED: u8 = 22;
    pub const DECIMAL: u8 = 23;
    pub const FLOAT: u8 = 24;
    pub const BINARY: u8 = 25;
    pub const OCTAL: u8 = 26;
    pub const HEX: u8 = 27;
    pub const JSX_TEXT: u8 = 28;
    pub const JSX_TAG_END: u8 = 29;
    pub const JSX_LT: u8 = 30;
    pub const LBRACE: u8 = 32;
    pub const RBRACE: u8 = 33;
    pub const LPAREN: u8 = 34;
    pub const RPAREN: u8 = 35;
    pub const LBRACKET: u8 = 36;
    pub const RBRACKET: u8 = 37;
    pub const DOT: u8 = 38;
    pub const ELLIPSIS: u8 = 39;
    pub const SEMI: u8 = 40;
    pub const COMMA: u8 = 41;
    pub const COLON: u8 = 42;
    pub const QUESTION: u8 = 43;
    pub const OPTIONAL_CHAIN: u8 = 44;
    pub const NULLISH: u8 = 45;
    pub const NULLISH_EQ: u8 = 46;
    pub const ARROW: u8 = 47;
    pub const LT: u8 = 48;
    pub const LE: u8 = 49;
    pub const GT: u8 = 50;
    pub const GE: u8 = 51;
    pub const EQ: u8 = 52;
    pub const EQ_EQ: u8 = 53;
    pub const EQ_EQ_EQ: u8 = 54;
    pub const BANG: u8 = 55;
    pub const BANG_EQ: u8 = 56;
    pub const BANG_EQ_EQ: u8 = 57;
    pub const PLUS: u8 = 58;
    pub const PLUS_PLUS: u8 = 59;
    pub const PLUS_EQ: u8 = 60;
    pub const MINUS: u8 = 61;
    pub const MINUS_MINUS: u8 = 62;
    pub const MINUS_EQ: u8 = 63;
    pub const STAR: u8 = 64;
    pub const STAR_STAR: u8 = 65;
    pub const STAR_EQ: u8 = 66;
    pub const STAR_STAR_EQ: u8 = 67;
    pub const SLASH: u8 = 68;
    pub const SLASH_EQ: u8 = 69;
    pub const PERCENT: u8 = 70;
    pub const PERCENT_EQ: u8 = 71;
    pub const AMP: u8 = 72;
    pub const AMP_AMP: u8 = 73;
    pub const AMP_EQ: u8 = 74;
    pub const AMP_AMP_EQ: u8 = 75;
    pub const PIPE: u8 = 76;
    pub const PIPE_PIPE: u8 = 77;
    pub const PIPE_EQ: u8 = 78;
    pub const PIPE_PIPE_EQ: u8 = 79;
    pub const CARET: u8 = 80;
    pub const CARET_EQ: u8 = 81;
    pub const TILDE: u8 = 82;
    pub const LSHIFT: u8 = 83;
    pub const LSHIFT_EQ: u8 = 84;
    pub const RSHIFT: u8 = 85;
    pub const RSHIFT_EQ: u8 = 86;
    pub const URSHIFT: u8 = 87;
    pub const URSHIFT_EQ: u8 = 88;
    pub const AT: u8 = 89;
    pub const KW_BASE: u8 = 128;
    pub const KW_BREAK: u8 = 128;
    pub const KW_CASE: u8 = 129;
    pub const KW_CATCH: u8 = 130;
    pub const KW_CLASS: u8 = 131;
    pub const KW_CONST: u8 = 132;
    pub const KW_CONTINUE: u8 = 133;
    pub const KW_DEBUGGER: u8 = 134;
    pub const KW_DEFAULT: u8 = 135;
    pub const KW_DELETE: u8 = 136;
    pub const KW_DO: u8 = 137;
    pub const KW_ELSE: u8 = 138;
    pub const KW_ENUM: u8 = 139;
    pub const KW_EXPORT: u8 = 140;
    pub const KW_EXTENDS: u8 = 141;
    pub const KW_FALSE: u8 = 142;
    pub const KW_FINALLY: u8 = 143;
    pub const KW_FOR: u8 = 144;
    pub const KW_FUNCTION: u8 = 145;
    pub const KW_IF: u8 = 146;
    pub const KW_IMPORT: u8 = 147;
    pub const KW_IN: u8 = 148;
    pub const KW_INSTANCEOF: u8 = 149;
    pub const KW_NEW: u8 = 150;
    pub const KW_NULL: u8 = 151;
    pub const KW_RETURN: u8 = 152;
    pub const KW_SUPER: u8 = 153;
    pub const KW_SWITCH: u8 = 154;
    pub const KW_THIS: u8 = 155;
    pub const KW_THROW: u8 = 156;
    pub const KW_TRUE: u8 = 157;
    pub const KW_TRY: u8 = 158;
    pub const KW_TYPEOF: u8 = 159;
    pub const KW_VAR: u8 = 160;
    pub const KW_VOID: u8 = 161;
    pub const KW_WHILE: u8 = 162;
    pub const KW_WITH: u8 = 163;
    pub const KW_YIELD: u8 = 164;
    pub const KW_LET: u8 = 165;
    pub const KW_STATIC: u8 = 166;
    pub const KW_ASYNC: u8 = 167;
    pub const KW_AWAIT: u8 = 168;
    pub const KW_OF: u8 = 169;
    pub const KW_FROM: u8 = 170;
    pub const KW_AS: u8 = 171;
    // TS-mode contextual keywords (`LexOptions::ts`) plus the strict-mode
    // reserved words; JS mode lexes all of these spellings as IDENT.
    // Contiguous after the JS block so `>= KW_BASE` range checks cover both.
    pub const KW_ABSTRACT: u8 = 172;
    pub const KW_ACCESSOR: u8 = 173;
    pub const KW_ANY: u8 = 174;
    pub const KW_ASSERTS: u8 = 175;
    pub const KW_BIGINT: u8 = 176;
    pub const KW_BOOLEAN: u8 = 177;
    pub const KW_DECLARE: u8 = 178;
    pub const KW_GLOBAL: u8 = 179;
    pub const KW_IMPLEMENTS: u8 = 180;
    pub const KW_INFER: u8 = 181;
    pub const KW_INTERFACE: u8 = 182;
    pub const KW_INTRINSIC: u8 = 183;
    pub const KW_IS: u8 = 184;
    pub const KW_KEYOF: u8 = 185;
    pub const KW_MODULE: u8 = 186;
    pub const KW_NAMESPACE: u8 = 187;
    pub const KW_NEVER: u8 = 188;
    pub const KW_NUMBER: u8 = 189;
    pub const KW_OBJECT: u8 = 190;
    pub const KW_OUT: u8 = 191;
    pub const KW_OVERRIDE: u8 = 192;
    pub const KW_PACKAGE: u8 = 193;
    pub const KW_PRIVATE: u8 = 194;
    pub const KW_PROTECTED: u8 = 195;
    pub const KW_PUBLIC: u8 = 196;
    pub const KW_READONLY: u8 = 197;
    pub const KW_REQUIRE: u8 = 198;
    pub const KW_SATISFIES: u8 = 199;
    pub const KW_STRING: u8 = 200;
    pub const KW_SYMBOL: u8 = 201;
    pub const KW_TYPE: u8 = 202;
    pub const KW_UNDEFINED: u8 = 203;
    pub const KW_UNIQUE: u8 = 204;
    pub const KW_UNKNOWN: u8 = 205;
    pub const KW_USING: u8 = 206;
    pub const INVALID: u8 = 255;
}

pub const TRIVIA_MIN: u8 = token_kind::LINE_COMMENT;
pub const TRIVIA_MAX: u8 = token_kind::LINE_TERMINATOR;

#[inline]
#[must_use]
pub const fn is_trivia(kind: u8) -> bool {
    kind.wrapping_sub(TRIVIA_MIN) <= TRIVIA_MAX - TRIVIA_MIN
}

#[inline]
#[must_use]
pub const fn is_string_kind(kind: u8) -> bool {
    kind == token_kind::STRING || kind == token_kind::STRING_COOKED
}

#[inline]
#[must_use]
pub const fn is_template_no_sub_kind(kind: u8) -> bool {
    kind == token_kind::TEMPLATE_NO_SUB || kind == token_kind::TEMPLATE_NO_SUB_COOKED
}

#[inline]
#[must_use]
pub const fn is_template_head_kind(kind: u8) -> bool {
    kind == token_kind::TEMPLATE_HEAD || kind == token_kind::TEMPLATE_HEAD_COOKED
}

#[inline]
#[must_use]
pub const fn is_template_middle_kind(kind: u8) -> bool {
    kind == token_kind::TEMPLATE_MIDDLE || kind == token_kind::TEMPLATE_MIDDLE_COOKED
}

#[inline]
#[must_use]
pub const fn is_template_tail_kind(kind: u8) -> bool {
    kind == token_kind::TEMPLATE_TAIL || kind == token_kind::TEMPLATE_TAIL_COOKED
}

#[inline]
#[must_use]
pub const fn is_ident_kind(kind: u8) -> bool {
    kind == token_kind::IDENT || kind == token_kind::IDENT_ESCAPED
}

#[inline]
#[must_use]
pub const fn is_private_ident_kind(kind: u8) -> bool {
    kind == token_kind::PRIVATE_IDENT || kind == token_kind::PRIVATE_IDENT_ESCAPED
}

#[inline]
#[must_use]
pub const fn is_numeric_kind(kind: u8) -> bool {
    kind == token_kind::NUMBER
        || kind == token_kind::DECIMAL
        || kind == token_kind::FLOAT
        || kind == token_kind::BINARY
        || kind == token_kind::OCTAL
        || kind == token_kind::HEX
}

pub mod token_flags {
    pub const LINE_BEFORE: u16 = 1 << 0;
    pub const COMMENT_BEFORE: u16 = 1 << 1;
    pub const HAS_ESCAPE: u16 = 1 << 2;
    pub const HAS_NON_ASCII: u16 = 1 << 3;
    pub const CONTAINS_NEWLINE: u16 = 1 << 4;
    pub const UNTERMINATED: u16 = 1 << 5;
    pub const INVALID: u16 = 1 << 6;
    pub const LEGACY_OCTAL: u16 = 1 << 7;
    pub const ESCAPED_KEYWORD: u16 = 1 << 8;
    pub const REGEXP_VALIDATED: u16 = 1 << 9;
    pub const ASI_RESTRICTED: u16 = 1 << 10;
}

const _: () = assert!(token_kind::HASHBANG > token_kind::LINE_COMMENT);
const _: () = assert!(token_kind::HASHBANG < token_kind::LINE_TERMINATOR);

pub fn token_kind_name(kind: u8) -> &'static str {
    use token_kind::*;
    match kind {
        EOF => "EOF",
        IDENT => "IDENT",
        PRIVATE_IDENT => "PRIVATE_IDENT",
        NUMBER => "NUMBER",
        BIGINT => "BIGINT",
        STRING => "STRING",
        STRING_COOKED => "STRING_COOKED",
        IDENT_ESCAPED => "IDENT_ESCAPED",
        PRIVATE_IDENT_ESCAPED => "PRIVATE_IDENT_ESCAPED",
        TEMPLATE_NO_SUB_COOKED => "TEMPLATE_NO_SUB_COOKED",
        TEMPLATE_HEAD_COOKED => "TEMPLATE_HEAD_COOKED",
        TEMPLATE_MIDDLE_COOKED => "TEMPLATE_MIDDLE_COOKED",
        TEMPLATE_TAIL_COOKED => "TEMPLATE_TAIL_COOKED",
        DECIMAL => "DECIMAL",
        FLOAT => "FLOAT",
        BINARY => "BINARY",
        OCTAL => "OCTAL",
        HEX => "HEX",
        JSX_TEXT => "JSX_TEXT",
        JSX_TAG_END => "JSX_TAG_END",
        JSX_LT => "JSX_LT",
        REGEXP => "REGEXP",
        TEMPLATE_NO_SUB => "TEMPLATE_NO_SUB",
        TEMPLATE_HEAD => "TEMPLATE_HEAD",
        TEMPLATE_MIDDLE => "TEMPLATE_MIDDLE",
        TEMPLATE_TAIL => "TEMPLATE_TAIL",
        LINE_COMMENT => "LINE_COMMENT",
        BLOCK_COMMENT => "BLOCK_COMMENT",
        HASHBANG => "HASHBANG",
        WHITESPACE => "WHITESPACE",
        LINE_TERMINATOR => "LINE_TERMINATOR",
        LBRACE => "{",
        RBRACE => "}",
        LPAREN => "(",
        RPAREN => ")",
        LBRACKET => "[",
        RBRACKET => "]",
        DOT => ".",
        ELLIPSIS => "...",
        SEMI => ";",
        COMMA => ",",
        COLON => ":",
        QUESTION => "?",
        OPTIONAL_CHAIN => "?.",
        NULLISH => "??",
        NULLISH_EQ => "??=",
        ARROW => "=>",
        LT => "<",
        LE => "<=",
        GT => ">",
        GE => ">=",
        EQ => "=",
        EQ_EQ => "==",
        EQ_EQ_EQ => "===",
        BANG => "!",
        BANG_EQ => "!=",
        BANG_EQ_EQ => "!==",
        PLUS => "+",
        PLUS_PLUS => "++",
        PLUS_EQ => "+=",
        MINUS => "-",
        MINUS_MINUS => "--",
        MINUS_EQ => "-=",
        STAR => "*",
        STAR_STAR => "**",
        STAR_EQ => "*=",
        STAR_STAR_EQ => "**=",
        SLASH => "/",
        SLASH_EQ => "/=",
        PERCENT => "%",
        PERCENT_EQ => "%=",
        AMP => "&",
        AMP_AMP => "&&",
        AMP_EQ => "&=",
        AMP_AMP_EQ => "&&=",
        PIPE => "|",
        PIPE_PIPE => "||",
        PIPE_EQ => "|=",
        PIPE_PIPE_EQ => "||=",
        CARET => "^",
        CARET_EQ => "^=",
        TILDE => "~",
        LSHIFT => "<<",
        LSHIFT_EQ => "<<=",
        RSHIFT => ">>",
        RSHIFT_EQ => ">>=",
        URSHIFT => ">>>",
        URSHIFT_EQ => ">>>=",
        AT => "@",
        KW_BREAK => "break",
        KW_CASE => "case",
        KW_CATCH => "catch",
        KW_CLASS => "class",
        KW_CONST => "const",
        KW_CONTINUE => "continue",
        KW_DEBUGGER => "debugger",
        KW_DEFAULT => "default",
        KW_DELETE => "delete",
        KW_DO => "do",
        KW_ELSE => "else",
        KW_ENUM => "enum",
        KW_EXPORT => "export",
        KW_EXTENDS => "extends",
        KW_FALSE => "false",
        KW_FINALLY => "finally",
        KW_FOR => "for",
        KW_FUNCTION => "function",
        KW_IF => "if",
        KW_IMPORT => "import",
        KW_IN => "in",
        KW_INSTANCEOF => "instanceof",
        KW_NEW => "new",
        KW_NULL => "null",
        KW_RETURN => "return",
        KW_SUPER => "super",
        KW_SWITCH => "switch",
        KW_THIS => "this",
        KW_THROW => "throw",
        KW_TRUE => "true",
        KW_TRY => "try",
        KW_TYPEOF => "typeof",
        KW_VAR => "var",
        KW_VOID => "void",
        KW_WHILE => "while",
        KW_WITH => "with",
        KW_YIELD => "yield",
        KW_LET => "let",
        KW_STATIC => "static",
        KW_ASYNC => "async",
        KW_AWAIT => "await",
        KW_OF => "of",
        KW_FROM => "from",
        KW_AS => "as",
        KW_ABSTRACT => "abstract",
        KW_ACCESSOR => "accessor",
        KW_ANY => "any",
        KW_ASSERTS => "asserts",
        KW_BIGINT => "bigint",
        KW_BOOLEAN => "boolean",
        KW_DECLARE => "declare",
        KW_GLOBAL => "global",
        KW_IMPLEMENTS => "implements",
        KW_INFER => "infer",
        KW_INTERFACE => "interface",
        KW_INTRINSIC => "intrinsic",
        KW_IS => "is",
        KW_KEYOF => "keyof",
        KW_MODULE => "module",
        KW_NAMESPACE => "namespace",
        KW_NEVER => "never",
        KW_NUMBER => "number",
        KW_OBJECT => "object",
        KW_OUT => "out",
        KW_OVERRIDE => "override",
        KW_PACKAGE => "package",
        KW_PRIVATE => "private",
        KW_PROTECTED => "protected",
        KW_PUBLIC => "public",
        KW_READONLY => "readonly",
        KW_REQUIRE => "require",
        KW_SATISFIES => "satisfies",
        KW_STRING => "string",
        KW_SYMBOL => "symbol",
        KW_TYPE => "type",
        KW_UNDEFINED => "undefined",
        KW_UNIQUE => "unique",
        KW_UNKNOWN => "unknown",
        KW_USING => "using",
        INVALID => "INVALID",
        _ => "<unknown>",
    }
}

/// Bit 31 of a `starts` entry: reserved "newline before this token" flag.
/// The lexer does not set it yet, but consumers must still read offsets
/// through [`offset`].
pub const NEWLINE_BEFORE_MASK: u32 = 0x8000_0000;

pub const OFFSET_MASK: u32 = 0x7FFF_FFFF;

/// Maximum lexable source length in bytes, imposed by the 31-bit offset field.
pub const MAX_SOURCE_LEN: u32 = OFFSET_MASK;

#[inline]
pub const fn offset(s: u32) -> u32 {
    s & OFFSET_MASK
}

#[inline]
pub const fn newline(s: u32) -> bool {
    (s & NEWLINE_BEFORE_MASK) != 0
}

#[inline]
pub const fn pack_start(offset: u32, newline_before: bool) -> u32 {
    debug_assert!(offset <= OFFSET_MASK);
    if newline_before { offset | NEWLINE_BEFORE_MASK } else { offset }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct StringSpan {
    pub start: u32,
    pub end_and_flags: u32,
}

#[expect(clippy::inline_always, reason = "cook-path hot: spans are packed per string/template")]
impl StringSpan {
    pub const LONE_SURROGATES_MASK: u32 = 0x8000_0000;
    pub const END_MASK: u32 = 0x7FFF_FFFF;
    /// On `start`, template spans only: the body contained a
    /// `NotEscapeSequence` (bad `\u`/`\x`, octal, `\8`/`\9`). The equivalent
    /// of oxc_parser's `cooked: None`, from which a parser raises the
    /// untagged-template error. Raw `start` reads must mask.
    pub const COOKED_INVALID_MASK: u32 = 0x8000_0000;
    pub const START_MASK: u32 = 0x7FFF_FFFF;

    #[inline(always)]
    #[must_use]
    pub const fn new(start: u32, end: u32, lone_surrogates: bool) -> Self {
        debug_assert!(end <= Self::END_MASK);
        let end_and_flags = if lone_surrogates { end | Self::LONE_SURROGATES_MASK } else { end };
        Self { start, end_and_flags }
    }

    #[inline(always)]
    #[must_use]
    pub const fn with_cooked_invalid(self) -> Self {
        Self { start: self.start | Self::COOKED_INVALID_MASK, end_and_flags: self.end_and_flags }
    }

    #[inline(always)]
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start & Self::START_MASK
    }

    #[inline(always)]
    #[must_use]
    pub const fn cooked_invalid(self) -> bool {
        (self.start & Self::COOKED_INVALID_MASK) != 0
    }

    #[inline(always)]
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end_and_flags & Self::END_MASK
    }

    #[inline(always)]
    #[must_use]
    pub const fn lone_surrogates(self) -> bool {
        (self.end_and_flags & Self::LONE_SURROGATES_MASK) != 0
    }
}
