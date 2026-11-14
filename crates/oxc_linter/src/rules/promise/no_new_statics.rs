use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{AstNode, context::LintContext, rule::Rule, utils::PROMISE_STATIC_METHODS};

fn static_promise_diagnostic(static_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use `new` on `Promise.{static_name}`"))
        .with_help(format!(
            "`Promise.{static_name}` is not a constructor. Call it as a function instead."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNewStatics;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows calling new on static `Promise` methods.
    ///
    /// ### Why is this bad?
    ///
    /// Calling a static `Promise` method with `new` is invalid and will result
    /// in a `TypeError` at runtime.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const x = new Promise.resolve(value);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const x = Promise.resolve(value);
    /// ```
    NoNewStatics,
    promise,
    correctness,
    recommended,
    suggestion,
    version = "0.6.1",
    short_description = "Disallows calling new on static `Promise` methods.",
);

impl Rule for NoNewStatics {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = &new_expr.callee.get_member_expr() else {
            return;
        };

        let Expression::Identifier(ident) = &member_expr.object() else {
            return;
        };

        if ident.name != "Promise" || !ctx.is_reference_to_global_variable(ident) {
            return;
        }

        let Some(prop_name) = member_expr.static_property_name().and_then(JSStr::as_str) else {
            return;
        };

        if PROMISE_STATIC_METHODS.contains(&prop_name) {
            ctx.diagnostic_with_suggestion(
                static_promise_diagnostic(
                    prop_name,
                    Span::new(new_expr.span.start, ident.span.start - 1),
                ),
                |fixer| {
                    let callee_span = new_expr.callee.span();
                    let source = ctx.source_range(Span::new(callee_span.start, new_expr.span.end));
                    let callee_end = new_expr
                        .type_arguments
                        .as_ref()
                        .map_or(callee_span.end, |arguments| arguments.span.end);
                    if callee_end == new_expr.span.end {
                        fixer.replace(new_expr.span, format!("{source}()"))
                    } else {
                        fixer.replace(new_expr.span, source.to_string())
                    }
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "Promise.resolve()",
        "Promise.reject()",
        "Promise.all()",
        "Promise.race()",
        "new Promise(function (resolve, reject) {})",
        "new SomeClass()",
        "SomeClass.resolve()",
        "new SomeClass.resolve()",
    ];

    let fail = vec![
        "new Promise.resolve()",
        "new Promise.reject()",
        "new Promise.all()",
        "new Promise.allSettled()",
        "new Promise.any()",
        "new Promise.race()",
        "function foo() {
			  var a = getA()
			  return new Promise.resolve(a)
			}",
    ];

    let fix = vec![
        ("new Promise.resolve()", "Promise.resolve()", None),
        ("new Promise.reject()", "Promise.reject()", None),
        ("new Promise.all()", "Promise.all()", None),
        ("new Promise.allSettled()", "Promise.allSettled()", None),
        ("new Promise.any()", "Promise.any()", None),
        ("new Promise.race()", "Promise.race()", None),
        ("new Promise.resolve;", "Promise.resolve();", None),
        ("new (Promise.resolve);", "(Promise.resolve)();", None),
        ("new (Promise.resolve)(1);", "(Promise.resolve)(1);", None),
        ("new Promise.resolve<string>;", "Promise.resolve<string>();", None),
        ("new Promise['resolve'];", "Promise['resolve']();", None),
    ];
    let fix = fix
        .into_iter()
        .flat_map(|(source, expected, config): (_, _, Option<serde_json::Value>)| {
            [
                (source, source, config.clone(), FixKind::SafeFix),
                (source, expected, config, FixKind::Suggestion),
            ]
        })
        .collect::<Vec<_>>();
    Tester::new(NoNewStatics::NAME, NoNewStatics::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
