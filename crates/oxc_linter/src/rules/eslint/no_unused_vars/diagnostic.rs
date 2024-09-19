use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::SymbolFlags;
use oxc_span::{GetSpan, Span};

use super::Symbol;

fn pronoun_for_symbol(symbol_flags: SymbolFlags) -> &'static str {
    if symbol_flags.is_function() {
        "Function"
    } else if symbol_flags.is_class() {
        "Class"
    } else if symbol_flags.is_interface() {
        "Interface"
    } else if symbol_flags.is_type_alias() {
        "Type alias"
    } else if symbol_flags.is_enum() {
        "Enum"
    } else if symbol_flags.is_enum_member() {
        "Enum member"
    } else if symbol_flags.is_type_import() {
        "Type"
    } else if symbol_flags.is_import() {
        "Identifier"
    } else {
        "Variable"
    }
}

pub fn used_ignored(symbol: &Symbol<'_, '_>) -> OxcDiagnostic {
    let pronoun = pronoun_for_symbol(symbol.flags());
    let name = symbol.name();

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is marked as ignored but is used."))
        .with_label(symbol.span().label(format!("'{name}' is declared here")))
        .with_help(format!("Consider renaming this {}.", pronoun.cow_to_lowercase()))
}
/// Variable 'x' is declared but never used.
pub fn declared(symbol: &Symbol<'_, '_>) -> OxcDiagnostic {
    let (verb, help) = if symbol.flags().is_catch_variable() {
        ("caught", "Consider handling this error.")
    } else {
        ("declared", "Consider removing this declaration.")
    };
    let pronoun = pronoun_for_symbol(symbol.flags());
    let name = symbol.name();

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is {verb} but never used."))
        .with_label(symbol.span().label(format!("'{name}' is declared here")))
        .with_help(help)
}

/// Variable 'x' is assigned a value but never used.
pub fn assign(symbol: &Symbol<'_, '_>, assign_span: Span) -> OxcDiagnostic {
    let pronoun = pronoun_for_symbol(symbol.flags());
    let name = symbol.name();

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is assigned a value but never used."))
        .with_labels([
            symbol.span().label(format!("'{name}' is declared here")),
            assign_span.label("it was last assigned here"),
        ])
        .with_help("Did you mean to use this variable?")
}

/// Parameter 'x' is declared but never used.
pub fn param(symbol: &Symbol<'_, '_>) -> OxcDiagnostic {
    let name = symbol.name();

    OxcDiagnostic::warn(format!("Parameter '{name}' is declared but never used."))
        .with_label(symbol.span().label(format!("'{name}' is declared here")))
        .with_help("Consider removing this parameter.")
}

/// Identifier 'x' imported but never used.
pub fn imported(symbol: &Symbol<'_, '_>) -> OxcDiagnostic {
    let pronoun = pronoun_for_symbol(symbol.flags());
    let name = symbol.name();

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is imported but never used."))
        .with_label(symbol.span().label(format!("'{name}' is imported here")))
        .with_help("Consider removing this import.")
}
