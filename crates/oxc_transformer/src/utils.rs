use oxc_syntax::{identifier::is_identifier_name, keyword::is_keyword};

pub fn is_valid_identifier(name: &str, reserved: bool) -> bool {
    if reserved && (is_keyword(name) || is_reserved_word(name, true)) {
        return false;
    }
    is_identifier_name(name)
}

pub fn is_reserved_word(name: &str, in_module: bool) -> bool {
    (in_module && name == "await") || name == "enum"
}
