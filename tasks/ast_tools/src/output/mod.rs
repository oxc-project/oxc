use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use proc_macro2::TokenStream;

use crate::{log, log_result};

mod javascript;
mod rust;
mod yaml;
use javascript::print_javascript;
use rust::print_rust;
use yaml::print_yaml;

/// An output from codegen.
///
/// Can be either Rust or Javascript.
pub enum Output {
    Rust { path: String, tokens: TokenStream },
    Javascript { path: String, code: String },
    Yaml { path: String, code: String },
    Raw { path: String, code: String },
}

impl Output {
    pub fn into_raw(self, generator_path: &str) -> RawOutput {
        let (path, code) = match self {
            Self::Rust { path, tokens } => {
                let code = print_rust(&tokens, generator_path);
                (path, code)
            }
            Self::Javascript { path, code } => {
                let code = print_javascript(&code, generator_path);
                (path, code)
            }
            Self::Yaml { path, code } => {
                let code = print_yaml(&code, generator_path);
                (path, code)
            }
            Self::Raw { path, code } => (path, code),
        };
        RawOutput { path, content: code.into_bytes() }
    }
}

/// A raw output from codegen.
#[derive(Debug)]
pub struct RawOutput {
    pub path: String,
    pub content: Vec<u8>,
}

impl RawOutput {
    /// Write output to file
    pub fn write_to_file(&self) -> io::Result<()> {
        write_all_to(&self.content, &self.path)
    }
}

/// Get path for an output.
pub fn output_path(krate: &str, path: &str) -> String {
    format!("{krate}/src/generated/{path}")
}

/// Write data to file.
pub fn write_all_to<P: AsRef<Path>>(data: &[u8], path: P) -> io::Result<()> {
    let path = path.as_ref();
    log!("Write {}... ", path.to_string_lossy());
    let result = write_all_impl(data, path);
    log_result!(result);
    result
}

fn write_all_impl(data: &[u8], path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(data)
}
