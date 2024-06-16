use oxc_ast::ast::Function;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, Span};

pub fn function_must_have_explicit_return_type(func: &Function<'_>) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Function must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(func.id.as_ref().map_or_else(
        || {
            let start = func.params.span.start;
            Span::new(start, start)
        },
        |id| id.span,
    ))
}

pub fn type_containing_private_name(name: &Atom<'_>, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "Type containing private name '{name}' can't be used with --isolatedDeclarations."
    ))
    .with_label(span)
}

pub fn computed_property_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Computed property names on class or object literals cannot be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn signature_computed_property_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Computed properties must be number or string literals, variables or dotted expressions with --isolatedDeclarations.")
        .with_label(span)
}
