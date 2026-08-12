//! Build-script helpers for formatter fixture tests.
//!
//! Walks a `tests/fixtures/` directory and emits `#[test]` functions into `$OUT_DIR/generated_tests.rs`
//! that each call a per-crate `test_file(path)` helper. The generated file is meant to be
//! consumed via `include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"))` from the
//! integration-test target.
//!
//! Language is parameterized by [`GenerateConfig::extensions`] — only files whose extension is
//! in that list are picked up as test inputs.

// This module is exclusively called from `build.rs` scripts, where `println!`
// is the documented way to emit Cargo directives.
#![allow(clippy::print_stdout)]

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Write},
    path::{MAIN_SEPARATOR, Path, PathBuf},
};

/// Build-script generation parameters.
pub struct GenerateConfig<'a> {
    /// Extensions (without the leading dot) recognized as test inputs.
    /// e.g. `&["js", "jsx", "ts", "tsx"]` for JS, `&["json", "jsonc", "json5"]` for JSON.
    pub extensions: &'a [&'a str],
}

/// Walks `fixtures_dir` and writes an auto-generated test module tree to `out_file`.
///
/// Also emits `cargo:rerun-if-changed=<fixtures_dir>` so Cargo re-runs the build script when
/// fixtures change.
///
/// # Errors
/// Propagates filesystem errors from directory traversal or file write.
pub fn generate_tests(
    out_file: &Path,
    fixtures_dir: &Path,
    config: &GenerateConfig,
) -> io::Result<()> {
    let mut f = File::create(out_file)?;

    if !fixtures_dir.exists() {
        writeln!(f, "// No test fixtures found")?;
        return Ok(());
    }

    writeln!(f, "// Auto-generated test modules and functions")?;
    writeln!(f)?;

    let mut dir_structure = DirStructure::default();
    collect_tests(fixtures_dir, fixtures_dir, &mut dir_structure, config.extensions)?;

    generate_modules(&mut f, &dir_structure, 0, config.extensions)?;

    println!("cargo:rerun-if-changed={}", fixtures_dir.display());

    Ok(())
}

#[derive(Default)]
struct DirStructure {
    /// Test files in this directory (relative paths from fixtures root).
    test_files: Vec<PathBuf>,
    /// Subdirectories keyed by name.
    subdirs: BTreeMap<String, DirStructure>,
}

impl DirStructure {
    fn has_test_files(&self) -> bool {
        !self.test_files.is_empty() || self.subdirs.values().any(Self::has_test_files)
    }
}

fn collect_tests(
    dir: &Path,
    base_dir: &Path,
    structure: &mut DirStructure,
    extensions: &[&str],
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            let subdir = structure.subdirs.entry(dir_name).or_default();
            collect_tests(&path, base_dir, subdir, extensions)?;
        } else if is_test_file(&path, extensions) {
            let relative_path = path.strip_prefix(base_dir).unwrap().to_path_buf();
            structure.test_files.push(relative_path);
        }
    }
    Ok(())
}

fn generate_modules(
    f: &mut File,
    structure: &DirStructure,
    indent_level: usize,
    extensions: &[&str],
) -> io::Result<()> {
    let indent = "    ".repeat(indent_level);

    for test_file in &structure.test_files {
        generate_test_function(f, test_file, indent_level, extensions)?;
    }

    for (dir_name, subdir) in &structure.subdirs {
        let module_name = sanitize_module_name(dir_name);

        writeln!(f, "{indent}#[cfg(test)]")?;
        writeln!(f, "{indent}mod {module_name} {{")?;
        if subdir.has_test_files() {
            writeln!(f, "{indent}    use super::test_file;")?;
        }
        writeln!(f)?;

        generate_modules(f, subdir, indent_level + 1, extensions)?;

        writeln!(f, "{indent}}}")?;
        writeln!(f)?;
    }

    Ok(())
}

fn is_test_file(path: &Path, extensions: &[&str]) -> bool {
    // `options.json` is the harness's per-directory option overrides, not a test input.
    // Without this skip the JSON formatter (whose extensions include `json`) would try
    // to format the option file itself.
    if path.file_name().and_then(|n| n.to_str()) == Some("options.json") {
        return false;
    }
    path.extension().and_then(|e| e.to_str()).is_some_and(|ext| extensions.contains(&ext))
}

#[expect(clippy::disallowed_methods)]
fn generate_test_function(
    f: &mut File,
    relative_path: &Path,
    indent_level: usize,
    extensions: &[&str],
) -> io::Result<()> {
    let indent = "    ".repeat(indent_level);
    let filename = relative_path.file_name().unwrap().to_str().unwrap();
    let test_name = file_to_test_name(filename, extensions);

    writeln!(f, "{indent}#[test]")?;
    writeln!(f, "{indent}fn {test_name}() {{")?;
    writeln!(
        f,
        "{}    let path = std::path::Path::new(\"tests/fixtures/{}\");",
        indent,
        relative_path.display().to_string().replace(MAIN_SEPARATOR, "/")
    )?;
    writeln!(f, "{indent}    test_file(path);")?;
    writeln!(f, "{indent}}}")?;
    writeln!(f)?;

    Ok(())
}

fn file_to_test_name(filename: &str, extensions: &[&str]) -> String {
    let mut name = String::new();
    for c in filename.chars() {
        if c.is_alphanumeric() {
            name.push(c.to_ascii_lowercase());
        } else {
            name.push('_');
        }
    }

    // Strip the trailing `_<ext>` token if it matches one of the configured extensions.
    if let Some(pos) = name.rfind('_') {
        let after = &name[pos + 1..];
        if extensions.iter().any(|e| e.eq_ignore_ascii_case(after)) {
            name.truncate(pos);
        }
    }

    sanitize_identifier(name, "test")
}

fn sanitize_module_name(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push('_');
        }
    }
    sanitize_identifier(result, "")
}

fn sanitize_identifier(mut name: String, prefix: &str) -> String {
    if name.is_empty() || name.chars().next().unwrap().is_numeric() {
        name = if prefix.is_empty() { format!("_{name}") } else { format!("{prefix}_{name}") };
    }
    if is_reserved_keyword(&name) {
        return format!("r#{name}");
    }
    name
}

fn is_reserved_keyword(s: &str) -> bool {
    matches!(
        s,
        "mod"
            | "fn"
            | "let"
            | "mut"
            | "const"
            | "static"
            | "type"
            | "use"
            | "as"
            | "async"
            | "await"
            | "break"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "loop"
            | "match"
            | "move"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "unsafe"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}
