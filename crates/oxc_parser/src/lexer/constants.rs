use unicode_id_start::{is_id_continue, is_id_start};

use super::Kind;

pub const EOF: char = '\0';

// 11.1 Unicode Format-Control Characters

/// U+200C ZERO WIDTH NON-JOINER, abbreviated in the spec as <ZWNJ>.
/// Specially permitted in identifiers.
pub const ZWNJ: char = '\u{200c}';

/// U+200D ZERO WIDTH JOINER, abbreviated as <ZWJ>.
/// Specially permitted in identifiers.
pub const ZWJ: char = '\u{200d}';

/// U+FEFF ZERO WIDTH NO-BREAK SPACE, abbreviated <ZWNBSP>.
/// Considered a whitespace character in JS.
pub const ZWNBSP: char = '\u{feff}';

// 11.2 White Space
/// U+0009 CHARACTER TABULATION, abbreviated <TAB>.
pub const TAB: char = '\u{9}';

/// U+000B VERTICAL TAB, abbreviated <VT>.
pub const VT: char = '\u{b}';

/// U+000C FORM FEED, abbreviated <FF>.
pub const FF: char = '\u{c}';

/// U+00A0 NON-BREAKING SPACE, abbreviated <NBSP>.
pub const NBSP: char = '\u{a0}';

pub const fn is_regular_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t')
}

pub const fn is_irregular_whitespace(c: char) -> bool {
    matches!(
        c,
        VT | FF | NBSP | ZWNBSP | '\u{85}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

// 11.3 Line Terminators

///  U+000A LINE FEED, abbreviated in the spec as <LF>.
pub const LF: char = '\u{a}';

/// U+000D CARRIAGE RETURN, abbreviated in the spec as <CR>.
pub const CR: char = '\u{d}';

/// U+2028 LINE SEPARATOR, abbreviated <LS>.
pub const LS: char = '\u{2028}';

/// U+2029 PARAGRAPH SEPARATOR, abbreviated <PS>.
pub const PS: char = '\u{2029}';

pub const fn is_regular_line_terminator(c: char) -> bool {
    matches!(c, LF | CR)
}

pub const fn is_irregular_line_terminator(c: char) -> bool {
    matches!(c, LS | PS)
}

pub const fn is_line_terminator(c: char) -> bool {
    is_regular_line_terminator(c) || is_irregular_line_terminator(c)
}

/// Section 12.6 Detect `IdentifierStartChar`
#[inline]
pub fn is_identifier_start(c: char) -> bool {
    c == '$' || c == '_' || is_id_start(c)
}

/// Section 12.6 Detect `IdentifierPartChar`
/// NOTE 2: The nonterminal `IdentifierPart` derives _ via `UnicodeIDContinue`.
#[inline]
pub fn is_identifier_part(c: char) -> bool {
    c == '$' || is_id_continue(c) || c == ZWNJ || c == ZWJ
}

pub const SINGLE_CHAR_TOKENS: &[Kind; 128] = &[
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::LParen, // 0x28
    Kind::RParen, // 0x29
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Comma, // 0x2C
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Colon,     // 0x3A
    Kind::Semicolon, // 0x3B
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::RAngle, // 0x3E
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::LBrack, // 0x5B
    Kind::Undetermined,
    Kind::RBrack, // 0x5D
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::Undetermined,
    Kind::LCurly, // 0x7B
    Kind::Undetermined,
    Kind::RCurly, // 0x7D
    Kind::Tilde,  // 0x7E
    Kind::Undetermined,
];
