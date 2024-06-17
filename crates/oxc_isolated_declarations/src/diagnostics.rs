use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Atom, Span};

pub fn method_must_have_explicit_return_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Method must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn function_must_have_explicit_return_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Function must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn accessor_must_have_explicit_return_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "At least one accessor must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn property_must_have_explicit_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Property must have an explicit type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
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

pub fn enum_member_initializers(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Enum member initializers must be computable without references to external symbols with --isolatedDeclarations.")
        .with_label(span)
}

pub fn extends_clause_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Extends clause can't contain an expression with --isolatedDeclarations.")
        .with_label(span)
}

pub fn default_export_inferred(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Default exports can't be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn array_inferred(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Arrays can't be inferred with --isolatedDeclarations.").with_label(span)
}

pub fn shorthand_property(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Objects that contain shorthand properties can't be inferred with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn inferred_type_of_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Expression type can't be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn inferred_type_of_class_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Class expression type can't be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn parameter_must_have_explicit_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Parameter must have an explicit type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn implicitly_adding_undefined_to_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "Declaration emit for this parameter requires implicitly adding undefined to it's type. This is not supported with --isolatedDeclarations.",
    )
    .with_label(span)
}
