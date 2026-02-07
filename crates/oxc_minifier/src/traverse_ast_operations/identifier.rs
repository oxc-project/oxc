use oxc_syntax::identifier::{is_identifier_name, is_identifier_part, is_identifier_start};

/// Convert a String to a valid identifier name.
///
/// Based on Babel's [`toIdentifier`] function.
///
/// [`toIdentifier`]: https://github.com/babel/babel/blob/3bcfee232506a4cebe410f02042fb0f0adeeb0b1/packages/babel-types/src/converters/toIdentifier.ts#L4-L26
pub fn to_identifier(input: String) -> String {
    if is_identifier_name(&input) {
        return input;
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
        return "_".to_string();
    }

    name
}

#[test]
fn test() {
    let cases = &[
        ("foo", "foo"),
        ("fooBar", "fooBar"),
        ("fooBar1", "fooBar1"),
        ("foo-bar", "fooBar"),
        ("foo bar", "fooBar"),
        ("foo-bar-1", "fooBar1"),
        ("1foo-bar", "FooBar"),
        ("1-foo-bar", "FooBar"),
        ("-- --", "_"),
        ("_output$headers$x-amzn-requestid", "_output$headers$xAmznRequestid"),
    ];

    for &(input, expected) in cases {
        assert_eq!(to_identifier(input.to_string()), expected);
    }
}
