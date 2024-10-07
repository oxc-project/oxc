use std::{path::Path, sync::Arc};

use oxc::{
    diagnostics::{Error, NamedSource, OxcDiagnostic},
    span::SourceType,
};

pub fn wrap_diagnostics(
    filename: &Path,
    source_type: SourceType,
    source_text: &str,
    errors: Vec<OxcDiagnostic>,
) -> Vec<String> {
    if errors.is_empty() {
        return vec![];
    }
    let source = {
        let lang = match (source_type.is_javascript(), source_type.is_jsx()) {
            (true, false) => "JavaScript",
            (true, true) => "JSX",
            (false, true) => "TypeScript React",
            (false, false) => {
                if source_type.is_typescript_definition() {
                    "TypeScript Declaration"
                } else {
                    "TypeScript"
                }
            }
        };

        let ns = NamedSource::new(filename.to_string_lossy(), source_text.to_string())
            .with_language(lang);
        Arc::new(ns)
    };

    errors
        .into_iter()
        .map(move |diagnostic| Error::from(diagnostic).with_source_code(Arc::clone(&source)))
        .map(|error| format!("{error:?}"))
        .collect()
}
