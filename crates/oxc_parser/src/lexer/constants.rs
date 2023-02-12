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

// Unused
// pub const fn is_regular_whitespace(c: char) -> bool {
//     matches!(c, ' ' | '\t')
// }

/// Whitespace where ASCII code > 127
pub const fn is_irregular_whitespace(c: char) -> bool {
    matches!(
        c,
        NBSP | ZWNBSP | '\u{85}' | '\u{1680}' | '\u{2000}'
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

pub const JUMP_TABLE: &[Kind; 128] = &[
    /*   0 */ Kind::Eof,
    /*   1 */ Kind::Undetermined,
    /*   2 */ Kind::Undetermined,
    /*   3 */ Kind::Undetermined,
    /*   4 */ Kind::Undetermined,
    /*   5 */ Kind::Undetermined,
    /*   6 */ Kind::Undetermined,
    /*   7 */ Kind::Undetermined,
    /*   8 */ Kind::Undetermined,
    /*   9 */ Kind::WhiteSpace, // Horizontal Tab
    /*  10 */ Kind::NewLine, // Line Feed
    /*  11 */ Kind::WhiteSpace, // Vertical Tab
    /*  12 */ Kind::WhiteSpace, // Form Feed
    /*  13 */ Kind::NewLine, // Carriage Return
    /*  14 */ Kind::Undetermined,
    /*  15 */ Kind::Undetermined,
    /*  16 */ Kind::Undetermined,
    /*  17 */ Kind::Undetermined,
    /*  18 */ Kind::Undetermined,
    /*  19 */ Kind::Undetermined,
    /*  20 */ Kind::Undetermined,
    /*  21 */ Kind::Undetermined,
    /*  22 */ Kind::Undetermined,
    /*  23 */ Kind::Undetermined,
    /*  24 */ Kind::Undetermined,
    /*  25 */ Kind::Undetermined,
    /*  26 */ Kind::Undetermined,
    /*  27 */ Kind::Undetermined,
    /*  28 */ Kind::Undetermined,
    /*  29 */ Kind::Undetermined,
    /*  30 */ Kind::Undetermined,
    /*  31 */ Kind::Undetermined,
    /*  32 */ Kind::WhiteSpace, // Space
    /*  33 */ Kind::Bang,
    /*  34 */ Kind::Str, // "
    /*  35 */ Kind::PrivateIdentifier, // #
    /*  36 */ Kind::Ident, // $
    /*  37 */ Kind::Percent,
    /*  38 */ Kind::Amp,
    /*  39 */ Kind::Str, // '
    /*  40 */ Kind::LParen,
    /*  41 */ Kind::RParen,
    /*  42 */ Kind::Star,
    /*  43 */ Kind::Plus,
    /*  44 */ Kind::Comma,
    /*  45 */ Kind::Minus,
    /*  46 */ Kind::Dot,
    /*  47 */ Kind::Slash,
    /*  48 */ Kind::Octal, // placeholder for 0 start
    /*  49 */ Kind::Decimal, // 1
    /*  50 */ Kind::Decimal, // 2
    /*  51 */ Kind::Decimal, // 3
    /*  52 */ Kind::Decimal, // 4
    /*  53 */ Kind::Decimal, // 5
    /*  54 */ Kind::Decimal, // 6
    /*  55 */ Kind::Decimal, // 7
    /*  56 */ Kind::Decimal, // 8
    /*  57 */ Kind::Decimal, // 9
    /*  58 */ Kind::Colon,
    /*  59 */ Kind::Semicolon,
    /*  60 */ Kind::LAngle,
    /*  61 */ Kind::Eq,
    /*  62 */ Kind::RAngle,
    /*  63 */ Kind::Question,
    /*  64 */ Kind::At,
    /*  65 */ Kind::Ident,
    /*  66 */ Kind::Ident,
    /*  67 */ Kind::Ident,
    /*  68 */ Kind::Ident,
    /*  69 */ Kind::Ident,
    /*  70 */ Kind::Ident,
    /*  71 */ Kind::Ident,
    /*  72 */ Kind::Ident,
    /*  73 */ Kind::Ident,
    /*  74 */ Kind::Ident,
    /*  75 */ Kind::Ident,
    /*  76 */ Kind::Ident,
    /*  77 */ Kind::Ident,
    /*  78 */ Kind::Ident,
    /*  79 */ Kind::Ident,
    /*  80 */ Kind::Ident,
    /*  81 */ Kind::Ident,
    /*  82 */ Kind::Ident,
    /*  83 */ Kind::Ident,
    /*  84 */ Kind::Ident,
    /*  85 */ Kind::Ident,
    /*  86 */ Kind::Ident,
    /*  87 */ Kind::Ident,
    /*  88 */ Kind::Ident,
    /*  89 */ Kind::Ident,
    /*  90 */ Kind::Ident,
    /*  91 */ Kind::LBrack,
    /*  92 */ Kind::Unknown, // placeholder for `\`
    /*  93 */ Kind::RBrack,
    /*  94 */ Kind::Caret,
    /*  95 */ Kind::Ident, // _
    /*  96 */ Kind::NoSubstitutionTemplate,
    /*  97 */ Kind::Ident,
    /*  98 */ Kind::Ident,
    /*  99 */ Kind::Ident,
    /* 100 */ Kind::Ident,
    /* 101 */ Kind::Ident,
    /* 102 */ Kind::Ident,
    /* 103 */ Kind::Ident,
    /* 104 */ Kind::Ident,
    /* 105 */ Kind::Ident,
    /* 106 */ Kind::Ident,
    /* 107 */ Kind::Ident,
    /* 108 */ Kind::Ident,
    /* 109 */ Kind::Ident,
    /* 110 */ Kind::Ident,
    /* 111 */ Kind::Ident,
    /* 112 */ Kind::Ident,
    /* 113 */ Kind::Ident,
    /* 114 */ Kind::Ident,
    /* 115 */ Kind::Ident,
    /* 116 */ Kind::Ident,
    /* 117 */ Kind::Ident,
    /* 118 */ Kind::Ident,
    /* 119 */ Kind::Ident,
    /* 120 */ Kind::Ident,
    /* 121 */ Kind::Ident,
    /* 122 */ Kind::Ident,
    /* 123 */ Kind::LCurly,
    /* 124 */ Kind::Pipe,
    /* 125 */ Kind::RCurly,
    /* 126 */ Kind::Tilde,
    /* 127 */ Kind::Undetermined,
];
