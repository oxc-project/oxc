use oxc_ast::{
    ast::{CallExpression, ChainElement, Expression, MemberExpression},
    match_member_expression, AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::cmp::ContentEq;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_call_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Avoid unnecessary use of .{name}()"))
        .with_help("Replace with a normal function invocation")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessCall;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow unnecessary calls to `.call()` and `.apply()`
    ///
    /// ### Why is this bad?
    /// `Function.prototype.call()` and `Function.prototype.apply()` are slower than the normal function invocation.
    ///
    /// This rule compares code statically to check whether or not thisArg is changed.
    /// So if the code about thisArg is a dynamic expression, this rule cannot judge correctly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // These are same as `foo(1, 2, 3);`
    /// foo.call(undefined, 1, 2, 3);
    /// foo.apply(undefined, [1, 2, 3]);
    /// foo.call(null, 1, 2, 3);
    /// foo.apply(null, [1, 2, 3]);
    ///
    /// // These are same as `obj.foo(1, 2, 3);`
    /// obj.foo.call(obj, 1, 2, 3);
    /// obj.foo.apply(obj, [1, 2, 3]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
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
    /// ```
    NoUselessCall,
    eslint,
    perf
);

impl Rule for NoUselessCall {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(callee) =
            as_member_expression_without_chain_expression(call_expr.callee.without_parentheses())
        else {
            return;
        };

        if !is_call_or_non_variadic_apply(call_expr, callee) {
            return;
        }

        let applied = callee.object().without_parentheses();
        let Some(first_arg) = call_expr.arguments.first() else { return };
        let Some(this_arg) = first_arg.as_expression() else { return };

        if validate_this_argument(this_arg, applied) {
            ctx.diagnostic(no_useless_call_diagnostic(
                callee.static_property_name().unwrap(),
                call_expr.span,
            ));
        }
    }
}

fn validate_this_argument(this_arg: &Expression, applied: &Expression) -> bool {
    if this_arg.is_null_or_undefined() {
        return matches!(applied, Expression::Identifier(_));
    }

    let Some(applied_member) = as_member_expression_without_chain_expression(applied) else {
        return false;
    };

    validate_member_expression(this_arg, applied_member)
}

fn validate_member_expression(this_arg: &Expression, applied_member: &MemberExpression) -> bool {
    let applied_object = applied_member.object().without_parentheses();

    if let Some(this_arg_member) = as_member_expression_without_chain_expression(this_arg) {
        return as_member_expression_without_chain_expression(applied_object).is_some_and(
            |applied_object_member| applied_object_member.content_eq(this_arg_member),
        );
    }

    applied_object.content_eq(this_arg)
}

fn is_call_function(call_expr: &CallExpression, callee: &MemberExpression) -> bool {
    callee.static_property_name().is_some_and(|name| name == "call")
        && !call_expr.arguments.is_empty()
}

fn is_apply_function(call_expr: &CallExpression, callee: &MemberExpression) -> bool {
    callee.static_property_name().is_some_and(|name| name == "apply")
        && call_expr.arguments.len() == 2
        && call_expr
            .arguments
            .get(1)
            .and_then(|arg| arg.as_expression())
            .is_some_and(|expr| matches!(expr, Expression::ArrayExpression(_)))
}

fn is_call_or_non_variadic_apply(call_expr: &CallExpression, callee: &MemberExpression) -> bool {
    !callee.is_computed()
        && (is_call_function(call_expr, callee) || is_apply_function(call_expr, callee))
}

fn as_member_expression_without_chain_expression<'a>(
    expr: &'a Expression,
) -> Option<&'a MemberExpression<'a>> {
    match expr {
        Expression::ChainExpression(chain_expr) => {
            if let match_member_expression!(ChainElement) = chain_expr.expression {
                chain_expr.expression.as_member_expression()
            } else {
                None
            }
        }
        match_member_expression!(Expression) => expr.as_member_expression(),
        _ => None,
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
            ], [1, 2]);
        ",
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

    Tester::new(NoUselessCall::NAME, NoUselessCall::PLUGIN, pass, fail).test_and_snapshot();
}
