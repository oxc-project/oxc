pub fn is_syntax_character(c: u32) -> bool {
    char::from_u32(c).map_or(false, |c| {
        matches!(
            c,
            '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
        )
    })
}

pub fn is_decimal_digits(c: u32) -> bool {
    char::from_u32(c)
        .map_or(false, |c| matches!(c, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'))
}

pub fn is_non_zero_digit(c: u32) -> bool {
    char::from_u32(c)
        .map_or(false, |c| matches!(c, '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'))
}
