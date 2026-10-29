use std::{
    io::{self, Write},
    path::Path,
};

use oxc_span::{ModuleKind, SourceType};
use serde_json::json;

/// Write a conformance input for training the published Node bindings.
///
/// # Errors
/// Returns an error if the record cannot be written.
pub fn write_pgo_case(
    writer: &mut impl Write,
    suite: &str,
    path: &Path,
    code: &str,
    source_type: SourceType,
    expected_failure: bool,
) -> io::Result<()> {
    let lang = if source_type.is_typescript_definition() {
        "dts"
    } else {
        match (source_type.is_typescript(), source_type.is_jsx()) {
            (true, true) => "tsx",
            (true, false) => "ts",
            (false, true) => "jsx",
            (false, false) => "js",
        }
    };
    let source_type = match source_type.module_kind() {
        ModuleKind::Module => "module",
        ModuleKind::Script => "script",
        ModuleKind::CommonJS => "commonjs",
        ModuleKind::Unambiguous => "unambiguous",
    };
    serde_json::to_writer(
        &mut *writer,
        &json!({
            "suite": suite,
            "filename": crate::normalize_path(path),
            "code": code,
            "options": { "lang": lang, "sourceType": source_type },
            "expectedFailure": expected_failure,
        }),
    )?;
    writer.write_all(b"\n")
}
