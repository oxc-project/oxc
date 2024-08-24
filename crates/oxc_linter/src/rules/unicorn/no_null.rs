use oxc_ast::{
    ast::{
        Argument, BinaryExpression, CallExpression, Expression, NullLiteral, VariableDeclarator,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;
use serde_json::Value;

use crate::{
    ast_util::is_method_call,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn no_null_diagnostic(null: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `null` literals").with_label(null)
}

#[derive(Debug, Default, Clone)]
pub struct NoNull {
    check_strict_equality: bool,
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
    style,
    conditional_fix
);

fn match_null_arg(call_expr: &CallExpression, index: usize, span: Span) -> bool {
    call_expr.arguments.get(index).map_or(false, |arg| {
        if let Argument::NullLiteral(null_lit) = arg {
            return null_lit.span == span;
        }

        false
    })
}

impl NoNull {
    fn diagnose_binary_expression(
        &self,
        ctx: &LintContext,
        null_literal: &NullLiteral,
        binary_expr: &BinaryExpression,
    ) {
        match binary_expr.operator {
            // `if (foo != null) {}`
            BinaryOperator::Equality | BinaryOperator::Inequality => {
                ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
                    fix_null(fixer, null_literal)
                });
            }

            // `if (foo !== null) {}`
            BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                if self.check_strict_equality {
                    ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
                        fix_null(fixer, null_literal)
                    });
                }
            }
            _ => {
                ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
                    fix_null(fixer, null_literal)
                });
            }
        }
    }
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
        ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
            fixer.delete_range(Span::new(variable_declarator.id.span().end, null_literal.span.end))
        });

        return;
    }

    // `const foo = null`
    ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
        fix_null(fixer, null_literal)
    });
}

fn match_call_expression_pass_case(null_literal: &NullLiteral, call_expr: &CallExpression) -> bool {
    // `Object.create(null)`, `Object.create(null, foo)`
    if is_method_call(call_expr, Some(&["Object"]), Some(&["create"]), Some(1), Some(2))
        && !call_expr.optional
        && !matches!(&call_expr.callee, Expression::ComputedMemberExpression(_))
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
        && !matches!(&call_expr.callee, Expression::ComputedMemberExpression(_))
        && match_null_arg(call_expr, 1, null_literal.span)
    {
        return true;
    }

    false
}

impl Rule for NoNull {
    fn from_configuration(value: Value) -> Self {
        Self {
            check_strict_equality: value
                .get(0)
                .and_then(|v| v.get("checkStrictEquality"))
                .and_then(Value::as_bool)
                .unwrap_or_default(),
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
                self.diagnose_binary_expression(ctx, null_literal, binary_expr);
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
                ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
                    fixer.delete_range(null_literal.span)
                });

                return;
            }
        }

        ctx.diagnostic_with_fix(no_null_diagnostic(null_literal.span), |fixer| {
            fix_null(fixer, null_literal)
        });
    }
}

fn fix_null<'a>(fixer: RuleFixer<'_, 'a>, null: &NullLiteral) -> RuleFix<'a> {
    fixer.replace(null.span, "undefined")
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
        ("if (foo === null || foo === undefined) {}", None),
    ];

    let fail = vec![
        ("const foo = null", None),
        ("foo(null)", None),
        ("if (foo == null) {}", None),
        ("if (foo != null) {}", None),
        ("if (null == foo) {}", None),
        ("if (null != foo) {}", None),
        ("let curr;\nwhile (curr != null) { curr = stack.pop() }", None),
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

    let fix = vec![
        ("let x = null;", "let x;", None),
        ("let x = null as any;", "let x = undefined as any;", None),
        ("if (foo == null) {}", "if (foo == undefined) {}", None),
        ("if (foo != null) {}", "if (foo != undefined) {}", None),
        ("if (foo == null) {}", "if (foo == undefined) {}", Some(check_strict_equality(true))),
        // FIXME
        (
            "if (foo === null || foo === undefined) {}",
            "if (foo === undefined || foo === undefined) {}",
            Some(check_strict_equality(true)),
        ),
        (
            "
            let isNullish;
            switch (foo) {
                case null:
                case undefined:
                    isNullish = true;
                    break;
                default:
                    isNullish = false;
                    break;
            }
            ",
            "
            let isNullish;
            switch (foo) {
                case undefined:
                case undefined:
                    isNullish = true;
                    break;
                default:
                    isNullish = false;
                    break;
            }
            ",
            None,
        ),
    ];
    Tester::new(NoNull::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
