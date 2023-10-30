use lazy_static::lazy_static;
use miette::diagnostic;
use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use std::collections::HashMap;

use crate::{context::LintContext, rule::Rule, utils::is_dom_node_call, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-query-selector): Prefer `.{0}()` over `.{1}()`.")]
#[diagnostic(severity(Advice), help("It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors)."))]
struct PreferQuerySelectorDiagnostic(&'static str, &'static str, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferQuerySelector;

lazy_static! {
    static ref DISALLOWED_IDENTIFIER_NAMES: HashMap<&'static str, &'static str> = [
        ("getElementById", "querySelector"),
        ("getElementsByClassName", "querySelectorAll"),
        ("getElementsByTagName", "querySelectorAll"),
    ]
    .iter()
    .copied()
    .collect();
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `.querySelector()` over `.getElementById()`, `.querySelectorAll()` over `.getElementsByClassName()` and `.getElementsByTagName()`.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// document.getElementById('foo');
    /// document.getElementsByClassName('foo bar');
    /// document.getElementsByTagName('main');
    /// document.getElementsByClassName(fn());
    ///
    /// // Good
    /// document.querySelector('#foo');
    /// document.querySelector('.bar');
    /// document.querySelector('main #foo .bar');
    /// document.querySelectorAll('.foo .bar');
    /// document.querySelectorAll('li a');
    /// document.querySelector('li').querySelectorAll('a');
    /// ```
    PreferQuerySelector,
    pedantic
);

impl Rule for PreferQuerySelector {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        if call_expr.optional || call_expr.arguments.len() != 1 {
            return;
        }

        let Expression::MemberExpression(member_expr) = &call_expr.callee else {
            return;
        };

        if member_expr.optional()
            || member_expr.is_computed()
            || !is_dom_node_call(member_expr.object())
        {
            return;
        }

        let Argument::Expression(argument_expr) = call_expr.arguments.get(0).unwrap() else {
            return;
        };

        let Some((property_span, property_name)) = member_expr.static_property_info() else {
            return;
        };

        for (cur_property_name, preferred_selector) in &*DISALLOWED_IDENTIFIER_NAMES {
            if cur_property_name != &property_name {
                continue;
            }

            let diagnostic =
                PreferQuerySelectorDiagnostic(preferred_selector, cur_property_name, property_span);

            if argument_expr.is_null() {
                return ctx.diagnostic_with_fix(diagnostic, || {
                    return Fix::new(*preferred_selector, property_span);
                });
            }

            let literal_value = match argument_expr {
                Expression::StringLiteral(literal) => Some(literal.value.trim()),
                Expression::TemplateLiteral(literal) => {
                    if literal.expressions.len() == 0 {
                        literal.quasis.get(0).unwrap().value.cooked.as_deref().map(str::trim)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(literal_value) = literal_value {
                return ctx.diagnostic_with_fix(diagnostic, || {
                    if literal_value.is_empty() {
                        return Fix::new(*preferred_selector, property_span);
                    }

                    let source_text = argument_expr.span().source_text(ctx.source_text());
                    let quotes_symbol = source_text.chars().next().unwrap();
                    let sharp = if cur_property_name.eq(&"getElementById") { "#" } else { "" };
                    return Fix::new(
                        format!(
                            "{preferred_selector}({quotes_symbol}{sharp}{literal_value}{quotes_symbol}"
                        ),
                        property_span.merge(&argument_expr.span()),
                    );
                });
            }

            ctx.diagnostic(diagnostic);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "new document.getElementById(foo);",
        "getElementById(foo);",
        "document['getElementById'](bar);",
        "document[getElementById](bar);",
        "document.foo(bar);",
        "document.getElementById();",
        "document?.getElementById('foo');",
        "document.getElementById?.('foo');",
        "document.getElementsByClassName(\"foo\", \"bar\");",
        "document.getElementById(...[\"id\"]);",
        "document.querySelector(\"#foo\");",
        "document.querySelector(\".bar\");",
        "document.querySelector(\"main #foo .bar\");",
        "document.querySelectorAll(\".foo .bar\");",
        "document.querySelectorAll(\"li a\");",
        "document.querySelector(\"li\").querySelectorAll(\"a\");",
    ];

    let fail = vec![
        "document.getElementById(\"foo\");",
        "document.getElementsByClassName(\"foo\");",
        "document.getElementsByClassName(\"foo bar\");",
        "document.getElementsByTagName(\"foo\");",
        "document.getElementById(\"\");",
        "document.getElementById('foo');",
        "document.getElementsByClassName('foo');",
        "document.getElementsByClassName('foo bar');",
        "document.getElementsByTagName('foo');",
        "document.getElementsByClassName('');",
        "document.getElementById(`foo`);",
        "document.getElementsByClassName(`foo`);",
        "document.getElementsByClassName(`foo bar`);",
        "document.getElementsByTagName(`foo`);",
        "document.getElementsByTagName(``);",
        "document.getElementsByClassName(`${fn()}`);",
        "document.getElementsByClassName(`foo ${undefined}`);",
        "document.getElementsByClassName(null);",
        "document.getElementsByTagName(null);",
        "document.getElementsByClassName(fn());",
        "document.getElementsByClassName(\"foo\" + fn());",
        "document.getElementsByClassName(foo + \"bar\");",
        "e.getElementById(3)",
    ];

    let fix = vec![
        ("document.getElementsByTagName('foo');", "document.querySelectorAll('foo');", None),
        (
            "document.getElementsByClassName(`foo bar`);",
            "document.querySelectorAll(`foo bar`);",
            None,
        ),
        ("document.getElementsByClassName(null);", "document.querySelectorAll(null);", None),
        ("document.getElementsByTagName(`   `);", "document.querySelectorAll(`   `);", None),
        ("document.getElementById(`id`);", "document.querySelector(`#id`);", None),
        (
            "document.getElementsByClassName(foo + \"bar\");",
            "document.getElementsByClassName(foo + \"bar\");",
            None,
        ),
        ("document.getElementsByClassName(fn());", "document.getElementsByClassName(fn());", None),
    ];

    Tester::new_without_config(PreferQuerySelector::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
