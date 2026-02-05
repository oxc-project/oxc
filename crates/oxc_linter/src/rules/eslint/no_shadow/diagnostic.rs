use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

pub fn no_shadow(span: Span, name: &str, shadowed_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already declared in the upper scope."))
        .with_help(format!(
            "Consider renaming '{name}' to avoid shadowing the variable from the outer scope."
        ))
        .with_labels([
            span.label(format!("'{name}' is declared here")),
            shadowed_span.label("shadowed declaration is here"),
        ])
}
