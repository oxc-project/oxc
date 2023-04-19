use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind, Atom, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Bad array method on arguments")]
#[diagnostic(
    severity(warning),
    help(
        "The 'arguments' object does not have '{0}()' method. If an array method was intended, consider converting the 'arguments' object to an array or using ES6 rest parameter instead."
    )
)]
struct BadArrayMethodOnArgumentsDiagnostic(Atom, #[label] pub Span);

/// `https://deepscan.io/docs/rules/bad-array-method-on-arguments`
#[derive(Debug, Default, Clone)]
pub struct BadArrayMethodOnArguments;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when an array method is called on the arguments object itself.
    ///
    /// ### Why is this bad?
    /// The arguments object is not an array, but an array-like object. It should be converted to a real array before calling an array method.
    /// Otherwise, a TypeError exception will be thrown because of the non-existent method.
    ///
    /// ### Example
    /// ```javascript
    /// function add(x, y) {
    ///   return x + y;
    /// }
    /// function sum() {
    ///   return arguments.reduce(add, 0);
    /// }
    /// ```
    BadArrayMethodOnArguments,
    correctness,
);

impl Rule for BadArrayMethodOnArguments {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // only access the Array.prototype.method can also pass, so we don't need check if the parent not a CallExpression
        let AstKind::CallExpression(_) = ctx.parent_kind(node) else {return};
        let AstKind::MemberExpression(member_expr) = node.get().kind() else {return};

        match member_expr {
            MemberExpression::StaticMemberExpression(expr) => {
                if let Some(reference) = expr.object.get_identifier_reference() {
                    if reference.name != "arguments" {
                        return;
                    }

                    if ARRAY_METHODS.iter().any(|method| method == &expr.property.name) {
                        ctx.diagnostic(BadArrayMethodOnArgumentsDiagnostic(
                            expr.property.name.clone(),
                            expr.span,
                        ));
                    }
                }
            }
            MemberExpression::ComputedMemberExpression(expr) => {
                if let Some(reference) = expr.object.get_identifier_reference() {
                    if reference.name != "arguments" {
                        return;
                    }

                    match &expr.expression {
                        Expression::StringLiteral(name) => {
                            if ARRAY_METHODS.iter().any(|method| method == &name.value) {
                                ctx.diagnostic(BadArrayMethodOnArgumentsDiagnostic(
                                    name.value.clone(),
                                    expr.span,
                                ));
                            }
                        }
                        Expression::TemplateLiteral(template) => {
                            if template.expressions.is_empty() && template.quasis.len() == 1 {
                                if let Some(name) = &template.quasis[0].value.cooked.as_deref() {
                                    if ARRAY_METHODS.iter().any(|method| method == name) {
                                        ctx.diagnostic(BadArrayMethodOnArgumentsDiagnostic(
                                            template.quasis[0]
                                                .value
                                                .cooked
                                                .as_ref()
                                                .unwrap()
                                                .clone(),
                                            expr.span,
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

// `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array#instance_methods`
const ARRAY_METHODS: [&str; 2] = ["reduce", "map"];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function sum() {}", None),
        ("function sum(...args) {return args.reduce((prev, cur) => prev + cur, 0)}", None),
        ("function sum() { arguments.foo }", None),
        ("function sum() { arguments.map }", None),
        // keep those passing tests for Deepscan Compatible
        ("function sum() {arguments[`map${''}`]((prev, cur) => prev + cur, 0)}", None),
        ("function sum() {arguments[`${''}map`]((prev, cur) => prev + cur, 0)}", None),
        ("function sum() {arguments[`${'map'}`]((prev, cur) => prev + cur, 0)}", None),
    ];

    let fail = vec![
        ("function sum() {arguments.map((prev, cur) => prev + cur, 0)}", None),
        ("function sum() {arguments['map']((prev, cur) => prev + cur, 0)}", None),
        ("function sum() {arguments[`map`]((prev, cur) => prev + cur, 0)}", None),
        ("function sum() {arguments.reduce((prev, cur) => prev + cur, 0)}", None),
    ];

    Tester::new(BadArrayMethodOnArguments::NAME, pass, fail).test_and_snapshot();
}
