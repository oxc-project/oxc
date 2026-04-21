pub mod ast;
mod diagnostics;
mod parser_impl;

pub use parser_impl::Parser;

#[cfg(test)]
mod test {
    use crate::parser::reader::{Options, template_literal_parser::parser_impl::Parser};

    #[test]
    fn should_pass() {
        for source_text in [
            "``",
            "`Hello, world!`",
            r#"`He said, "Hello!"`"#,
            r#"`She said, "Hello!"`"#,
            r"`It's a sunny day`",
            r"`Line1\nLine2`",
            r"`Column1\tColumn2`",
            r"`Path to file: C:\\Program Files\\MyApp`",
            r"`Backspace\bTest`",
            r"`FormFeed\fTest`",
            r"`CarriageReturn\rTest`",
            r"`TestWithValidDollarSignAtTheEnd$`",
            r"`TestWithValid$DollarSignBetween`",
            r"`VerticalTab\vTest`",
            "`   `",                        // whitespace only
            r"`Escaped \``",                // escaped backtick
            r"`Unicode: \u0041`",           // unicode escape
            r"`Code point: \u{1F600}`",     // unicode code point escape
            r"`Hex: \x41`",                 // hex escape
            "`Line1\\\nLine2`",             // line continuation
            "`Price: $100`",                // dollar sign not followed by brace
            r"`Smile: \uD83D\uDE00`",       // surrogate pair
            r"`Mix: \n\t\x41\u0042\u{43}`", // multiple escapes
            r"`Dollar: \$`",                // escaped dollar
            r"`Null: \0`",                  // should be valid if not followed by a digit
            r"`Surrogate: \uD800`", // lone high surrogate, not a valid code point but no error is reported
            r"`Valid: \z`",
            "`This is
            a multi-line
            template literal`",
        ] {
            if let Err(err) = Parser::new(source_text, Options::default()).parse() {
                panic!("Expect to parse: {source_text} but failed: {err}");
            }
        }
    }

    #[test]
    fn should_fail() {
        for source_text in [
            "`Unclosed template literal",
            r"`Invalid hex escape: \xG1`",
            r"`Invalid unicode escape: \u{G1}`",
            r"`Template with ${expression}`", // expression not supported
            r"`Incomplete: \u`",
            r"`Incomplete: \u{}`",
            r"`Incomplete: \u{110000}`", // out of Unicode range
            r"`Incomplete: \x`",
            r"`Incomplete: \x4`",
            "`Unescaped backtick: ``",
            r"`Line continuation: \\n\xG1`",
            r"`Dollar: ${{}`", // should fail, as `${` is not supported
        ] {
            let result = Parser::new(source_text, Options::default()).parse();
            assert!(result.is_err(), "Expect to fail: {source_text} but passed...");
        }
    }
}
