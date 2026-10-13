use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::static_ident;

use crate::{AstNode, context::LintContext, globals::GLOBAL_OBJECT_NAMES, rule::Rule};

fn prefer_global_number_constants_diagnostic(
    span: Span,
    property: &str,
    replacement: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{replacement}` over `Number.{property}`."))
        .with_help(format!("Replace `Number.{property}` with `{replacement}`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferGlobalNumberConstants;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers the global constants `NaN`, `Infinity` and `-Infinity` over the
    /// equivalent `Number` static properties `Number.NaN`,
    /// `Number.POSITIVE_INFINITY` and `Number.NEGATIVE_INFINITY`.
    ///
    /// ### Why is this bad?
    ///
    /// The global constants are shorter and easier to read than their
    /// `Number` static property counterparts.
    ///
    /// `Number.NaN` and `Number.POSITIVE_INFINITY` are auto-fixed when the
    /// replacement global is not shadowed. `Number.NEGATIVE_INFINITY` is
    /// reported without a fix, because inserting a leading `-` can change how
    /// the surrounding expression parses.
    ///
    /// Note: this rule enforces the opposite of the `checkNaN` and
    /// `checkInfinity` options of `unicorn/prefer-number-properties`.
    /// Enable only one of them.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = Number.NaN;
    /// const bar = Number.POSITIVE_INFINITY;
    /// const baz = Number.NEGATIVE_INFINITY;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = NaN;
    /// const bar = Infinity;
    /// const baz = -Infinity;
    /// ```
    PreferGlobalNumberConstants,
    unicorn,
    style,
    conditional_fix,
    version = "next",
    short_description = "Prefer global numeric constants over `Number` static properties.",
);

impl Rule for PreferGlobalNumberConstants {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member_expr) = node.kind().as_member_expression_kind() else {
            return;
        };

        // `Number.NaN`, `Number["NaN"]`, but not `Number.#foo`.
        let Some(property) = member_expr.static_property_name() else {
            return;
        };
        let (property, replacement) = match property.as_str() {
            "NaN" => ("NaN", "NaN"),
            "POSITIVE_INFINITY" => ("POSITIVE_INFINITY", "Infinity"),
            "NEGATIVE_INFINITY" => ("NEGATIVE_INFINITY", "-Infinity"),
            _ => return,
        };

        if !is_global_number(member_expr.object(), ctx) {
            return;
        }

        // `Number.NaN = 1`, `Number.NaN ||= 1`, `[Number.NaN] = []`, ...
        if member_expr.is_assigned_to_in_parent(&ctx.nodes().parent_kind(node.id())) {
            return;
        }

        // `const NaN = 1; Number.NaN` — replacing would change meaning.
        let global_name =
            if property == "NaN" { static_ident!("NaN") } else { static_ident!("Infinity") };
        if ctx.scoping().find_binding(node.scope_id(), global_name).is_some() {
            return;
        }

        let span = member_expr.span();
        let diagnostic = prefer_global_number_constants_diagnostic(span, property, replacement);

        // No fix for `-Infinity` (see rule docs), and don't drop comments
        // like `Number /* why */ .NaN`.
        if property == "NEGATIVE_INFINITY" || ctx.has_comments_between(span) {
            ctx.diagnostic(diagnostic);
        } else {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(span, replacement));
        }
    }
}

/// Is `expr` a reference to the global `Number` constructor?
///
/// Matches a bare `Number` that resolves to the global, or `window.Number`,
/// `globalThis.Number`, etc. where the global object itself is unshadowed.
fn is_global_number(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            ident.name == "Number" && ctx.is_reference_to_global_variable(ident)
        }
        expr => {
            let Some(member_expr) = expr.as_member_expression() else {
                return false;
            };
            let Expression::Identifier(object) = member_expr.object() else {
                return false;
            };
            GLOBAL_OBJECT_NAMES.contains(&object.name.as_str())
                && ctx.is_reference_to_global_variable(object)
                && member_expr.static_property_name().is_some_and(|name| name == "Number")
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = NaN;",
        "const foo = Infinity;",
        "const foo = -Infinity;",
        "const foo = Number.MAX_SAFE_INTEGER;",
        "const foo = object.Number.NaN;",
        "Number.NaN = 1;",
        "Number.POSITIVE_INFINITY ||= 1;",
        "[Number.NEGATIVE_INFINITY] = [];",
        "const Number = {
                NaN: 1,
                POSITIVE_INFINITY: 2,
                NEGATIVE_INFINITY: -2,
            };
            const foo = Number.NaN + Number.POSITIVE_INFINITY + Number.NEGATIVE_INFINITY;",
        "function foo() {
                const NaN = 1;
                const value = Number.NaN;
            }",
        "function foo() {
                const Infinity = 1;
                const positive = Number.POSITIVE_INFINITY;
                const negative = Number.NEGATIVE_INFINITY;
            }",
        "const NaN = 1; const value = Number.NaN;", // {"sourceType": "script"},
        "const Infinity = 1; const positive = Number.POSITIVE_INFINITY; const negative = Number.NEGATIVE_INFINITY;", // {"sourceType": "script"}
    ];

    let fail = vec![
        "const foo = Number.NaN;",
        "const foo = window.Number.NaN;",
        r#"const foo = Number["NaN"];"#,
        "const foo = Number.POSITIVE_INFINITY;",
        "const foo = Number.NEGATIVE_INFINITY;",
        "const foo = Number.NEGATIVE_INFINITY.toString();",
        "const foo = {value: Number.NaN};",
        "const foo = {[Number.POSITIVE_INFINITY]: Number.NEGATIVE_INFINITY};",
        "const foo = Number /* comment */ .NaN;",
    ];

    let fix = vec![
        ("const foo = Number.NaN;", "const foo = NaN;"),
        ("const foo = window.Number.NaN;", "const foo = NaN;"),
        (r#"const foo = Number["NaN"];"#, "const foo = NaN;"),
        ("const foo = Number.POSITIVE_INFINITY;", "const foo = Infinity;"),
        ("const foo = {value: Number.NaN};", "const foo = {value: NaN};"),
        (
            "const foo = {[Number.POSITIVE_INFINITY]: Number.NEGATIVE_INFINITY};",
            "const foo = {[Infinity]: Number.NEGATIVE_INFINITY};",
        ),
        // No fix: `-Infinity` and comments inside the expression.
        ("const foo = Number.NEGATIVE_INFINITY;", "const foo = Number.NEGATIVE_INFINITY;"),
        ("const foo = Number /* comment */ .NaN;", "const foo = Number /* comment */ .NaN;"),
    ];

    Tester::new(PreferGlobalNumberConstants::NAME, PreferGlobalNumberConstants::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
