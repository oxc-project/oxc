use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use oxc_span::SourceType;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut f = File::create(&dest_path).unwrap();

    let fixtures_dir = Path::new("tests/fixtures");

    if !fixtures_dir.exists() {
        // If no fixtures directory exists, create an empty file
        writeln!(f, "// No test fixtures found").unwrap();
        return;
    }

    writeln!(f, "// Auto-generated test modules and functions").unwrap();
    writeln!(f).unwrap();

    // Collect all test files organized by directory
    let mut dir_structure = DirStructure::new();
    collect_tests(fixtures_dir, fixtures_dir, &mut dir_structure).unwrap();

    // Generate nested modules
    generate_modules(&mut f, &dir_structure, 0).unwrap();

    println!("cargo:rerun-if-changed=tests/fixtures");
}

#[derive(Default)]
struct DirStructure {
    /// Test files in this directory (relative paths from fixtures root)
    test_files: Vec<PathBuf>,
    /// Subdirectories
    subdirs: BTreeMap<String, DirStructure>,
}

impl DirStructure {
    fn new() -> Self {
        Self::default()
    }
}

/// Collect all test files and organize them by directory
fn collect_tests(dir: &Path, base_dir: &Path, structure: &mut DirStructure) -> std::io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            let subdir = structure.subdirs.entry(dir_name).or_default();
            collect_tests(&path, base_dir, subdir)?;
        } else if is_test_file(&path) {
            let relative_path = path.strip_prefix(base_dir).unwrap().to_path_buf();
            structure.test_files.push(relative_path);
        }
    }

    Ok(())
}

/// Generate nested modules for the directory structure
fn generate_modules(
    f: &mut File,
    structure: &DirStructure,
    indent_level: usize,
) -> std::io::Result<()> {
    let indent = "    ".repeat(indent_level);

    // Generate test functions for files in this directory
    for test_file in &structure.test_files {
        generate_test_function(f, test_file, indent_level)?;
    }

    // Generate submodules
    for (dir_name, subdir) in &structure.subdirs {
        let module_name = sanitize_module_name(dir_name);

        writeln!(f, "{indent}#[cfg(test)]")?;
        writeln!(f, "{indent}mod {module_name} {{")?;
        writeln!(f, "{indent}    use super::test_file;")?;
        writeln!(f)?;

        generate_modules(f, subdir, indent_level + 1)?;

        writeln!(f, "{indent}}}")?;
        writeln!(f)?;
    }

    Ok(())
}

fn is_test_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        SourceType::from_extension(ext).is_ok()
    } else {
        false
    }
}

fn generate_test_function(
    f: &mut File,
    relative_path: &Path,
    indent_level: usize,
) -> std::io::Result<()> {
    let indent = "    ".repeat(indent_level);
    // Use only the filename for the test name (directories are handled by modules)
    let test_name = file_to_test_name(relative_path.file_name().unwrap().to_str().unwrap());

    writeln!(f, "{indent}#[test]")?;
    writeln!(f, "{indent}fn {test_name}() {{")?;
    writeln!(
        f,
        "{}    let path = std::path::Path::new(\"tests/fixtures/{}\");",
        indent,
        relative_path.display()
    )?;
    writeln!(f, "{indent}    test_file(path);")?;
    writeln!(f, "{indent}}}")?;
    writeln!(f)?;

    Ok(())
}

/// Convert filename to a valid Rust test function name
fn file_to_test_name(filename: &str) -> String {
    let mut name = String::new();

    // Replace non-alphanumeric characters with underscores
    for c in filename.chars() {
        if c.is_alphanumeric() {
            name.push(c.to_ascii_lowercase());
        } else {
            name.push('_');
        }
    }

    // Remove file extension
    if let Some(pos) = name.rfind('_') {
        let after_underscore = &name[pos + 1..];
        if SourceType::from_extension(after_underscore).is_ok() {
            name.truncate(pos);
        }
    }

    sanitize_identifier(name, "test")
}

/// Sanitize directory name to be a valid Rust module name
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

/// Sanitize a string to be a valid Rust identifier
/// - prefix: prefix to add if identifier is empty or starts with digit (e.g., "test" or "")
fn sanitize_identifier(mut name: String, prefix: &str) -> String {
    // Ensure it starts with a letter or underscore
    if name.is_empty() || name.chars().next().unwrap().is_numeric() {
        name = if prefix.is_empty() { format!("_{name}") } else { format!("{prefix}_{name}") };
    }

    // Handle reserved keywords
    if is_reserved_keyword(&name) {
        return format!("r#{name}");
    }

    name
}

/// Check if a string is a Rust reserved keyword
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
