use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use phf::phf_map;

use crate::{context::LintContext, rule::Rule, utils::is_node_value_not_dom_node, AstNode};

fn prefer_query_selector_diagnostic(
    good_method: &str,
    bad_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `.{good_method}()` over `.{bad_method}()`."))
        .with_help("It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferQuerySelector;

const DISALLOWED_IDENTIFIER_NAMES: phf::Map<&'static str, &'static str> = phf_map!(
    "getElementById" => "querySelector",
    "getElementsByClassName" => "querySelectorAll",
    "getElementsByTagName" => "querySelectorAll"
);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `.querySelector()` over `.getElementById()`, `.querySelectorAll()` over `.getElementsByClassName()` and `.getElementsByTagName()`.
    ///
    /// ### Why is this bad?
    ///
    /// - Using `.querySelector()` and `.querySelectorAll()` is more flexible and allows for more specific selectors.
    /// - It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// document.getElementById('foo');
    /// document.getElementsByClassName('foo bar');
    /// document.getElementsByTagName('main');
    /// document.getElementsByClassName(fn());
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// document.querySelector('#foo');
    /// document.querySelector('.bar');
    /// document.querySelector('main #foo .bar');
    /// document.querySelectorAll('.foo .bar');
    /// document.querySelectorAll('li a');
    /// document.querySelector('li').querySelectorAll('a');
    /// ```
    PreferQuerySelector,
    unicorn,
    pedantic,
    conditional_fix
);

impl Rule for PreferQuerySelector {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional || call_expr.arguments.len() != 1 {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if member_expr.optional()
            || member_expr.is_computed()
            || is_node_value_not_dom_node(member_expr.object())
        {
            return;
        }

        let Some(argument_expr) = call_expr.arguments[0].as_expression() else {
            return;
        };

        let Some((property_span, property_name)) = member_expr.static_property_info() else {
            return;
        };

        for (cur_property_name, preferred_selector) in &DISALLOWED_IDENTIFIER_NAMES {
            if cur_property_name != &property_name {
                continue;
            }

            let diagnostic = prefer_query_selector_diagnostic(
                preferred_selector,
                cur_property_name,
                property_span,
            );

            if argument_expr.is_null() {
                return ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    fixer.replace(property_span, *preferred_selector)
                });
            }

            let literal_value = match argument_expr {
                Expression::StringLiteral(literal) => Some(literal.value.trim()),
                Expression::TemplateLiteral(literal) => {
                    if literal.expressions.len() == 0 {
                        literal.quasis.first().unwrap().value.cooked.as_deref().map(str::trim)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(literal_value) = literal_value {
                return ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    if literal_value.is_empty() {
                        return fixer.replace(property_span, *preferred_selector);
                    }

                    let source_text = fixer.source_range(argument_expr.span());
                    let quotes_symbol = source_text.chars().next().unwrap();
                    let argument = match *cur_property_name {
                        "getElementById" => format!("#{literal_value}"),
                        "getElementsByClassName" => {
                            format!(
                                ".{}",
                                literal_value.split_whitespace().collect::<Vec<_>>().join(" .")
                            )
                        }
                        _ => literal_value.to_string(),
                    };
                    let span = property_span.merge(argument_expr.span());
                    fixer.replace(
                        span,
                        format!("{preferred_selector}({quotes_symbol}{argument}{quotes_symbol}"),
                    )
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
            "document.querySelectorAll(`.foo .bar`);",
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

    Tester::new(PreferQuerySelector::NAME, PreferQuerySelector::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
