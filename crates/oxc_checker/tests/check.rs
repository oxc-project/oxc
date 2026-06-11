//! Integration test: check the `basic` fixture project and snapshot its
//! diagnostics in a compact, deterministic format.

use std::fmt::Write;
use std::path::Path;

use oxc_checker::check_project;

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.matches('\n').count() + 1;
    let col = prefix.rfind('\n').map_or(offset + 1, |nl| offset - nl);
    (line, col)
}

#[test]
fn basic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/basic");
    let result = check_project(&root).unwrap();

    let mut out = String::new();
    for file in &result.files {
        let rel = file
            .path
            .strip_prefix(&root)
            .unwrap_or(&file.path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        for diagnostic in &file.diagnostics {
            let offset =
                diagnostic.labels.first().map_or(0usize, |l| l.offset().try_into().unwrap());
            let (line, col) = line_col(&file.source_text, offset);
            writeln!(
                out,
                "{rel}:{line}:{col} {code} {message}",
                code = diagnostic.code,
                message = diagnostic.message
            )
            .unwrap();
        }
    }
    insta::assert_snapshot!(out);
}
