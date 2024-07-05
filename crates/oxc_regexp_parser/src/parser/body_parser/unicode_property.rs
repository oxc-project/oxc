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

pub fn is_valid_unicode_property(name: &str, value: &str) -> bool {
    // TODO: Implement
    true
}

pub fn is_valid_lone_unicode_property(name_or_value: &str) -> bool {
    // TODO: Implement
    true
}

/// This should be used with `unicode_sets_mode`
pub fn is_valid_lone_unicode_property_of_strings(name_or_value: &str) -> bool {
    // TODO: Implement
    true
}
