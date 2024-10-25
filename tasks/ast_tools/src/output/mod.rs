use std::{fs, io::Write, path::PathBuf};

use proc_macro2::TokenStream;

mod javascript;
mod rust;
use javascript::print_javascript;
use rust::print_rust;

/// An output from codegen.
///
/// Can be either Rust or Javascript.
pub enum Output {
    Rust { path: PathBuf, tokens: TokenStream },
    Javascript { path: PathBuf, code: String },
}

impl Output {
    pub fn output(self, generator_path: &str) -> RawOutput {
        let (path, code) = match self {
            Self::Rust { path, tokens } => {
                let code = print_rust(&tokens, generator_path);
                (path, code)
            }
            Self::Javascript { path, code } => {
                let code = print_javascript(&code, generator_path);
                (path, code)
            }
        };
        RawOutput { path, content: code.into_bytes() }
    }
}

/// A raw output from codegen.
#[derive(Debug)]
pub struct RawOutput {
    pub path: PathBuf,
    pub content: Vec<u8>,
}

impl RawOutput {
    /// Get path of output as a string.
    pub fn path(&self) -> String {
        let path = self.path.to_string_lossy();
        path.replace('\\', "/")
    }

    /// Write output to file
    pub fn write_to_file(self) -> std::io::Result<()> {
        let Self { path, content } = self;
        let path = path.into_os_string();
        let path = path.to_str().unwrap();
        write_all_to(&content, path)?;
        Ok(())
    }
}

/// Get path for an output.
pub fn output_path(krate: &str, path: &str) -> PathBuf {
    std::path::PathBuf::from_iter(vec![krate, "src", "generated", path])
}

/// Write data to file.
pub fn write_all_to<S: AsRef<std::path::Path>>(data: &[u8], path: S) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}
