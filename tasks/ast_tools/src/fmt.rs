use std::process::Command;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use regex::{Captures, Regex, Replacer};
use syn::parse_file;

/// Pretty print
pub fn pretty_print(input: &TokenStream) -> String {
    let result = prettyplease::unparse(&parse_file(input.to_string().as_str()).unwrap());
    let result = COMMENT_REGEX.replace_all(&result, CommentReplacer).to_string();
    result
}

/// Run `cargo fmt`
pub fn cargo_fmt() {
    Command::new("cargo").arg("fmt").status().unwrap();
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
/// To dynamically generate a comment:
/// ```
/// let comment = format!("@ NOTE: {} doesn't exist!", name);
/// quote!(#[doc = #comment])
/// // or `quote!(#![doc = #comment])`
/// ```
///
/// `//!@@line_break` can be used to insert a line break in a position where `///@@line_break`
/// is not valid syntax e.g. before an `#![allow(...)]`.
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
