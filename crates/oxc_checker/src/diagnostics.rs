//! Diagnostic constructors, tsc-compatible codes and wording.

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    check::{TypeView, print::type_to_string},
    ir::TypeId,
};

fn ts_error(code: &'static str, message: String, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(message).with_error_code("ts", code).with_label(span)
}

/// TS2322 at a declaration/return site.
pub fn not_assignable(
    view: &TypeView<'_>,
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2322",
        format!(
            "Type '{}' is not assignable to type '{}'.",
            type_to_string(view, source),
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS2345 at a call argument.
pub fn argument_not_assignable(
    view: &TypeView<'_>,
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2345",
        format!(
            "Argument of type '{}' is not assignable to parameter of type '{}'.",
            type_to_string(view, source),
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS1360 for `satisfies`.
pub fn not_satisfies(
    view: &TypeView<'_>,
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "1360",
        format!(
            "Type '{}' does not satisfy the expected type '{}'.",
            type_to_string(view, source),
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS2741: exactly one required property missing.
pub fn missing_property(
    view: &TypeView<'_>,
    prop: &str,
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2741",
        format!(
            "Property '{prop}' is missing in type '{}' but required in type '{}'.",
            type_to_string(view, source),
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS2739 (2–5 missing) / TS2740 (6+ missing).
pub fn missing_properties(
    view: &TypeView<'_>,
    props: &[Box<str>],
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    let source = type_to_string(view, source);
    let target = type_to_string(view, target);
    if props.len() > 5 {
        let listed = props[..4].join(", ");
        ts_error(
            "2740",
            format!(
                "Type '{source}' is missing the following properties from type '{target}': {listed}, and {} more.",
                props.len() - 4
            ),
            span,
        )
    } else {
        let listed = props.join(", ");
        ts_error(
            "2739",
            format!(
                "Type '{source}' is missing the following properties from type '{target}': {listed}"
            ),
            span,
        )
    }
}

/// TS2353: excess property in a fresh object literal.
pub fn excess_property(
    view: &TypeView<'_>,
    prop: &str,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2353",
        format!(
            "Object literal may only specify known properties, and '{prop}' does not exist in type '{}'.",
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS4104: readonly array/tuple assigned to a mutable one.
pub fn readonly_to_mutable(
    view: &TypeView<'_>,
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "4104",
        format!(
            "The type '{}' is 'readonly' and cannot be assigned to the mutable type '{}'.",
            type_to_string(view, source),
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS2339: property does not exist (the type string is caller-rendered to
/// support `typeof import("...")` receivers).
pub fn property_does_not_exist(prop: &str, type_string: &str, span: Span) -> OxcDiagnostic {
    ts_error("2339", format!("Property '{prop}' does not exist on type '{type_string}'."), span)
}

/// TS2362: non-numeric left operand of an arithmetic operation.
pub fn arithmetic_left(span: Span) -> OxcDiagnostic {
    ts_error(
        "2362",
        "The left-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type."
            .to_string(),
        span,
    )
}

/// TS2363: non-numeric right operand of an arithmetic operation.
pub fn arithmetic_right(span: Span) -> OxcDiagnostic {
    ts_error(
        "2363",
        "The right-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type."
            .to_string(),
        span,
    )
}

/// TS2314: wrong number of type arguments.
pub fn wrong_type_arity(display: &str, required: usize, span: Span) -> OxcDiagnostic {
    ts_error(
        "2314",
        format!("Generic type '{display}' requires {required} type argument(s)."),
        span,
    )
}

/// TS2344: type argument does not satisfy its constraint.
pub fn constraint_not_satisfied(
    view: &TypeView<'_>,
    arg: TypeId,
    constraint: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2344",
        format!(
            "Type '{}' does not satisfy the constraint '{}'.",
            type_to_string(view, arg),
            type_to_string(view, constraint)
        ),
        span,
    )
}

/// TS2352: assertion between non-overlapping types.
pub fn no_overlap_conversion(
    view: &TypeView<'_>,
    source: TypeId,
    target: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2352",
        format!(
            "Conversion of type '{}' to type '{}' may be a mistake because neither type sufficiently overlaps with the other. If this was intentional, convert the expression to 'unknown' first.",
            type_to_string(view, source),
            type_to_string(view, target)
        ),
        span,
    )
}

/// TS2420: class does not implement its interface.
pub fn incorrectly_implements(
    view: &TypeView<'_>,
    class_name: &str,
    iface: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2420",
        format!(
            "Class '{class_name}' incorrectly implements interface '{}'.",
            type_to_string(view, iface)
        ),
        span,
    )
}

/// TS2416: class member incompatible with the implemented/extended base.
pub fn property_not_assignable_to_base(
    view: &TypeView<'_>,
    prop: &str,
    class_name: &str,
    iface: TypeId,
    span: Span,
) -> OxcDiagnostic {
    ts_error(
        "2416",
        format!(
            "Property '{prop}' in type '{class_name}' is not assignable to the same property in base type '{}'.",
            type_to_string(view, iface)
        ),
        span,
    )
}

/// TS2694: missing type member on a namespace import.
pub fn namespace_no_member(module: &str, name: &str, span: Span) -> OxcDiagnostic {
    ts_error("2694", format!("Namespace '\"{module}\"' has no exported member '{name}'."), span)
}

/// TS2366: some paths return a value, but the end of the function is
/// reachable and the return type excludes `undefined`.
pub fn lacks_ending_return(span: Span) -> OxcDiagnostic {
    ts_error(
        "2366",
        "Function lacks ending return statement and return type does not include 'undefined'."
            .to_string(),
        span,
    )
}

/// TS2355: annotated non-void function with no `return` statements.
pub fn must_return_value(span: Span) -> OxcDiagnostic {
    ts_error(
        "2355",
        "A function whose declared type is neither 'undefined', 'void', nor 'any' must return a value."
            .to_string(),
        span,
    )
}

/// TS2613: default import where the module has a same-named named export.
pub fn no_default_export_hint(module: &str, name: &str, span: Span) -> OxcDiagnostic {
    ts_error(
        "2613",
        format!(
            "Module '\"{module}\"' has no default export. Did you mean to use 'import {{ {name} }} from \"{module}\"' instead?"
        ),
        span,
    )
}

/// TS1192 with the resolved module path (tsc prints the resolved file, not
/// the specifier).
pub fn no_default_export_resolved(module: &str, span: Span) -> OxcDiagnostic {
    ts_error("1192", format!("Module '\"{module}\"' has no default export."), span)
}

/// TS2307 for unresolved module specifiers.
pub fn module_not_found(specifier: &str, span: Span) -> OxcDiagnostic {
    ts_error(
        "2307",
        format!("Cannot find module '{specifier}' or its corresponding type declarations."),
        span,
    )
}

/// TS2305 for a missing named export.
pub fn no_exported_member(specifier: &str, name: &str, span: Span) -> OxcDiagnostic {
    ts_error("2305", format!("Module '\"{specifier}\"' has no exported member '{name}'."), span)
}
