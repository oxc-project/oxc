use phf::phf_set;

const SYNTAX_CHARACTERS: phf::Set<char> = phf_set!['(', ')', '[', ']', '{', '}', '|', '-'];

const CLASS_SET_RESERVED_DOUBLE_PUNCTUATOR_CHARACTER: phf::Set<char> = phf_set! {
    '&' => AMPERSAND,
    '!' => EXCLAMATION_MARK,
    '#' => NUMBER_SIGN,
    '$' => DOLLAR_SIGN,
    '%' => PERCENT_SIGN,
    '*' => ASTERISK,
    '+' => PLUS_SIGN,
    ',' => COMMA,
    '.' => FULL_STOP,
    ':' => COLON,
    ';' => SEMICOLON,
    '<' => LESS_THAN_SIGN,
    '=' => EQUALS_SIGN,
    '>' => GREATER_THAN_SIGN,
    '?' => QUESTION_MARK,
    '@' => COMMERCIAL_AT,
    '^' => CIRCUMFLEX_ACCENT,
    '`' => GRAVE_ACCENT,
    '~' => TILDE,
};

const CLASS_SET_SYNTAX_CHARACTER: phf::Set<char> = phf_set! {
    '(' => LEFT_PARENTHESIS,
    ')' => RIGHT_PARENTHESIS,
    '[' => LEFT_SQUARE_BRACKET,
    ']' => RIGHT_SQUARE_BRACKET,
    '{' => LEFT_CURLY_BRACKET,
    '}' => RIGHT_CURLY_BRACKET,
    '/' => SOLIDUS,
    '-' => HYPHEN_MINUS,
    '\\' => REVERSE_SOLIDUS,
    '|' => VERTICAL_LINE,
};

const CLASS_SET_RESERVED_PUNCTUATOR: phf::Set<char> = phf_set! {
    '&' => AMPERSAND,
    '-' => HYPHEN_MINUS,
    '!' => EXCLAMATION_MARK,
    '#' => NUMBER_SIGN,
    '%' => PERCENT_SIGN,
    ',' => COMMA,
    ':' => COLON,
    ';' => SEMICOLON,
    '<' => LESS_THAN_SIGN,
    '=' => EQUALS_SIGN,
    '>' => GREATER_THAN_SIGN,
    '@' => COMMERCIAL_AT,
    '`' => GRAVE_ACCENT,
    '~' => TILDE,
};

#[inline]
pub fn is_syntax_character(cp: char) -> bool {
    SYNTAX_CHARACTERS.contains(&cp)
}

pub fn is_lead_surrogate(code: u32) -> bool {
    code >= 0xd800 && code <= 0xdbff
}

pub fn is_trail_surrogate(code: u32) -> bool {
    code >= 0xdc00 && code <= 0xdfff
}

pub fn combine_surrogate_pair(lead: u32, trail: u32) -> u32 {
    (lead - 0xd800) * 0x400 + (trail - 0xdc00) + 0x10000
}

pub fn is_class_set_reserved_double_punctuator_character(cp: char) -> bool {
    CLASS_SET_RESERVED_DOUBLE_PUNCTUATOR_CHARACTER.contains(&cp)
}

pub fn is_class_set_syntax_character(cp: char) -> bool {
    CLASS_SET_SYNTAX_CHARACTER.contains(&cp)
}

pub fn is_class_set_reserved_punctuator(cp: char) -> bool {
    CLASS_SET_RESERVED_PUNCTUATOR.contains(&cp)
}
