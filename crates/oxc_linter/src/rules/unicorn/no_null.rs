use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-null): Disallow the use of the `null` literal")]
#[diagnostic(severity(warning), help("Replace the `null` literal with `undefined`."))]
struct NoNullDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNull {
    check_strict_equality: Option<bool>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoNull,
    correctness
);

impl Rule for NoNull {
    fn from_configuration(_value: serde_json::Value) -> Self {
        Self {
            check_strict_equality: _value
                .get(0)
                .and_then(|v| v.get("checkStrictEquality"))
                .and_then(serde_json::Value::as_bool),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NullLiteral(null_lit) = node.kind() else {
            return;
        };

        if let Some(maybe_parent) = ctx.nodes().parent_node(node.id()) {
            if let AstKind::BinaryExpression(binary_expr) = maybe_parent.kind() {
                if !self.check_strict_equality.is_some_and(|val| val == true)
                    && matches!(
                        binary_expr.operator,
                        BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
                    )
                {
                    return;
                }
            }

            if matches!(maybe_parent.kind(), AstKind::Argument(_)) {
                if let Some(AstKind::CallExpression(call_expr)) =
                    ctx.nodes().parent_kind(maybe_parent.id())
                {
                    // `Object.create(null)`, `Object.create(null, foo)`
                    if is_method_call(
                        call_expr,
                        Some(&["Object"]),
                        Some(&["create"]),
                        Some(1),
                        Some(2),
                    ) && !call_expr.optional
                        && !matches!(&call_expr.callee, Expression::MemberExpression(member_expr) if member_expr.is_computed())
                    {
                        return;
                    }

                    // `useRef(null)`
                    if let Expression::Identifier(ident) = &call_expr.callee {
                        if ident.name == "useRef"
                            && call_expr.arguments.len() == 1
                            && !call_expr.optional
                        {
                            return;
                        }
                    }

                    // `React.useRef(null)`
                    if is_method_call(
                        call_expr,
                        Some(&["React"]),
                        Some(&["useRef"]),
                        Some(1),
                        Some(1),
                    ) && !call_expr.optional
                    {
                        return;
                    }

                    // `foo.insertBefore(bar, null)`
                    if is_method_call(call_expr, None, Some(&["insertBefore"]), Some(2), Some(2))
                        && !call_expr
                            .arguments
                            .iter()
                            .any(|argument| matches!(argument, Argument::SpreadElement(_)))
                        && !call_expr.optional
                        && !matches!(&call_expr.callee, Expression::MemberExpression(member_expr) if member_expr.is_computed())
                    {
                        return;
                    }
                }
            }
        }

        ctx.diagnostic(NoNullDiagnostic(null_lit.span));
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
