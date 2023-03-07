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

pub fn is_irregular_whitespace(c: char) -> bool {
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

pub fn is_regular_line_terminator(c: char) -> bool {
    matches!(c, LF | CR)
}

pub fn is_irregular_line_terminator(c: char) -> bool {
    matches!(c, LS | PS)
}

pub fn is_line_terminator(c: char) -> bool {
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
    /*   0 */ Kind::Undetermined,
    /*   1 */ Kind::Undetermined,
    /*   2 */ Kind::Undetermined,
    /*   3 */ Kind::Undetermined,
    /*   4 */ Kind::Undetermined,
    /*   5 */ Kind::Undetermined,
    /*   6 */ Kind::Undetermined,
    /*   7 */ Kind::Undetermined,
    /*   8 */ Kind::Undetermined,
    /*   9 */ Kind::Undetermined,
    /*  10 */ Kind::Undetermined,
    /*  11 */ Kind::Undetermined,
    /*  12 */ Kind::Undetermined,
    /*  13 */ Kind::Undetermined,
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
    /*  32 */ Kind::Undetermined,
    /*  33 */ Kind::Undetermined,
    /*  34 */ Kind::Undetermined,
    /*  35 */ Kind::Undetermined,
    /*  36 */ Kind::Undetermined,
    /*  37 */ Kind::Undetermined,
    /*  38 */ Kind::Undetermined,
    /*  39 */ Kind::Undetermined,
    /*  40 */ Kind::LParen, // 0x28
    /*  41 */ Kind::RParen, // 0x29
    /*  42 */ Kind::Undetermined,
    /*  43 */ Kind::Undetermined,
    /*  44 */ Kind::Comma, // 0x2C
    /*  45 */ Kind::Undetermined,
    /*  46 */ Kind::Undetermined,
    /*  47 */ Kind::Undetermined,
    /*  48 */ Kind::Undetermined,
    /*  49 */ Kind::Undetermined,
    /*  50 */ Kind::Undetermined,
    /*  51 */ Kind::Undetermined,
    /*  52 */ Kind::Undetermined,
    /*  53 */ Kind::Undetermined,
    /*  54 */ Kind::Undetermined,
    /*  55 */ Kind::Undetermined,
    /*  56 */ Kind::Undetermined,
    /*  57 */ Kind::Undetermined,
    /*  58 */ Kind::Colon, // 0x3A
    /*  59 */ Kind::Semicolon, // 0x3B
    /*  60 */ Kind::Undetermined,
    /*  61 */ Kind::Undetermined,
    /*  62 */ Kind::RAngle, // 0x3E
    /*  63 */ Kind::Undetermined,
    /*  64 */ Kind::At,
    /*  65 */ Kind::Undetermined,
    /*  66 */ Kind::Undetermined,
    /*  67 */ Kind::Undetermined,
    /*  68 */ Kind::Undetermined,
    /*  69 */ Kind::Undetermined,
    /*  70 */ Kind::Undetermined,
    /*  71 */ Kind::Undetermined,
    /*  72 */ Kind::Undetermined,
    /*  73 */ Kind::Undetermined,
    /*  74 */ Kind::Undetermined,
    /*  75 */ Kind::Undetermined,
    /*  76 */ Kind::Undetermined,
    /*  77 */ Kind::Undetermined,
    /*  78 */ Kind::Undetermined,
    /*  79 */ Kind::Undetermined,
    /*  80 */ Kind::Undetermined,
    /*  81 */ Kind::Undetermined,
    /*  82 */ Kind::Undetermined,
    /*  83 */ Kind::Undetermined,
    /*  84 */ Kind::Undetermined,
    /*  85 */ Kind::Undetermined,
    /*  86 */ Kind::Undetermined,
    /*  87 */ Kind::Undetermined,
    /*  88 */ Kind::Undetermined,
    /*  89 */ Kind::Undetermined,
    /*  90 */ Kind::Undetermined,
    /*  91 */ Kind::LBrack, // 0x5B
    /*  92 */ Kind::Undetermined,
    /*  93 */ Kind::RBrack, // 0x5D
    /*  94 */ Kind::Undetermined,
    /*  95 */ Kind::Undetermined,
    /*  96 */ Kind::Undetermined,
    /*  97 */ Kind::Undetermined,
    /*  98 */ Kind::Undetermined,
    /*  99 */ Kind::Undetermined,
    /* 100 */ Kind::Undetermined,
    /* 101 */ Kind::Undetermined,
    /* 102 */ Kind::Undetermined,
    /* 103 */ Kind::Undetermined,
    /* 104 */ Kind::Undetermined,
    /* 105 */ Kind::Undetermined,
    /* 106 */ Kind::Undetermined,
    /* 107 */ Kind::Undetermined,
    /* 108 */ Kind::Undetermined,
    /* 109 */ Kind::Undetermined,
    /* 110 */ Kind::Undetermined,
    /* 111 */ Kind::Undetermined,
    /* 112 */ Kind::Undetermined,
    /* 113 */ Kind::Undetermined,
    /* 114 */ Kind::Undetermined,
    /* 115 */ Kind::Undetermined,
    /* 116 */ Kind::Undetermined,
    /* 117 */ Kind::Undetermined,
    /* 118 */ Kind::Undetermined,
    /* 119 */ Kind::Undetermined,
    /* 120 */ Kind::Undetermined,
    /* 121 */ Kind::Undetermined,
    /* 122 */ Kind::Undetermined,
    /* 123 */ Kind::LCurly, // 0x7B
    /* 124 */ Kind::Undetermined,
    /* 125 */ Kind::RCurly, // 0x7D
    /* 126 */ Kind::Tilde, // 0x7E
    /* 127 */ Kind::Undetermined,
];
