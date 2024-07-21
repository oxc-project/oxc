use oxc_ast::{ast::{Argument, CallExpression, ChainElement, Expression, StaticMemberExpression}, match_member_expression, visit::walk::walk_expression, AstKind};
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
    /// ```
    NoUselessCall,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
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

fn is_apply_member(call_expr: &CallExpression, member_expr: &StaticMemberExpression) -> bool {
    (member_expr.property.name == "call" && call_expr.arguments.len() >= 1) || 
    (member_expr.property.name == "apply" && call_expr.arguments.len() == 2 && is_array_argument(call_expr.arguments.get(1)))
}

fn is_call_or_non_variadic_apply(call_expr: &CallExpression, ctx: &LintContext) -> bool {
    let skip_expr_callee = skip_chain_expression(&call_expr.callee);
    let Some(static_member_expr) = get_skip_chain_expression_static_member_expression(skip_expr_callee) else {
        return false;
    };

    is_apply_member(call_expr, static_member_expr)
}

fn is_validate_this_arg(ctx: &LintContext, expected_this: Option<&Expression>, this_arg: &Argument) -> bool {
    match expected_this {
        Some(expected_this) => {
            let Some(this) = this_arg.as_expression() else {
                return false;
            };
            is_same_reference(skip_parenthesized_expression(expected_this), skip_parenthesized_expression(this), ctx)
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
        if !is_call_or_non_variadic_apply(call_expr, ctx) {
            return
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


        if is_validate_this_arg(ctx, expected_this, this_arg) {
            ctx.diagnostic(no_useless_call_diagnostic(call_expr.span))
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
