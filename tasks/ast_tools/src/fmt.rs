use std::process::Command;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use regex::{Captures, Regex, Replacer};
use syn::parse_file;

static INSERT_MACRO_IDENT: &str = "insert";
static ENDL_MACRO_IDENT: &str = "endl";
static WHITE_SPACES: &str = " \t";

/// Pretty print
pub fn pretty_print(input: &TokenStream) -> String {
    let result = prettyplease::unparse(&parse_file(input.to_string().as_str()).unwrap());
    // `insert!` and `endl!` macros are not currently used
    // let result = ENDL_REGEX.replace_all(&result, EndlReplacer);
    // let result = INSERT_REGEX.replace_all(&result, InsertReplacer).to_string();
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
    static ref COMMENT_REGEX: Regex =
        Regex::new(format!(r"[{WHITE_SPACES}]*//[/!]@(.*)").as_str()).unwrap();
}

/// Replace `insert!` macro calls with the contents of the `insert!`.
///
/// e.g. `insert!("#![allow(dead_code)]")` is replaced by `#![allow(dead_code)]`.
///
/// We use this when inserting outer attributes (`#![allow(unused)]`) or plain comments (`//` not `///`).
/// `quote!` macro ignores plain comments, so it's not possible to produce them otherwise.
#[expect(dead_code)] // `insert!` macro is not currently used
struct InsertReplacer;

impl Replacer for InsertReplacer {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        assert_eq!(caps.len(), 2);
        let arg = caps.get(1);
        if let Some(arg) = arg {
            dst.push_str(arg.as_str());
        }
    }
}

lazy_static! {
    static ref INSERT_REGEX: Regex = Regex::new(
        format!(
            r#"(?m)^[{WHITE_SPACES}]*{INSERT_MACRO_IDENT}!\([\n\s\S]*?\"([\n\s\S]*?)\"[\n\s\S]*?\);$"#
        )
        .as_str()
    )
    .unwrap();
}

/// Remove `endl!();`, so it produces a line break.
///
/// e.g.:
/// ```
/// use oxc_allocator::Allocator;
/// endl!();
/// use oxc_ast::*;
/// ```
/// becomes:
/// ```
/// use oxc_allocator::Allocator;
///
/// use oxc_ast::*;
/// ```
///
/// We use `endl!();` because `quote!` macro ignores whitespace,
/// so we have to use another means to generate line breaks.
#[expect(dead_code)] // `endl!` macro is not currently used
struct EndlReplacer;

impl Replacer for EndlReplacer {
    fn replace_append(&mut self, _: &Captures, _: &mut String) {}
}

lazy_static! {
    static ref ENDL_REGEX: Regex =
        Regex::new(format!(r"[{WHITE_SPACES}]*{ENDL_MACRO_IDENT}!\(\);").as_str()).unwrap();
}
