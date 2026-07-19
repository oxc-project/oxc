use std::{
    io::Write,
    process::{Command, Stdio},
};

use lazy_regex::{Captures, Lazy, Regex, lazy_regex, regex::Replacer};
use proc_macro2::TokenStream;
use syn::parse2;

use crate::logln;

use super::add_header;

/// Format Rust code, and add header.
pub fn print_rust(tokens: &TokenStream, generator_path: &str) -> String {
    // Note: Cloning `TokenStream` is cheap, because internally it's an `Rc`
    let file = match parse2(tokens.clone()) {
        Ok(file) => file,
        Err(err) => {
            // Parsing failed. Return unformatted code, to aid debugging.
            logln!("FAILED TO PARSE Rust code:\n{err}");
            return tokens.to_string();
        }
    };

    let code = prettyplease::unparse(&file);
    let code = COMMENT_REGEX.replace_all(&code, CommentReplacer).to_string();
    let code = add_header(&code, generator_path, "//");
    rust_fmt(&code)
}

/// Format Rust code with `rustfmt`.
///
/// Does not format on disk - interfaces with `rustfmt` via stdin/stdout.
pub fn rust_fmt(source_text: &str) -> String {
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run rustfmt (is it installed?)");

    let stdin = rustfmt.stdin.as_mut().unwrap();
    stdin.write_all(source_text.as_bytes()).unwrap();
    stdin.flush().unwrap();

    let output = rustfmt.wait_with_output().unwrap();
    if output.status.success() {
        String::from_utf8(output.stdout).unwrap()
    } else {
        // Formatting failed. Return unformatted code, to aid debugging.
        let error =
            String::from_utf8(output.stderr).unwrap_or_else(|_| "Unknown error".to_string());
        logln!("FAILED TO FORMAT Rust code:\n{error}");
        source_text.to_string()
    }
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
/// is not valid syntax e.g. before an `#![expect(...)]`.
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

static COMMENT_REGEX: Lazy<Regex> = lazy_regex!(r"[ \t]*//[/!]@(.*)");
