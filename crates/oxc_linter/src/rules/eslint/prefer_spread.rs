use oxc_ast::{
    ast::{match_member_expression, CallExpression, ChainElement, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{cmp::ContentEq, Span};

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn eslint_prefer_spread_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require spread operators instead of .apply()").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSpread;

declare_oxc_lint!(
    /// This rule is combined 2 rules from `eslint:prefer-spread` and `unicorn:prefer-spread`.
    ///
    /// ### What it does
    ///
    /// Require spread operators instead of .apply()
    ///
    /// ### Why is this bad?
    ///
    /// Before ES2015, one must use Function.prototype.apply() to call variadic functions.
    /// ```javascript
    /// var args = [1, 2, 3, 4];
    /// Math.max.apply(Math, args);
    /// ```
    ///
    /// In ES2015, one can use spread syntax to call variadic functions.
    /// ```javascript
    /// var args = [1, 2, 3, 4];
    /// Math.max(...args);
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.apply(undefined, args);
    /// foo.apply(null, args);
    /// obj.foo.apply(obj, args);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // Using spread syntax
    /// foo(...args);
    /// obj.foo(...args);
    ///
    /// // The `this` binding is different.
    /// foo.apply(obj, args);
    /// obj.foo.apply(null, args);
    /// obj.foo.apply(otherObj, args);
    ///
    /// // The argument list is not variadic.
    /// // Those are warned by the `no-useless-call` rule.
    /// foo.apply(undefined, [1, 2, 3]);
    /// foo.apply(null, [1, 2, 3]);
    /// obj.foo.apply(obj, [1, 2, 3]);
    /// ```
    PreferSpread,
    eslint,
    style
);

impl Rule for PreferSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        check_eslint_prefer_spread(call_expr, ctx);
    }
}

fn check_eslint_prefer_spread(call_expr: &CallExpression, ctx: &LintContext) {
    if !is_method_call(call_expr, None, Some(&["apply"]), Some(2), Some(2)) {
        return;
    }

    let callee = call_expr.callee.without_parentheses();
    let callee = match callee {
        match_member_expression!(Expression) => callee.to_member_expression(),
        Expression::ChainExpression(chain) => match chain.expression {
            match_member_expression!(ChainElement) => chain.expression.to_member_expression(),
            _ => return,
        },
        _ => return,
    };

    let args = &call_expr.arguments;
    let Some(args0) = args[0].as_expression() else {
        return;
    };

    if args[1].is_spread() {
        return;
    }
    if let Some(Expression::ArrayExpression(_)) = args[1].as_expression() {
        return;
    }

    let applied = callee.object().without_parentheses();

    if args0.is_null_or_undefined() {
        if !matches!(applied, Expression::Identifier(_)) {
            return;
        }
    } else if let Some(applied) = as_member_expression_without_chain_expression(applied) {
        let applied_object = applied.object().without_parentheses();

        if let Some(args0) = as_member_expression_without_chain_expression(args0) {
            let Some(applied_object) =
                as_member_expression_without_chain_expression(applied_object)
            else {
                return;
            };

            if applied_object.content_ne(args0) {
                return;
            }
        } else if applied_object.content_ne(args0) {
            return;
        }
    } else {
        return;
    }

    ctx.diagnostic(eslint_prefer_spread_diagnostic(call_expr.span));
}

fn as_member_expression_without_chain_expression<'a>(
    expr: &'a Expression,
) -> Option<&'a MemberExpression<'a>> {
    match expr {
        Expression::ChainExpression(chain_expr) => match chain_expr.expression {
            match_member_expression!(ChainElement) => chain_expr.expression.as_member_expression(),
            _ => None,
        },
        match_member_expression!(Expression) => expr.as_member_expression(),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        /* Test cases for original eslint:prefer-spread */
        "foo.apply(obj, args);",
        "obj.foo.apply(null, args);",
        "obj.foo.apply(otherObj, args);",
        "a.b(x, y).c.foo.apply(a.b(x, z).c, args);",
        "a.b.foo.apply(a.b.c, args);",
        "foo.apply(undefined, [1, 2]);",
        "foo.apply(null, [1, 2]);",
        "obj.foo.apply(obj, [1, 2]);",
        "var apply; foo[apply](null, args);",
        "foo.apply();",
        "obj.foo.apply();",
        "obj.foo.apply(obj, ...args)",
        "(a?.b).c.foo.apply(a?.b.c, args);",
        "a?.b.c.foo.apply((a?.b).c, args);",
        "class C { #apply; foo() { foo.#apply(undefined, args); } }",
    ];

    let fail = vec![
        "foo.apply(undefined, args);",
        "foo.apply(void 0, args);",
        "foo.apply(null, args);",
        "obj.foo.apply(obj, args);",
        "a.b.c.foo.apply(a.b.c, args);",
        "a.b(x, y).c.foo.apply(a.b(x, y).c, args);",
        "[].concat.apply([ ], args);",
        "[].concat.apply([
			/*empty*/
			], args);",
        "foo.apply?.(undefined, args);",
        "foo?.apply(undefined, args);",
        "foo?.apply?.(undefined, args);",
        "(foo?.apply)(undefined, args);",
        "(foo?.apply)?.(undefined, args);",
        "(obj?.foo).apply(obj, args);",
        "a?.b.c.foo.apply(a?.b.c, args);",
        "(a?.b.c).foo.apply(a?.b.c, args);",
        "(a?.b).c.foo.apply((a?.b).c, args);",
        "class C { #foo; foo() { obj.#foo.apply(obj, args); } }",
    ];

    Tester::new(PreferSpread::NAME, PreferSpread::PLUGIN, pass, fail).test_and_snapshot();
}
