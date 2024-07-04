pub fn is_syntax_character(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| {
        matches!(
            c,
            '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
        )
    })
}

pub fn is_decimal_digits(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| c.is_ascii_digit())
}

pub fn is_non_zero_digit(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, |c| c != '0' && c.is_ascii_digit())
}

pub fn is_id_continue(cp: u32) -> bool {
    char::from_u32(cp).map_or(false, unicode_id_start::is_id_continue_unicode)
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
