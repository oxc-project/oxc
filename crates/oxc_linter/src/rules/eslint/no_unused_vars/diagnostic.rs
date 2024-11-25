use std::fmt;

use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::SymbolFlags;
use oxc_span::{GetSpan, Span};

use super::{options::IgnorePattern, Symbol};

fn pronoun_for_symbol(
    symbol_flags: SymbolFlags,
) -> (/* singular */ &'static str, /* plural */ &'static str) {
    if symbol_flags.is_function() {
        ("Function", "functions")
    } else if symbol_flags.is_class() {
        ("Class", "classes")
    } else if symbol_flags.is_interface() {
        ("Interface", "interfaces")
    } else if symbol_flags.is_type_alias() {
        ("Type alias", "type aliases")
    } else if symbol_flags.is_enum() {
        ("Enum", "enums")
    } else if symbol_flags.is_enum_member() {
        ("Enum member", "enum members")
    } else if symbol_flags.is_type_import() {
        ("Type", "types")
    } else if symbol_flags.is_import() {
        ("Identifier", "identifiers")
    } else if symbol_flags.is_catch_variable() {
        ("Catch parameter", "caught errors")
    } else {
        ("Variable", "variables")
    }
}

pub fn used_ignored<R>(symbol: &Symbol<'_, '_>, pat: &IgnorePattern<R>) -> OxcDiagnostic
where
    R: fmt::Display,
{
    let (pronoun_singular, _) = pronoun_for_symbol(symbol.flags());
    let name = symbol.name();

    let help_suffix = match pat {
        IgnorePattern::None => ".".into(),
        IgnorePattern::Default => {
            name.strip_prefix('_').map_or(".".into(), |name| format!(" to '{name}'."))
        }
        IgnorePattern::Some(ref r) => {
            format!(" to match the pattern /{r}/.")
        }
    };

    OxcDiagnostic::warn(format!("{pronoun_singular} '{name}' is marked as ignored but is used."))
        .with_label(symbol.span().label(format!("'{name}' is declared here")))
        .with_help(format!(
            "Consider renaming this {}{help_suffix}",
            pronoun_singular.cow_to_lowercase()
        ))
}
/// Variable 'x' is declared but never used.
pub fn declared<R>(symbol: &Symbol<'_, '_>, pat: &IgnorePattern<R>) -> OxcDiagnostic
where
    R: fmt::Display,
{
    let (verb, help) = if symbol.flags().is_catch_variable() {
        ("caught", "Consider handling this error.")
    } else {
        ("declared", "Consider removing this declaration.")
    };
    let name = symbol.name();
    let (pronoun, pronoun_plural) = pronoun_for_symbol(symbol.flags());
    let suffix = pat.diagnostic_help(pronoun_plural);

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is {verb} but never used.{suffix}"))
        .with_label(symbol.span().label(format!("'{name}' is declared here")))
        .with_help(help)
}

/// Variable 'x' is assigned a value but never used.
pub fn assign<R>(
    symbol: &Symbol<'_, '_>,
    assign_span: Span,
    pat: &IgnorePattern<R>,
) -> OxcDiagnostic
where
    R: fmt::Display,
{
    let name = symbol.name();
    let (pronoun, pronoun_plural) = pronoun_for_symbol(symbol.flags());
    let suffix = pat.diagnostic_help(pronoun_plural);

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is assigned a value but never used.{suffix}"))
        .with_labels([
            symbol.span().label(format!("'{name}' is declared here")),
            assign_span.label("it was last assigned here"),
        ])
        .with_help("Did you mean to use this variable?")
}

/// Parameter 'x' is declared but never used.
pub fn param<R>(symbol: &Symbol<'_, '_>, pat: &IgnorePattern<R>) -> OxcDiagnostic
where
    R: fmt::Display,
{
    let name = symbol.name();
    let suffix = pat.diagnostic_help("parameters");

    OxcDiagnostic::warn(format!("Parameter '{name}' is declared but never used.{suffix}"))
        .with_label(symbol.span().label(format!("'{name}' is declared here")))
        .with_help("Consider removing this parameter.")
}

/// Identifier 'x' imported but never used.
pub fn imported(symbol: &Symbol<'_, '_>) -> OxcDiagnostic {
    let (pronoun, _) = pronoun_for_symbol(symbol.flags());
    let name = symbol.name();

    OxcDiagnostic::warn(format!("{pronoun} '{name}' is imported but never used."))
        .with_label(symbol.span().label(format!("'{name}' is imported here")))
        .with_help("Consider removing this import.")
}
