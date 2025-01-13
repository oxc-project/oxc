use oxc_allocator::Box;
use oxc_ast::{
    ast::{
        Argument, AssignmentOperator, CallExpression, Expression, FormalParameters,
        LogicalOperator, SimpleAssignmentTarget,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn prefer_promise_reject_errors_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected the Promise rejection reason to be an Error").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferPromiseRejectErrors {
    allow_empty_reject: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require using Error objects as Promise rejection reasons
    ///
    /// ### Why is this bad?
    ///
    /// It is considered good practice to only pass instances of the built-in `Error` object to the `reject()` function for user-defined errors in Promises. `Error` objects automatically store a stack trace, which can be used to debug an error by determining where it came from. If a Promise is rejected with a non-`Error` value, it can be difficult to determine where the rejection occurred.
    ///
    /// ### Options
    ///
    /// This rule takes one optional object argument:
    /// - `allowEmptyReject: true` (`false` by default) allows calls to `Promise.reject()` with no arguments.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Promise.reject("something bad happened");
    ///
    /// Promise.reject(5);
    ///
    /// Promise.reject();
    ///
    /// new Promise(function(resolve, reject) {
    ///     reject("something bad happened")
    /// });
    ///
    /// new Promise(function(resolve, reject) {
    ///     reject();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Promise.reject(new Error("something bad happened"));
    ///
    /// Promise.reject(new TypeError("something bad happened"));
    ///
    /// new Promise(function(resolve, reject) {
    ///     reject(new Error("something bad happened"));
    /// });
    ///
    /// var foo = getUnknownValue();
    /// Promise.reject(foo);
    /// ```
    PreferPromiseRejectErrors,
    eslint,
    style,
    none
);

impl Rule for PreferPromiseRejectErrors {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_empty_reject = value.get(0).map_or(false, |v| {
            v.get("allowEmptyReject").map_or(false, |b| b.as_bool().unwrap_or(false))
        });

        Self { allow_empty_reject }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if !is_method_call(call_expr, Some(&["Promise"]), Some(&["reject"]), None, None) {
                    return;
                }

                check_reject_call(call_expr, ctx, self.allow_empty_reject);
            }
            AstKind::NewExpression(new_expr) => {
                let Expression::Identifier(ident) = &new_expr.callee else {
                    return;
                };

                if ident.name != "Promise" || new_expr.arguments.len() == 0 {
                    return;
                }

                let Some(arg) =
                    new_expr.arguments[0].as_expression().map(Expression::get_inner_expression)
                else {
                    return;
                };

                match arg {
                    Expression::FunctionExpression(func) => {
                        check_reject_in_function(&func.params, ctx, self.allow_empty_reject);
                    }
                    Expression::ArrowFunctionExpression(func) => {
                        check_reject_in_function(&func.params, ctx, self.allow_empty_reject);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn check_reject_call(call_expr: &CallExpression, ctx: &LintContext, allow_empty_reject: bool) {
    if call_expr.arguments.len() == 0 && allow_empty_reject {
        return;
    }

    if call_expr.arguments.len() == 0
        || call_expr.arguments[0].as_expression().is_some_and(|e| !could_be_error(e))
        || is_undefined(&call_expr.arguments[0])
    {
        ctx.diagnostic(prefer_promise_reject_errors_diagnostic(call_expr.span));
    }
}

fn check_reject_in_function(
    params: &Box<'_, FormalParameters<'_>>,
    ctx: &LintContext,
    allow_empty_reject: bool,
) {
    if params.parameters_count() <= 1 {
        return;
    }

    let Some(reject_arg) = params.items[1].pattern.get_binding_identifier() else {
        return;
    };

    ctx.symbol_references(reject_arg.symbol_id()).for_each(|reference| {
        let Some(node) = ctx.nodes().parent_node(reference.node_id()) else {
            return;
        };
        if let AstKind::CallExpression(call_expr) = node.kind() {
            check_reject_call(call_expr, ctx, allow_empty_reject);
        }
    });
}
/**
 * Port from eslint.
 * @see <https://github.com/eslint/eslint/blob/36ef8bbeab495ef2598a4b1f52e32b4cb50be5e2/lib/rules/utils/ast-utils.js#L2079>
 */
fn could_be_error(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(_)
        | Expression::CallExpression(_)
        | Expression::NewExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::YieldExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::ChainExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::AssignmentExpression(expr) => match expr.operator {
            AssignmentOperator::Assign | AssignmentOperator::LogicalAnd => {
                could_be_error(&expr.right)
            }
            AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish => {
                expr.left.as_simple_assignment_target().map_or(false, |left| {
                    matches!(
                        left,
                        SimpleAssignmentTarget::AssignmentTargetIdentifier(_)
                            | SimpleAssignmentTarget::ComputedMemberExpression(_)
                            | SimpleAssignmentTarget::StaticMemberExpression(_)
                            | SimpleAssignmentTarget::PrivateFieldExpression(_)
                    )
                }) || could_be_error(&expr.right)
            }
            _ => false,
        },
        Expression::SequenceExpression(expr) => {
            expr.expressions.len() != 0 && could_be_error(expr.expressions.last().unwrap())
        }
        Expression::LogicalExpression(expr) => {
            if expr.operator == LogicalOperator::And {
                return could_be_error(&expr.right);
            }

            could_be_error(&expr.left) || could_be_error(&expr.right)
        }
        Expression::ConditionalExpression(expr) => {
            could_be_error(&expr.consequent) || could_be_error(&expr.alternate)
        }
        _ => false,
    }
}

fn is_undefined(arg: &Argument) -> bool {
    match arg.as_expression().map(oxc_ast::ast::Expression::get_inner_expression) {
        Some(Expression::Identifier(ident)) => ident.name == "undefined",
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Promise.resolve(5)", None),
        ("Foo.reject(5)", None),
        ("Promise.reject(foo)", None),
        ("Promise.reject(foo.bar)", None),
        ("Promise.reject(foo.bar())", None),
        ("Promise.reject(new Error())", None),
        ("Promise.reject(new TypeError)", None),
        ("Promise.reject(new Error('foo'))", None),
        ("Promise.reject(foo || 5)", None),
        ("Promise.reject(5 && foo)", None),
        ("new Foo((resolve, reject) => reject(5))", None),
        ("new Promise(function(resolve, reject) { return function(reject) { reject(5) } })", None),
        ("new Promise(function(resolve, reject) { if (foo) { const reject = somethingElse; reject(5) } })", None),
        ("new Promise(function(resolve, {apply}) { apply(5) })", None),
        ("new Promise(function(resolve, reject) { resolve(5, reject) })", None),
        ("async function foo() { Promise.reject(await foo); }", None),
        ("Promise.reject()", Some(serde_json::json!([{ "allowEmptyReject": true }]))),
        ("new Promise(function(resolve, reject) { reject() })", Some(serde_json::json!([{ "allowEmptyReject": true }]))),
        ("Promise.reject(obj?.foo)", None),
        ("Promise.reject(obj?.foo())", None),
        ("Promise.reject(foo = new Error())", None),
        ("Promise.reject(foo ||= 5)", None),
        ("Promise.reject(foo.bar ??= 5)", None),
        ("Promise.reject(foo[bar] ??= 5)", None),
        ("class C { #reject; foo() { Promise.#reject(5); } }", None),
        ("class C { #error; foo() { Promise.reject(this.#error); } }", None)
    ];

    let fail = vec![
        ("Promise.reject(5)", None),
        ("Promise.reject('foo')", None),
        ("Promise.reject(`foo`)", None),
        ("Promise.reject(!foo)", None),
        ("Promise.reject(void foo)", None),
        ("Promise.reject()", None),
        ("Promise.reject(undefined)", None),
        ("Promise.reject({ foo: 1 })", None),
        ("Promise.reject([1, 2, 3])", None),
        ("Promise.reject()", Some(serde_json::json!([{ "allowEmptyReject": false }]))),
        (
            "new Promise(function(resolve, reject) { reject() })",
            Some(serde_json::json!([{ "allowEmptyReject": false }])),
        ),
        ("Promise.reject(undefined)", Some(serde_json::json!([{ "allowEmptyReject": true }]))),
        ("Promise.reject('foo', somethingElse)", None),
        ("new Promise(function(resolve, reject) { reject(5) })", None),
        ("new Promise((resolve, reject) => { reject(5) })", None),
        ("new Promise((resolve, reject) => reject(5))", None),
        ("new Promise((resolve, reject) => reject())", None),
        ("new Promise(function(yes, no) { no(5) })", None),
        (
            "
            new Promise((resolve, reject) => {
                fs.readFile('foo.txt', (err, file) => {
                if (err) reject('File not found')
                else resolve(file)
                })
            })
            ",
            None,
        ),
        ("new Promise(({foo, bar, baz}, reject) => reject(5))", None),
        ("new Promise(function(reject, reject) { reject(5) })", None),
        ("new Promise(function(foo, arguments) { arguments(5) })", None),
        ("new Promise((foo, arguments) => arguments(5))", None),
        ("new Promise(function({}, reject) { reject(5) })", None),
        ("new Promise(({}, reject) => reject(5))", None),
        ("new Promise((resolve, reject, somethingElse = reject(5)) => {})", None),
        // Optional chaining
        ("Promise.reject?.(5)", None),
        ("Promise?.reject(5)", None),
        ("Promise?.reject?.(5)", None),
        ("(Promise?.reject)(5)", None),
        ("(Promise?.reject)?.(5)", None),
        // Assignments with mathematical operators will either evaluate to a primitive value or throw a TypeError
        ("Promise.reject(foo += new Error())", None),
        ("Promise.reject(foo -= new Error())", None),
        ("Promise.reject(foo **= new Error())", None),
        ("Promise.reject(foo <<= new Error())", None),
        ("Promise.reject(foo |= new Error())", None),
        ("Promise.reject(foo &= new Error())", None),
        // evaluates either to a falsy value of `foo` (which, then, cannot be an Error object), or to `5`
        ("Promise.reject(foo && 5)", None),
        ("Promise.reject(foo &&= 5)", None),
    ];

    Tester::new(PreferPromiseRejectErrors::NAME, PreferPromiseRejectErrors::PLUGIN, pass, fail)
        .test_and_snapshot();
}
