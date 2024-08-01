// ```
// SyntaxCharacter :: one of
//   ^ $ \ . * + ? ( ) [ ] { } |
// ```
pub fn is_syntax_character(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| {
        matches!(
            c,
            '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
        )
    })
}

// ```
// ClassSetSyntaxCharacter :: one of
//   ( ) [ ] { } / - \ |
// ```
pub fn is_class_set_syntax_character(cp: u32) -> bool {
    char::from_u32(cp)
        .map_or(false, |c| matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | '/' | '-' | '\\' | '|'))
}

// ```
// ClassSetReservedDoublePunctuator :: one of
//   && !! ## $$ %% ** ++ ,, .. :: ;; << == >> ?? @@ ^^ `` ~~
// ````
pub fn is_class_set_reserved_double_punctuator(cp1: u32, cp2: u32) -> bool {
    char::from_u32(cp1).map_or(false, |c1| {
        char::from_u32(cp2).map_or(false, |c2| {
            matches!(
                (c1, c2),
                ('&', '&')
                    | ('!', '!')
                    | ('#', '#')
                    | ('$', '$')
                    | ('%', '%')
                    | ('*', '*')
                    | ('+', '+')
                    | (',', ',')
                    | ('.', '.')
                    | (':', ':')
                    | (';', ';')
                    | ('<', '<')
                    | ('=', '=')
                    | ('>', '>')
                    | ('?', '?')
                    | ('@', '@')
                    | ('^', '^')
                    | ('`', '`')
                    | ('~', '~')
            )
        })
    })
}

// ```
// ClassSetReservedPunctuator :: one of
//   & - ! # % , : ; < = > @ ` ~
// ```
pub fn is_class_set_reserved_punctuator(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| {
        matches!(
            c,
            '&' | '-' | '!' | '#' | '%' | ',' | ':' | ';' | '<' | '=' | '>' | '@' | '`' | '~'
        )
    })
}

pub fn is_decimal_digit(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| c.is_ascii_digit())
}

pub fn is_octal_digit(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| c.is_ascii_digit() && c < '8')
}

pub fn is_valid_unicode(cp: u32) -> bool {
    (0..=0x0010_ffff).contains(&cp)
}

// ```
// UnicodePropertyNameCharacter ::
//   AsciiLetter
//   _
// ```
// <https://tc39.es/ecma262/#prod-UnicodePropertyNameCharacter>
pub fn is_unicode_property_name_character(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
}

// ```
// UnicodePropertyValueCharacter ::
//   UnicodePropertyNameCharacter
//   DecimalDigit
// ```
// <https://tc39.es/ecma262/#prod-UnicodePropertyValueCharacter>
pub fn is_unicode_property_value_character(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn is_unicode_id_start(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, unicode_id_start::is_id_start)
}
pub fn is_unicode_id_continue(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, unicode_id_start::is_id_continue)
}

pub fn is_identifier_start_char(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| unicode_id_start::is_id_start(c) || c == '$' || c == '_')
}

pub fn is_identifier_part_char(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| unicode_id_start::is_id_continue(c) || c == '$')
}

pub fn is_lead_surrogate(cp: u32) -> bool {
    (0xd800..=0xdbff).contains(&cp)
}

pub fn is_trail_surrogate(cp: u32) -> bool {
    (0xdc00..=0xdfff).contains(&cp)
}

pub fn combine_surrogate_pair(lead: u32, trail: u32) -> u32 {
    (lead - 0xd800) * 0x400 + trail - 0xdc00 + 0x10000
}

pub fn map_control_escape(cp: u32) -> Option<u32> {
    match char::from_u32(cp) {
        Some('f') => Some(0x0c),
        Some('n') => Some(0x0a),
        Some('r') => Some(0x0d),
        Some('t') => Some(0x09),
        Some('v') => Some(0x0b),
        _ => None,
    }
}

pub fn map_c_ascii_letter(cp: u32) -> Option<u32> {
    char::from_u32(cp).and_then(|c| c.is_ascii_alphabetic().then_some(cp % 0x20))
}

pub fn map_hex_digit(cp: u32) -> Option<u32> {
    char::from_u32(cp).filter(char::is_ascii_hexdigit).and_then(|c| c.to_digit(16))
}
