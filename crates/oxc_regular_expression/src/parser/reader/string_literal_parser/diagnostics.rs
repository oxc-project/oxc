use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

#[cold]
pub fn invalid_input(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "String literal should be wrapped with ' or \", or escaped properly".to_string(),
    )
    .with_label(span)
}

#[cold]
pub fn legacy_in_strict_mode(kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Not allowed {kind} in strict mode")).with_label(span)
}

#[cold]
pub fn too_large_unicode_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Too large unicode escape sequence".to_string()).with_label(span)
}
