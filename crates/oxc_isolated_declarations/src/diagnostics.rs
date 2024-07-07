use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

pub fn function_must_have_explicit_return_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9007: Function must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn method_must_have_explicit_return_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9008: Method must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn accessor_must_have_explicit_return_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9009: At least one accessor must have an explicit return type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn variable_must_have_explicit_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9010: Variable must have an explicit type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn parameter_must_have_explicit_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9011: Parameter must have an explicit type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn property_must_have_explicit_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9012: Property must have an explicit type annotation with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn inferred_type_of_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS9013: Expression type can't be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn signature_computed_property_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS9014: Computed properties must be number or string literals, variables or dotted expressions with --isolatedDeclarations.")
        .with_label(span)
}

pub fn object_with_spread_assignments(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9015: Objects that contain spread assignments can't be inferred with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn shorthand_property(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9016: Objects that contain shorthand properties can't be inferred with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn array_inferred(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS9017: Only const arrays can be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn arrays_with_spread_elements(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9018: Arrays with spread elements can't inferred with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn binding_element_export(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9019: Binding elements can't be exported directly with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn enum_member_initializers(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS9020: Enum member initializers must be computable without references to external symbols with --isolatedDeclarations.")
        .with_label(span)
}

pub fn extends_clause_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9021: Extends clause can't contain an expression with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn inferred_type_of_class_expression(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9022: Inference from class expressions is not supported with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn implicitly_adding_undefined_to_type(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9025: Declaration emit for this parameter requires implicitly adding undefined to it's type. This is not supported with --isolatedDeclarations.",
    )
    .with_label(span)
}

pub fn function_with_assigning_properties(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(
        "TS9023: Assigning properties to functions without declaring them is not supported with --isolatedDeclarations. Add an explicit declaration for the properties assigned to this function.",
    )
    .with_label(span)
}

// TS9026: Declaration emit for this file requires preserving this import for augmentations. This is not supported with --isolatedDeclarations.
// This error requires cross-file checking, which we cannot support.

pub fn default_export_inferred(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS9037: Default exports can't be inferred with --isolatedDeclarations.")
        .with_label(span)
}

pub fn computed_property_name(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("TS9038: Computed property names on class or object literals cannot be inferred with --isolatedDeclarations.")
        .with_label(span)
}

#[allow(clippy::needless_pass_by_value)]
pub fn type_containing_private_name(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(format!(
        "TS9039: Type containing private name '{name}' can't be used with --isolatedDeclarations."
    ))
    .with_label(span)
}
