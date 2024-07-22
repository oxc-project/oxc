use oxc_ast::{ast::{Argument, CallExpression, ChainElement, Expression, MemberExpression, StaticMemberExpression}, match_member_expression, visit::walk::walk_expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::*, context::LintContext, rule::Rule, rules::jest::expect_expect, utils::is_same_reference, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoUselessCall;

declare_oxc_lint!(
    /// ### What it does
    /// 
    /// Disallow unnecessary calls to `.call()` and `.apply()`
    ///
    /// ### Why is this bad?
    /// 
    /// This rule is aimed to flag usage of Function.prototype.call() and Function.prototype.apply() that can be replaced with the normal function invocation.
    ///
    /// ### Example
    /// ```javascript
    /// // error
    /// // These are same as `foo(1, 2, 3);`
    /// foo.call(undefined, 1, 2, 3);
    /// foo.apply(undefined, [1, 2, 3]);
    /// foo.call(null, 1, 2, 3);
    /// foo.apply(null, [1, 2, 3]);
    /// 
    /// // These are same as `obj.foo(1, 2, 3);`
    /// obj.foo.call(obj, 1, 2, 3);
    /// obj.foo.apply(obj, [1, 2, 3]);
    /// 
    /// // success
    /// // The `this` binding is different.
    /// foo.call(obj, 1, 2, 3);
    /// foo.apply(obj, [1, 2, 3]);
    /// obj.foo.call(null, 1, 2, 3);
    /// obj.foo.apply(null, [1, 2, 3]);
    /// obj.foo.call(otherObj, 1, 2, 3);
    /// obj.foo.apply(otherObj, [1, 2, 3]);
    ///
    /// // The argument list is variadic.
    /// // Those are warned by the `prefer-spread` rule.
    /// foo.apply(undefined, args);
    /// foo.apply(null, args);
    /// obj.foo.apply(obj, args);
    /// 
    /// ```
    NoUselessCall,
    suspicious,
);

fn no_useless_call_diagnostic(span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow the use of undeclared variables.")
        .with_label(span1)
}

fn is_array_argument(_expr: Option<&Argument>) -> bool {
    let Some(expr) = _expr else {
        return false;
    };

    expr.is_array()
}

fn is_call_or_non_variadic_apply(call_expr: &CallExpression) -> bool {
    let skip_expr_callee = skip_chain_expression(&call_expr.callee);
    let Some(member_expr) = get_skip_chain_expr_member_expr(skip_expr_callee) else {
        return false;
    };

    let Some(static_name) = member_expr.static_property_name() else {
        return false;
    };

    (static_name == "call" && call_expr.arguments.len() >= 1) || 
    (static_name == "apply" && call_expr.arguments.len() == 2 && is_array_argument(call_expr.arguments.get(1)))
}

fn is_validate_this_arg(ctx: &LintContext, expected_this: Option<&Expression>, this_arg: &Expression) -> bool {
    match expected_this {
        Some(expected_this) => {
            is_same_reference(expected_this.without_parenthesized(), this_arg.without_parenthesized(), ctx)
        }
        None => {
            this_arg.is_null_or_undefined()
        }
    }
}

impl Rule for NoUselessCall {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_call_or_non_variadic_apply(call_expr) {
            return;
        }

        let callee = skip_chain_expression(&call_expr.callee);
        let Some(callee_member) = get_skip_chain_expr_member_expr(callee) else {
            return;
        };
        let applied = skip_chain_expression(callee_member.object());
        let expected_this = match get_skip_chain_expr_member_expr(applied) {
            Some(member) => Some(member.object()),
            None => None,
        };
        let Some(this_arg) = call_expr.arguments.get(0) else {
            return;
        };
        let Some(this_expr) = this_arg.as_expression() else {
            return;
        };

        if is_validate_this_arg(ctx, expected_this, this_expr) {
            ctx.diagnostic(no_useless_call_diagnostic(call_expr.callee.span()))
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.apply(obj, 1, 2);",
        "obj.foo.apply(null, 1, 2);",
        "obj.foo.apply(otherObj, 1, 2);",
        "a.b(x, y).c.foo.apply(a.b(x, z).c, 1, 2);",
        "foo.apply(obj, [1, 2]);",
        "obj.foo.apply(null, [1, 2]);",
        "obj.foo.apply(otherObj, [1, 2]);",
        "a.b(x, y).c.foo.apply(a.b(x, z).c, [1, 2]);",
        "a.b.foo.apply(a.b.c, [1, 2]);",
        "foo.apply(null, args);",
        "obj.foo.apply(obj, args);",
        "var call; foo[call](null, 1, 2);",
        "var apply; foo[apply](null, [1, 2]);",
        "foo.call();",
        "obj.foo.call();",
        "foo.apply();",
        "obj.foo.apply();",
        "obj?.foo.bar.call(obj.foo, 1, 2);", // { "ecmaVersion": 2020 },
        "class C { #call; wrap(foo) { foo.#call(undefined, 1, 2); } }", // { "ecmaVersion": 2022 }
        "(obj?.foo).test.bar.call(obj?.foo.test, 1, 2);" // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        "foo.call(undefined, 1, 2);",
        "foo.call(void 0, 1, 2);",
        "foo.call(null, 1, 2);",
        "obj.foo.call(obj, 1, 2);",
        "a.b.c.foo.call(a.b.c, 1, 2);",
        "a.b(x, y).c.foo.call(a.b(x, y).c, 1, 2);",
        "foo.apply(undefined, [1, 2]);",
        "foo.apply(void 0, [1, 2]);",
        "foo.apply(null, [1, 2]);",
        "obj.foo.apply(obj, [1, 2]);",
        "a.b.c.foo.apply(a.b.c, [1, 2]);",
        "a.b(x, y).c.foo.apply(a.b(x, y).c, [1, 2]);",
        "[].concat.apply([ ], [1, 2]);",
        "[].concat.apply([
			/*empty*/
			], [1, 2]);",
        r#"abc.get("foo", 0).concat.apply(abc . get("foo",  0 ), [1, 2]);"#,
        "foo.call?.(undefined, 1, 2);",  // { "ecmaVersion": 2020 },
        "foo?.call(undefined, 1, 2);",   // { "ecmaVersion": 2020 },
        "(foo?.call)(undefined, 1, 2);", // { "ecmaVersion": 2020 },
        "obj.foo.call?.(obj, 1, 2);",    // { "ecmaVersion": 2020 },
        "obj?.foo.call(obj, 1, 2);",     // { "ecmaVersion": 2020 },
        "(obj?.foo).call(obj, 1, 2);",   // { "ecmaVersion": 2020 },
        "(obj?.foo.call)(obj, 1, 2);",   // { "ecmaVersion": 2020 },
        "obj?.foo.bar.call(obj?.foo, 1, 2);", // { "ecmaVersion": 2020 },
        "(obj?.foo).bar.call(obj?.foo, 1, 2);", // { "ecmaVersion": 2020 },
        "obj.foo?.bar.call(obj.foo, 1, 2);", // { "ecmaVersion": 2020 }
    ];

    Tester::new(NoUselessCall::NAME, pass, fail).test_and_snapshot();
}
