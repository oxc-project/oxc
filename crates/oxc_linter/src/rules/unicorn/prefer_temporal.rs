use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierReference, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, iter_outer_expressions, variable_declaration_kind},
    context::LintContext,
    fixer::RuleFixer,
    globals::GLOBAL_OBJECT_NAMES,
    rule::{DefaultRuleConfig, Rule},
    utils::static_string_value,
};

/// The largest absolute time value a `Date` can hold.
const DATE_MAX_TIME: f64 = 8_640_000_000_000_000.0;

fn prefer_temporal_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Temporal` over `{description}`"))
        .with_help("Use the `Temporal` API to hold a date or a time.")
        .with_label(span)
}

fn prefer_temporal_parse_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Temporal` over `{description}`"))
        .with_help(
            "Date string parsing is inconsistent across implementations. Use `Temporal.PlainDate.from()` instead.",
        )
        .with_label(span)
}

fn prefer_temporal_month_diagnostic(span: Span, description: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Temporal` over `{description}`"))
        .with_help(
            "The month argument of `Date` is zero-indexed, which is error-prone. Use `Temporal.PlainDate.from({year, month, day})` instead.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferTemporal {
    /// Whether to also flag `Date.now()`.
    check_date_now: bool,
    /// Whether to also flag bare references to `Date`, such as `x instanceof Date`.
    check_references: bool,
    /// Whether to also flag methods called on `Date` instances, such as `date.getFullYear()`.
    /// This option needs type information, which oxlint does not have, so it does nothing.
    check_methods: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports the `Date` API and asks for the `Temporal` API instead.
    ///
    /// ### Why is this bad?
    ///
    /// `Temporal` is the modern replacement for `Date`. `Date` has a zero-indexed month, it is
    /// mutable, it cannot hold a time zone or a calendar, and it parses date strings differently
    /// on each platform. `Temporal` reached stage 4, and ships in Node.js 26 and in recent
    /// browsers. A polyfill is available as `@js-temporal/polyfill`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const now = new Date();
    /// const date = new Date('2024-08-16');
    /// const christmas = new Date(2000, 11, 25);
    /// const timestamp = Date.parse('2024-08-16');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const now = Temporal.Now.instant();
    /// const date = Temporal.PlainDate.from('2024-08-16');
    /// const christmas = Temporal.PlainDate.from({year: 2000, month: 12, day: 25});
    /// const timestamp = Temporal.PlainDate.from('2024-08-16');
    /// ```
    ///
    /// ### Options
    ///
    /// #### checkDateNow
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Whether to also flag `Date.now()`. `Temporal.Now.instant().epochMilliseconds` returns the
    /// same number, so this report carries an automatic fix.
    ///
    /// #### checkReferences
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// Whether to also flag bare references to `Date`, such as `x instanceof Date`.
    ///
    /// #### checkMethods
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// This option needs type information to find calls on `Date` instances. oxlint does not have
    /// type information here, so the option is accepted and then ignored. It exists to keep an
    /// ESLint configuration valid after a migration.
    PreferTemporal,
    unicorn,
    restriction,
    conditional_fix_suggestion,
    config = PreferTemporal,
    version = "next",
    short_description = "Prefer `Temporal` over `Date`.",
);

impl Rule for PreferTemporal {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NewExpression(new_expr) => check_new_expression(new_expr, ctx),
            AstKind::CallExpression(call_expr) => {
                self.check_call_expression(call_expr, ctx);
            }
            AstKind::IdentifierReference(ident) if self.check_references => {
                check_bare_reference(node, ident, ctx);
            }
            _ => {}
        }
    }
}

impl PreferTemporal {
    fn check_call_expression<'a>(&self, call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
        let span = call_expr.span;
        let callee = call_expr.callee.get_inner_expression();

        // `Date()` without `new` returns a string, not a `Date`.
        if is_global_date(callee, ctx) {
            ctx.diagnostic(prefer_temporal_diagnostic(span, "Date()"));
            return;
        }

        let Some(member_expr) = callee.as_member_expression() else {
            return;
        };
        if !is_global_date(member_expr.object(), ctx) {
            return;
        }

        match member_expr.static_property_name() {
            Some("parse") => {
                ctx.diagnostic(prefer_temporal_parse_diagnostic(span, "Date.parse(…)"));
            }
            Some("UTC") => {
                ctx.diagnostic(prefer_temporal_month_diagnostic(span, "Date.UTC(…)"));
            }
            Some("now") if self.check_date_now => {
                let diagnostic = prefer_temporal_diagnostic(span, "Date.now()");
                // `Date.now()` and `Temporal.Now.instant().epochMilliseconds` both return the
                // epoch milliseconds as a number, so the replacement is exact.
                if holds_comment(span, ctx) {
                    ctx.diagnostic(diagnostic);
                } else {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fixer.replace(span, "Temporal.Now.instant().epochMilliseconds")
                    });
                }
            }
            _ => {}
        }
    }
}

fn check_new_expression<'a>(new_expr: &NewExpression<'a>, ctx: &LintContext<'a>) {
    if !is_global_date(&new_expr.callee, ctx) {
        return;
    }

    let span = new_expr.span;

    // `new Date()` reads the current moment.
    let Some(argument) = new_expr.arguments.first() else {
        let diagnostic = prefer_temporal_diagnostic(span, "new Date()");
        let Some(fixer) = suggestion_fixer(span, ctx) else {
            ctx.diagnostic(diagnostic);
            return;
        };
        ctx.diagnostic_with_suggestions(
            diagnostic,
            [
                fixer
                    .replace(span, "Temporal.Now.instant()")
                    .with_message("Replace with `Temporal.Now.instant()`."),
                fixer
                    .replace(span, "Temporal.Now.zonedDateTimeISO()")
                    .with_message("Replace with `Temporal.Now.zonedDateTimeISO()`."),
                fixer
                    .replace(span, "Temporal.Now.plainDateTimeISO()")
                    .with_message("Replace with `Temporal.Now.plainDateTimeISO()`."),
            ],
        );
        return;
    };

    // `new Date(year, month, …)` builds a date from calendar parts.
    if new_expr.arguments.len() > 1 {
        ctx.diagnostic(prefer_temporal_month_diagnostic(span, "new Date(…)"));
        return;
    }

    // A spread argument hides both the argument count and the argument type.
    let Some(argument_expr) = argument.as_expression() else {
        ctx.diagnostic(prefer_temporal_diagnostic(span, "new Date(…)"));
        return;
    };

    // A number argument is always a count of epoch milliseconds.
    if let Some(value) = resolved_number(argument_expr, ctx) {
        if is_epoch_milliseconds(value) {
            let diagnostic = prefer_temporal_diagnostic(span, "new Date(…)");
            match suggestion_fixer(span, ctx) {
                Some(fixer) => {
                    let replacement = format!(
                        "Temporal.Instant.fromEpochMilliseconds({})",
                        ctx.source_range(argument.span())
                    );
                    let message = format!("Replace with `{replacement}`.");
                    ctx.diagnostic_with_suggestions(
                        diagnostic,
                        [fixer.replace(span, replacement).with_message(message)],
                    );
                }
                None => ctx.diagnostic(diagnostic),
            }
            return;
        }
    } else if resolved_string(argument_expr, ctx).is_some() {
        ctx.diagnostic(prefer_temporal_parse_diagnostic(span, "new Date(…)"));
        return;
    }

    ctx.diagnostic(prefer_temporal_diagnostic(span, "new Date(…)"));
}

fn check_bare_reference<'a>(
    node: &AstNode<'a>,
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) {
    if ident.name != "Date" || !ctx.is_reference_to_global_variable(ident) {
        return;
    }

    // A type annotation such as `let date: Date` is not a value use of `Date`.
    if !ctx.scoping().get_reference(ident.reference_id()).flags().is_value() {
        return;
    }

    // Construction, calls and static members are already reported on their own node.
    let parent = iter_outer_expressions(ctx.nodes(), node.id()).next();
    let reported_elsewhere = match parent {
        Some(AstKind::NewExpression(new_expr)) => {
            new_expr.callee.get_inner_expression().span() == ident.span
        }
        Some(AstKind::CallExpression(call_expr)) => {
            call_expr.callee.get_inner_expression().span() == ident.span
        }
        Some(parent) => parent.as_member_expression_kind().is_some_and(|member_expr| {
            member_expr.object().get_inner_expression().span() == ident.span
        }),
        None => false,
    };
    if reported_elsewhere {
        return;
    }

    ctx.diagnostic(prefer_temporal_diagnostic(ident.span, "Date"));
}

/// Whether `expr` is the global `Date`, either on its own or on a global object.
fn is_global_date<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let expr = expr.get_inner_expression();

    if let Expression::Identifier(ident) = expr {
        return ident.name == "Date" && ctx.is_reference_to_global_variable(ident);
    }

    let Some(member_expr) = expr.as_member_expression() else {
        return false;
    };
    let Expression::Identifier(object) = member_expr.object().get_inner_expression() else {
        return false;
    };
    GLOBAL_OBJECT_NAMES.contains(&object.name.as_str())
        && member_expr.static_property_name() == Some("Date")
}

/// Build a fixer for `span`, unless a replacement would drop a comment.
fn suggestion_fixer<'c, 'a>(span: Span, ctx: &'c LintContext<'a>) -> Option<RuleFixer<'c, 'a>> {
    if holds_comment(span, ctx) {
        return None;
    }
    Some(RuleFixer::new(FixKind::Suggestion, ctx))
}

fn holds_comment(span: Span, ctx: &LintContext) -> bool {
    ctx.comments_range(span.start..span.end).next().is_some()
}

/// Whether `value` is a whole number that `Date` can hold.
///
/// `Temporal.Instant.fromEpochMilliseconds()` throws on any other number, whereas `Date`
/// truncates it or builds an invalid date. Only suggest the replacement when both agree.
fn is_epoch_milliseconds(value: f64) -> bool {
    value.abs() <= DATE_MAX_TIME && value.fract() == 0.0
}

/// Resolve a side-effect-free number, through at most one `const` binding.
fn resolved_number<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> Option<f64> {
    static_number_value(expr).or_else(|| static_number_value(const_initializer(expr, ctx)?))
}

/// Resolve a side-effect-free string, through at most one `const` binding.
fn resolved_string<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> Option<String> {
    static_string_value(expr).or_else(|| static_string_value(const_initializer(expr, ctx)?))
}

fn static_number_value(expr: &Expression<'_>) -> Option<f64> {
    match expr.get_inner_expression() {
        Expression::NumericLiteral(literal) => Some(literal.value),
        Expression::UnaryExpression(unary) => match unary.operator {
            UnaryOperator::UnaryNegation => Some(-static_number_value(&unary.argument)?),
            UnaryOperator::UnaryPlus => static_number_value(&unary.argument),
            _ => None,
        },
        _ => None,
    }
}

/// The initializer of the `const` declaration that `expr` refers to.
fn const_initializer<'c, 'a>(
    expr: &'c Expression<'a>,
    ctx: &'c LintContext<'a>,
) -> Option<&'c Expression<'a>> {
    let Expression::Identifier(ident) = expr.get_inner_expression() else {
        return None;
    };
    let AstKind::VariableDeclarator(declarator) = get_declaration_of_variable(ident, ctx)?.kind()
    else {
        return None;
    };
    // `const {ms} = 1000` binds `undefined`, not the initializer.
    if !declarator.id.is_binding_identifier()
        || !variable_declaration_kind(declarator, ctx).is_const()
    {
        return None;
    }
    declarator.init.as_ref()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let check_date_now = || Some(serde_json::json!([{ "checkDateNow": true }]));
    let check_references = || Some(serde_json::json!([{ "checkReferences": true }]));
    let check_methods = || Some(serde_json::json!([{ "checkMethods": true }]));

    let pass = vec![
        ("Temporal.Now.instant()", None),
        (r#"Temporal.PlainDate.from("2024-08-16")"#, None),
        // `Date.now()` is allowed by default
        ("Date.now()", None),
        // Bare references are not flagged by default
        ("x instanceof Date", None),
        ("const d = Date;", None),
        ("foo(Date)", None),
        // Not the global `Date`
        ("const Date = Foo; new Date();", None),
        (r#"import Date from "foo"; new Date();"#, None),
        ("class Date {} new Date();", None),
        ("function foo(Date) { return new Date(); }", None),
        (r#"foo.Date.parse("2024-08-16")"#, None),
        ("foo.Date.UTC(2000, 11, 25)", None),
        // Type annotations are not value references
        ("let x: Date;", None),
        ("let x: Date;", check_references()),
        // `checkReferences` leaves `Date.now()` and member access alone
        ("Date.now()", check_references()),
        ("Date.prototype", check_references()),
        ("Date.length", check_references()),
        // `checkMethods` needs type information, so it does nothing
        ("function f(x) { return x.getFullYear(); }", check_methods()),
        ("Date.now()", check_methods()),
        (r#"const string = "x"; const length = string.length;"#, check_methods()),
        ("const object = {getFullYear() {}}; object.getFullYear();", check_methods()),
        ("function f(date: Date & {tag: string}) { return date.tag; }", check_methods()),
        ("function f(date: Date & {tag(): string}) { return date.tag(); }", check_methods()),
        (
            "class SubDate extends Date {} function f(date: SubDate) { return date.getTime(); }",
            check_methods(),
        ),
        ("function f(date: Date) { return date.getFullYear(); }", check_methods()),
        ("declare const date: Date; date.toISOString();", check_methods()),
        ("function f(date: Date) { return date.setUTCFullYear(2024); }", check_methods()),
        (r#"function f(date: Date) { return date["getTime"](); }"#, check_methods()),
        ("function f(date: Date | undefined) { return date?.getMonth(); }", check_methods()),
    ];

    let fail = vec![
        // `new Date()` reads the current moment
        ("new Date()", None),
        ("const now = new Date();", None),
        ("new Date", None),
        // A number argument is a count of epoch milliseconds
        ("new Date(1_724_198_400_000)", None),
        ("new Date(0)", None),
        ("new Date(-1000)", None),
        ("const ms = 1000; new Date(ms);", None),
        ("const {ms} = 1000; new Date(ms);", None),
        // A string argument goes through inconsistent parsing
        (r#"new Date("2024-08-16")"#, None),
        (r#"const string = "2024-08-16"; new Date(string);"#, None),
        ("new Date(`2024-08-16`)", None),
        // Calendar parts, with the zero-indexed month
        ("new Date(2024, 0, 1)", None),
        ("new Date(2024, 11, 25, 10, 30)", None),
        // An unknown single argument
        ("new Date(input)", None),
        (
            "const modes = new Set(['foo']); modes.clear(); new Date(modes.size ? 0 : '2020-01-01')",
            None,
        ),
        (
            "const modes = new Set(['foo']); modes.clear(); new Date((modes.size && 0) || timestamp)",
            None,
        ),
        (
            "const object = {value: true}; Object.defineProperty(object, 'value', {get() { return 0; }}); new Date(object.value ? 0 : timestamp)",
            None,
        ),
        // `Temporal.Instant.fromEpochMilliseconds()` throws on these, so no suggestion
        ("new Date(NaN)", None),
        ("new Date(Number.POSITIVE_INFINITY)", None),
        ("new Date(1.5)", None),
        ("new Date(9_000_000_000_000_000)", None),
        ("new Date(1e21)", None),
        // A `BigInt` or a spread argument is not a recognized number
        ("new Date(0n)", None),
        ("new Date(...timestamps)", None),
        // A comment inside prevents the suggestion, but the report stays
        ("new Date(/* epoch */ 1000)", None),
        (r#"Date.parse("2024-08-16")"#, None),
        ("Date.UTC(2000, 11, 25)", None),
        ("Date()", None),
        ("new Date(x as number)", None),
        // `checkDateNow`
        ("Date.now()", check_date_now()),
        ("const start = Date.now();", check_date_now()),
        ("const duration = Date.now() - start;", check_date_now()),
        (r#"Date["now"]()"#, check_date_now()),
        ("Date?.now()", check_date_now()),
        ("globalThis.Date.now()", check_date_now()),
        // `checkReferences`
        ("x instanceof Date", check_references()),
        ("const d = Date;", check_references()),
        ("foo(Date)", check_references()),
        ("[Date]", check_references()),
        (r#"typeof Date === "function""#, check_references()),
        // Construction is reported once, not twice
        ("new Date()", check_references()),
        (r#"Date.parse("2024-08-16")"#, check_references()),
        // Parentheses and TypeScript wrappers do not split the report in two
        (r#"(Date).parse("2024-08-16")"#, check_references()),
        ("const d = (Date);", check_references()),
        // `checkMethods` does nothing, but `new Date()` is still reported
        ("new Date().getFullYear();", check_methods()),
    ];

    let fix = vec![
        ("new Date(0)", "Temporal.Instant.fromEpochMilliseconds(0)", None),
        ("new Date(-1000)", "Temporal.Instant.fromEpochMilliseconds(-1000)", None),
        (
            "const ms = 1000; new Date(ms);",
            "const ms = 1000; Temporal.Instant.fromEpochMilliseconds(ms);",
            None,
        ),
        ("Date.now()", "Temporal.Now.instant().epochMilliseconds", check_date_now()),
    ];

    // `new Date()` carries three replacements for one diagnostic.
    let fix_current_moment = vec![(
        "new Date()",
        (
            "Temporal.Now.instant()",
            "Temporal.Now.zonedDateTimeISO()",
            "Temporal.Now.plainDateTimeISO()",
        ),
    )];

    Tester::new(PreferTemporal::NAME, PreferTemporal::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();

    Tester::new(
        PreferTemporal::NAME,
        PreferTemporal::PLUGIN,
        Vec::<&str>::new(),
        Vec::<&str>::new(),
    )
    .expect_fix(fix_current_moment)
    .test();
}
