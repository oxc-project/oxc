use std::{
    io::Write,
    process::{Command, Stdio},
};

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use regex::{Captures, Regex, Replacer};
use syn::parse_file;

/// Format Rust code, and add header.
pub fn print_rust(tokens: &TokenStream, generator_path: &str) -> String {
    let header = generate_header(generator_path);
    let tokens = quote! {
        #header
        #tokens
    };

    let result = prettyplease::unparse(&parse_file(tokens.to_string().as_str()).unwrap());
    let result = COMMENT_REGEX.replace_all(&result, CommentReplacer).to_string();
    rust_fmt(&result)
}

/// Creates a generated file warning + required information for a generated file.
fn generate_header(generator_path: &str) -> TokenStream {
    let generator_path = generator_path.replace('\\', "/");

    // TODO: Add generation date, AST source hash, etc here.
    let edit_comment = format!("@ To edit this generated file you have to edit `{generator_path}`");
    quote::quote! {
        //!@ Auto-generated code, DO NOT EDIT DIRECTLY!
        #![doc = #edit_comment]
        //!@@line_break
    }
}

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
