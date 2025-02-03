use std::{
    io::Write,
    process::{Command, Stdio},
};

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use regex::{Captures, Regex, Replacer};
use syn::parse2;

use super::add_header;

/// Format Rust code, and add header.
pub fn print_rust(tokens: TokenStream, generator_path: &str) -> String {
    let code = prettyplease::unparse(&parse2(tokens).unwrap());
    let code = COMMENT_REGEX.replace_all(&code, CommentReplacer).to_string();
    let code = add_header(&code, generator_path, "//");
    rust_fmt(&code)
}

/// Format Rust code with `rustfmt`.
///
/// Does not format on disk - interfaces with `rustfmt` via stdin/stdout.
fn rust_fmt(source_text: &str) -> String {
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run rustfmt (is it installed?)");

    let stdin = rustfmt.stdin.as_mut().unwrap();
    stdin.write_all(source_text.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let output = rustfmt.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

/// Replace doc comments which start with `@` with plain comments or line breaks.
///
/// Original comment can be either `///@` or `//!@`.
///
/// * `///@ foo` becomes `// foo`.
/// * `//!@ foo` becomes `// foo`.
/// * `///@@line_break` is removed - i.e. line break.
/// * `//!@@line_break` is removed - i.e. line break.
///
/// `quote!` macro ignores plain comments, but we can use these to generate plain comments
/// in generated code.
///
/// `//!@` form can be used to insert a line break in a position where `///@ ...`
/// is not valid syntax e.g. before an `#![allow(...)]`.
///
/// To dynamically generate a comment:
/// ```
/// let comment = format!("@ NOTE: {} doesn't exist!", name);
/// quote!( #[doc = #comment] )
/// // or `quote!( #![doc = #comment] )`
/// ```
struct CommentReplacer;

impl Replacer for CommentReplacer {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 2);
        let body = caps.get(1).unwrap().as_str();
        if body != "@line_break" {
            dst.push_str("//");
            dst.push_str(body);
        }
    }
}

lazy_static! {
    static ref COMMENT_REGEX: Regex = Regex::new(r"[ \t]*//[/!]@(.*)").unwrap();
}
