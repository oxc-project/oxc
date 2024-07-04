use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

pub fn redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Identifier `{x0}` has already been declared")).with_labels([
        span1.label(format!("`{x0}` has already been declared here")),
        span2.label("It can not be redeclared here"),
    ])
}
