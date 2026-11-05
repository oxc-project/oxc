pub const LF: char = '\n';
pub const CR: char = '\r';
pub const LS: char = '\u{2028}';
pub const PS: char = '\u{2029}';

// ```
// LineTerminator ::
//   <LF>
//   <CR>
//   <LS>
//   <PS>
// ```
pub fn is_line_terminator(ch: char) -> bool {
    matches!(ch, LF | CR | LS | PS)
}

// ```
// SingleEscapeCharacter :: one of
//   ' " \ b f n r t v
// ```
pub fn is_single_escape_character(ch: char) -> bool {
    map_single_escape_character(ch).is_some()
}

/// Code unit value of a `SingleEscapeCharacter` (ECMA-262, Table 33).
pub fn map_single_escape_character(ch: char) -> Option<u32> {
    Some(match ch {
        'b' => 0x08,
        't' => 0x09,
        'n' => 0x0a,
        'v' => 0x0b,
        'f' => 0x0c,
        'r' => 0x0d,
        '\'' | '"' | '\\' => ch as u32,
        _ => return None,
    })
}

// ```
// NonEscapeCharacter ::
//   SourceCharacter but not one of EscapeCharacter or LineTerminator
//
// EscapeCharacter ::
//   SingleEscapeCharacter
//   DecimalDigit
//   x
//   u
// ```
pub fn is_non_escape_character(ch: char) -> bool {
    let is_escape_character =
        |ch| is_single_escape_character(ch) || ch.is_ascii_digit() || matches!(ch, 'x' | 'u');

    if is_escape_character(ch) || is_line_terminator(ch) {
        return false;
    }
    true
}
