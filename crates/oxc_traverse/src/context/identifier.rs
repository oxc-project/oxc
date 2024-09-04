use std::borrow::Cow;

use oxc_syntax::identifier::{is_identifier_name, is_identifier_part, is_identifier_start};

/// Convert a str to a valid identifier name.
///
/// Based on Babel's [`toIdentifier`](https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-types/src/converters/toIdentifier.ts#L4-L26) function.
pub fn to_identifier(input: &str) -> Cow<str> {
    if is_identifier_name(input) {
        return Cow::Borrowed(input);
    }

    let mut name = String::with_capacity(input.len());

    let mut capitalize_next = false;

    let mut chars = input.chars();
    if let Some(first) = chars.next() {
        if is_identifier_start(first) {
            name.push(first);
        } else {
            capitalize_next = true;
        }
    }

    for c in chars {
        if !is_identifier_part(c) {
            capitalize_next = true;
        } else if capitalize_next {
            name.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            name.push(c);
        }
    }

    if name.is_empty() {
        return Cow::Borrowed("_");
    }

    Cow::Owned(name)
}

#[test]
fn test() {
    assert_eq!(to_identifier("foo"), "foo");
    assert_eq!(to_identifier("fooBar"), "fooBar");
    assert_eq!(to_identifier("fooBar1"), "fooBar1");

    assert_eq!(to_identifier("foo-bar"), "fooBar");
    assert_eq!(to_identifier("foo bar"), "fooBar");
    assert_eq!(to_identifier("foo-bar-1"), "fooBar1");
    assert_eq!(to_identifier("1foo-bar"), "FooBar");
    assert_eq!(to_identifier("1-foo-bar"), "FooBar");
    assert_eq!(to_identifier("-- --"), "_");

    assert_eq!(to_identifier("_output$headers$x-amzn-requestid"), "_output$headers$xAmznRequestid");
}
