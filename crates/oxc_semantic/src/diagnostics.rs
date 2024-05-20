use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_span::Span;

pub fn redeclaration(x0: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Identifier `{x0}` has already been declared")).with_labels([
        LabeledSpan::new_with_span(Some(format!("`{x0}` has already been declared here")), span1),
        LabeledSpan::new_with_span(Some("It can not be redeclared here".into()), span2),
    ])
}
