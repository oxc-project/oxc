use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use regex::{Captures, Regex, Replacer};
use syn::parse_file;

use crate::Result;

static INSERT_MACRO_IDENT: &str = "insert";
static INSERT_MACRO_IDENT_LEN: usize = INSERT_MACRO_IDENT.len();

static ENDL_MACRO_IDENT: &str = "endl";
static ENDL_MACRO_IDENT_LEN: usize = ENDL_MACRO_IDENT.len();

static WHITE_SPACES: &str = "   ";

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

struct EndlReplacer;

impl Replacer for EndlReplacer {
    fn replace_append(&mut self, _: &Captures, _: &mut String) {}
}

/// Pretty Print
pub fn pprint(input: &TokenStream) -> String {
    lazy_static! {
        static ref INSERT_REGEX: Regex = Regex::new(
            format!(
                r#"(?m)^[{WHITE_SPACES}]*{INSERT_MACRO_IDENT}!\([\n\s\S]*?\"([\n\s\S]*?)\"[\n\s\S]*?\);$"#
            )
            .as_str()
        )
        .unwrap();
    };

    lazy_static! {
        static ref ENDL_REGEX: Regex =
            Regex::new(format!(r"[{WHITE_SPACES}]*{ENDL_MACRO_IDENT}!\(\);").as_str()).unwrap();
    };

    let result = prettyplease::unparse(&parse_file(input.to_string().as_str()).unwrap());
    let result = ENDL_REGEX.replace_all(&result, EndlReplacer);
    let result = INSERT_REGEX.replace_all(&result, InsertReplacer).to_string();
    result
}

/// Runs cargo fmt.
pub fn cargo_fmt() {
    Command::new("cargo").arg("fmt").status().unwrap();
}
