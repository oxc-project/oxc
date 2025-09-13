use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

#[cold]
pub fn invalid_input(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Template literal should be wrapped with ` or escaped properly".to_string(),
    )
    .with_label(span)
}

#[cold]
pub fn template_substitution(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Template literal should not contain unescaped `${}`".to_string())
        .with_label(span)
}

#[cold]
pub fn too_large_unicode_escape_sequence(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Too large unicode escape sequence".to_string()).with_label(span)
}

#[cold]
pub fn invalid_hex_escape(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid hex escape sequence".to_string()).with_label(span)
}

#[cold]
pub fn invalid_unicode_escape(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Invalid unicode escape sequence".to_string()).with_label(span)
}
