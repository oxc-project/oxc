use oxc_ast::{
    AstKind,
    ast::{Expression, RegExpFlags},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::{extract_regex_flags, get_declaration_of_variable, is_method_call},
    context::LintContext,
    rule::Rule,
};

fn bad_match_all_arg_diagnostic(match_all_span: Span, regex_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Global flag (g) is missing in the regular expression supplied to the `matchAll` method.")
        .with_help("To match all occurrences, use the `matchAll` method with the global flag (g) in the regular expression.")
        .with_note("Unlike `match`, `matchAll` throws a `TypeError` when passed a non-global regular expression instead of returning an iterator over all matches.")
        .with_labels([
            match_all_span.label("`matchAll` called here"),
            regex_span.label("RegExp supplied here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct BadMatchAllArg;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when the `matchAll` method is called with a regular expression that does not have the global flag (g).
    ///
    /// ### Why is this bad?
    ///
    /// When called with a regular expression, the `matchAll` method requires the global flag (g).
    /// Otherwise, it throws a `TypeError` at runtime instead of returning an iterator.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// str.matchAll(/abc/);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// str.matchAll(/abc/g);
    /// ```
    BadMatchAllArg,
    oxc,
    correctness,
    version = "1.76.0",
    short_description = "This rule warns when the `matchAll` method is called with a regular expression that does not have the global flag (g).",
);

impl Rule for BadMatchAllArg {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["matchAll"]), Some(1), None) {
            return;
        }

        let Some(regexp_argument) = call_expr.arguments[0].as_expression() else {
            return;
        };

        let Some((flags, regex_span)) = resolve_flags(regexp_argument, ctx) else {
            return;
        };

        if !flags.contains(RegExpFlags::G) {
            let Some(call_expr_callee) = call_expr.callee.as_member_expression() else {
                return;
            };
            let Some((match_all_span, _)) = call_expr_callee.static_property_info() else {
                return;
            };

            ctx.diagnostic(bad_match_all_arg_diagnostic(match_all_span, regex_span));
        }
    }
}

fn resolve_flags<'a>(
    expr: &'a Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<(RegExpFlags, Span)> {
    match expr.without_parentheses() {
        Expression::RegExpLiteral(regexp_literal) => {
            Some((regexp_literal.regex.flags, regexp_literal.span))
        }
        Expression::NewExpression(new_expr) => {
            if new_expr.callee.is_specific_id("RegExp") {
                extract_regex_flags(&new_expr.arguments).map(|flags| (flags, new_expr.span))
            } else {
                None
            }
        }
        Expression::Identifier(ident) => {
            let decl = get_declaration_of_variable(ident, ctx)?;
            let var_decl = decl.kind().as_variable_declarator()?;
            if let Some(init) = &var_decl.init {
                return resolve_flags(init, ctx);
            }
            None
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // valid call with global flag
        r"str.matchAll(/abc/g);",
        // incorrect number of arguments
        r"str.matchAll();",
        // not a method call
        r"matchAll(/abc/, 'flags');",
        // not a method call
        r"str();",
        // new RegExp with global flag
        r"str.matchAll(new RegExp('abc', 'g'));",
        // new matchAll
        r"new matchAll(/abc/);",
        // resolved vars - string
        r#"const foo = "string"; str.matchAll(foo);"#,
        // resolved vars - regex with global flag
        r"const foo = /abc/g; str.matchAll(foo);",
        // resolved vars - new RegExp with global flag
        r"const foo = new RegExp('abc', 'g'); str.matchAll(foo);",
        r"const foo = new RegExp('abc', isWindows ? 'g' : 'gi'); str.matchAll(foo);",
    ];

    let fail = vec![
        r"str.matchAll(/abc/);",
        r"str.matchAll(/abc/i);",
        r"str.matchAll(new RegExp('abc'));",
        r"str.matchAll(new RegExp('abc','i'));",
        // resolved vars
        r"
            const foo = /abc/;

            str.matchAll(foo);
        ",
        r"
            const foo = /abc/i;

            str.matchAll(foo);
        ",
        r"
            const foo = new RegExp('abc');

            str.matchAll(foo);
        ",
    ];

    Tester::new(BadMatchAllArg::NAME, BadMatchAllArg::PLUGIN, pass, fail).test_and_snapshot();
}
