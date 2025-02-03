use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use cow_utils::CowUtils;
use proc_macro2::TokenStream;

use crate::{log, log_result};

mod javascript;
mod rust;
mod yaml;
use javascript::print_javascript;
use rust::print_rust;
use yaml::print_yaml;

/// Get path for an output.
pub fn output_path(krate: &str, path: &str) -> String {
    format!("{krate}/src/generated/{path}")
}

/// Add a generated file warning to top of file.
fn add_header(code: &str, generator_path: &str, comment_start: &str) -> String {
    format!(
        "{comment_start} Auto-generated code, DO NOT EDIT DIRECTLY!\n\
        {comment_start} To edit this generated file you have to edit `{generator_path}`\n\n\
        {code}"
    )
}

/// An output from codegen.
///
/// Can be Rust, Javascript, or other formats.
#[expect(dead_code)]
pub enum Output {
    Rust { path: String, tokens: TokenStream },
    Javascript { path: String, code: String },
    Yaml { path: String, code: String },
    Raw { path: String, code: String },
}

impl Output {
    /// Convert [`Output`] to [`RawOutput`].
    ///
    /// This involves printing and formatting the output.
    pub fn into_raw(self, generator_path: &str) -> RawOutput {
        let generator_path = generator_path.cow_replace('\\', "/");

        let (path, code) = match self {
            Self::Rust { path, tokens } => {
                let code = print_rust(tokens, &generator_path);
                (path, code)
            }
            Self::Javascript { path, code } => {
                let code = print_javascript(&code, &generator_path);
                (path, code)
            }
            Self::Yaml { path, code } => {
                let code = print_yaml(&code, &generator_path);
                (path, code)
            }
            Self::Raw { path, code } => (path, code),
        };
        RawOutput { path, content: code.into_bytes() }
    }
}

/// A raw output from codegen.
///
/// Content is formatted, and in byte array form, ready to write to file.
#[derive(Debug)]
pub struct RawOutput {
    pub path: String,
    pub content: Vec<u8>,
}

impl RawOutput {
    /// Write [`RawOutput`] to file
    pub fn write_to_file(&self) -> io::Result<()> {
        log!("Write {}... ", &self.path);
        let result = write_to_file_impl(&self.content, &self.path);
        log_result!(result);
        result
    }
}

fn write_to_file_impl(data: &[u8], path: &str) -> io::Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(data)
}
