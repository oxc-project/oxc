use oxc_ast::{
    ast::{
        Argument, BinaryExpression, CallExpression, Expression, NullLiteral, VariableDeclarator,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-null): Disallow the use of the `null` literal")]
#[diagnostic(severity(warning), help("Replace the `null` literal with `undefined`."))]
struct ReplaceNullDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-null): Disallow the use of the `null` literal")]
#[diagnostic(severity(warning), help("Remove the `null` literal."))]
struct RemoveNullDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNull {
    check_strict_equality: Option<bool>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of the `null` literal, to encourage using `undefined` instead.
    ///
    /// ### Why is this bad?
    ///
    /// There are some reasons for using `undefined` instead of `null`.
    /// - From experience, most developers use `null` and `undefined` inconsistently and interchangeably, and few know when to use which.
    /// - Supporting both `null` and `undefined` complicates input validation.
    /// - Using `null` makes TypeScript types more verbose: `type A = {foo?: string | null}` vs `type A = {foo?: string}`.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// let foo = null;
    ///
    /// // Good
    /// let foo
    /// ```
    NoNull,
    style
);

fn match_null_arg(call_expr: &CallExpression, index: usize, span: Span) -> bool {
    call_expr.arguments.get(index).map_or(false, |arg| {
        if let Argument::Expression(Expression::NullLiteral(null_lit)) = arg {
            return null_lit.span == span;
        }

        false
    })
}

fn diagnose_binary_expression(
    no_null: &NoNull,
    ctx: &LintContext,
    null_literal: &NullLiteral,
    binary_expr: &BinaryExpression,
) {
    // checkStrictEquality=false && `if (foo !== null) {}`
    if !no_null.check_strict_equality.is_some_and(|val| val)
        && matches!(
            binary_expr.operator,
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
        )
    {
        return;
    }

    // `if (foo != null) {}`
    if matches!(binary_expr.operator, BinaryOperator::Equality | BinaryOperator::Inequality) {
        ctx.diagnostic_with_fix(ReplaceNullDiagnostic(null_literal.span), || {
            Fix::new("undefined", null_literal.span)
        });

        return;
    }

    // checkStrictEquality=true && `if (foo !== null) {}`
    ctx.diagnostic_with_fix(ReplaceNullDiagnostic(null_literal.span), || {
        Fix::new("undefined", null_literal.span)
    });
}

fn diagnose_variable_declarator(
    ctx: &LintContext,
    null_literal: &NullLiteral,
    variable_declarator: &VariableDeclarator,
    parent_kind: Option<AstKind>,
) {
    // `let foo = null;`
    if matches!(&variable_declarator.init, Some(Expression::NullLiteral(expr)) if expr.span == null_literal.span)
        && matches!(parent_kind, Some(AstKind::VariableDeclaration(var_declaration)) if !var_declaration.kind.is_const() )
    {
        ctx.diagnostic_with_fix(RemoveNullDiagnostic(null_literal.span), || {
            Fix::delete(Span {
                start: variable_declarator.id.span().end,
                end: null_literal.span.end,
            })
        });

        return;
    }

    // `const foo = null`
    ctx.diagnostic_with_fix(ReplaceNullDiagnostic(null_literal.span), || {
        Fix::new("undefined", null_literal.span)
    });
}

fn match_call_expression_pass_case(null_literal: &NullLiteral, call_expr: &CallExpression) -> bool {
    // `Object.create(null)`, `Object.create(null, foo)`
    if is_method_call(call_expr, Some(&["Object"]), Some(&["create"]), Some(1), Some(2))
        && !call_expr.optional
        && !matches!(&call_expr.callee, Expression::MemberExpression(member_expr) if member_expr.is_computed())
        && match_null_arg(call_expr, 0, null_literal.span)
    {
        return true;
    }

    // `useRef(null)`
    if let Expression::Identifier(ident) = &call_expr.callee {
        if ident.name == "useRef" && call_expr.arguments.len() == 1 && !call_expr.optional {
            return true;
        }
    }

    // `React.useRef(null)`
    if is_method_call(call_expr, Some(&["React"]), Some(&["useRef"]), Some(1), Some(1))
        && !call_expr.optional
    {
        return true;
    }

    // `foo.insertBefore(bar, null)`
    if is_method_call(call_expr, None, Some(&["insertBefore"]), Some(2), Some(2))
        && !call_expr
            .arguments
            .iter()
            .any(|argument| matches!(argument, Argument::SpreadElement(_)))
        && !call_expr.optional
        && !matches!(&call_expr.callee, Expression::MemberExpression(member_expr) if member_expr.is_computed())
        && match_null_arg(call_expr, 1, null_literal.span)
    {
        return true;
    }

    false
}

impl Rule for NoNull {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            check_strict_equality: value
                .get(0)
                .and_then(|v| v.get("checkStrictEquality"))
                .and_then(serde_json::Value::as_bool),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NullLiteral(null_literal) = node.kind() else {
            return;
        };

        if let Some(parent_node) = ctx.nodes().parent_node(node.id()) {
            let grand_parent_kind = ctx.nodes().parent_kind(parent_node.id());

            if matches!(parent_node.kind(), AstKind::Argument(_)) {
                if let Some(AstKind::CallExpression(call_expr)) = grand_parent_kind {
                    if match_call_expression_pass_case(null_literal, call_expr) {
                        return;
                    }
                }
            }

            if let AstKind::BinaryExpression(binary_expr) = parent_node.kind() {
                diagnose_binary_expression(self, ctx, null_literal, binary_expr);
                return;
            }

            if let AstKind::VariableDeclarator(variable_declarator) = parent_node.kind() {
                diagnose_variable_declarator(
                    ctx,
                    null_literal,
                    variable_declarator,
                    grand_parent_kind,
                );

                return;
            }

            // `function foo() { return null; }`,
            if matches!(parent_node.kind(), AstKind::ReturnStatement(_)) {
                ctx.diagnostic_with_fix(RemoveNullDiagnostic(null_literal.span), || {
                    Fix::delete(null_literal.span)
                });

                return;
            }
        }

        ctx.diagnostic_with_fix(ReplaceNullDiagnostic(null_literal.span), || {
            Fix::new("undefined", null_literal.span)
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn check_strict_equality(option: bool) -> serde_json::Value {
        serde_json::json!([{
            "checkStrictEquality": option,
        }])
    }

    let pass = vec![
        ("let foo", None),
        ("Object.create(null)", None),
        ("Object.create(null, {foo: {value:1}})", None),
        ("let insertedNode = parentNode.insertBefore(newNode, null)", None),
        ("const foo = \"null\";", None),
        ("Object.create()", None),
        ("Object.create(bar)", None),
        ("Object.create(\"null\")", None),
        ("useRef(null)", None),
        ("React.useRef(null)", None),
        ("if (foo === null) {}", None),
        ("if (null === foo) {}", None),
        ("if (foo !== null) {}", None),
        ("if (null !== foo) {}", None),
        ("foo = Object.create(null)", None),
        ("if (foo === null) {}", Some(check_strict_equality(false))),
        ("if (null === foo) {}", Some(check_strict_equality(false))),
        ("if (foo !== null) {}", Some(check_strict_equality(false))),
        ("if (null !== foo) {}", Some(check_strict_equality(false))),
    ];

    let fail = vec![
        ("const foo = null", None),
        ("foo(null)", None),
        ("if (foo == null) {}", None),
        ("if (foo != null) {}", None),
        ("if (null == foo) {}", None),
        ("if (null != foo) {}", None),
        // Suggestion `ReturnStatement`
        (
            "function foo() {
            return null;
            }",
            None,
        ),
        // Suggestion `VariableDeclaration`
        ("let foo = null;", None),
        ("var foo = null;", None),
        ("var foo = 1, bar = null, baz = 2;", None),
        ("const foo = null;", None),
        // `checkStrictEquality`
        ("if (foo === null) {}", Some(check_strict_equality(true))),
        ("if (null === foo) {}", Some(check_strict_equality(true))),
        ("if (foo !== null) {}", Some(check_strict_equality(true))),
        ("if (null !== foo) {}", Some(check_strict_equality(true))),
        // Not `CallExpression`
        ("new Object.create(null)", None),
        ("new foo.insertBefore(bar, null)", None),
        // Not `MemberExpression`
        ("create(null)", None),
        ("insertBefore(bar, null)", None),
        // `callee.property` is not a `Identifier`
        ("Object['create'](null)", None),
        ("foo['insertBefore'](bar, null)", None),
        // Computed
        ("Object[create](null)", None),
        ("foo[insertBefore](bar, null)", None),
        ("Object[null](null)", None),
        // Not matching method
        ("Object.notCreate(null)", None),
        ("foo.notInsertBefore(foo, null)", None),
        // Not `Object`
        ("NotObject.create(null)", None),
        // `callee.object.type` is not a `Identifier`
        ("lib.Object.create(null)", None),
        // More/Less arguments
        ("Object.create(...[null])", None),
        ("Object.create(null, bar, extraArgument)", None),
        ("foo.insertBefore(null)", None),
        ("foo.insertBefore(foo, null, bar)", None),
        ("foo.insertBefore(...[foo], null)", None),
        // Not in right position
        ("foo.insertBefore(null, bar)", None),
        ("Object.create(bar, null)", None),
    ];

    Tester::new(NoNull::NAME, pass, fail).test_and_snapshot();
}
