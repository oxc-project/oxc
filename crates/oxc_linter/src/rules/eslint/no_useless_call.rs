use oxc_ast::{
    AstKind,
    ast::{CallExpression, ChainElement, Expression, MemberExpression},
    match_member_expression,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::ContentEq;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_call_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Avoid unnecessary use of .{name}()"))
        .with_help("Replace with a normal function invocation")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessCall;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary calls to `.call()` and `.apply()`
    ///
    /// ### Why is this bad?
    ///
    /// `Function.prototype.call()` and `Function.prototype.apply()` are slower than the normal function invocation.
    ///
    /// This rule compares code statically to check whether or not thisArg is changed.
    /// So if the code about thisArg is a dynamic expression, this rule cannot judge correctly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // These are the same as `foo(1, 2, 3);`
    /// foo.call(undefined, 1, 2, 3);
    /// foo.apply(undefined, [1, 2, 3]);
    /// foo.call(null, 1, 2, 3);
    /// foo.apply(null, [1, 2, 3]);
    ///
    /// // These are the same as `obj.foo(1, 2, 3);`
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
    perf,
    version = "0.15.9",
    short_description = "Disallow unnecessary calls to `.call()` and `.apply()`",
);

impl Rule for NoUselessCall {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(callee) = classify_callee(&call_expr.callee) else { return };
        if !callee.kind.has_valid_arguments(call_expr) {
            return;
        }

        let Some(first_arg) = call_expr.arguments.first() else { return };
        let Some(this_arg) = first_arg.as_expression() else { return };

        if validate_this_argument(this_arg, callee.applied) {
            ctx.diagnostic(no_useless_call_diagnostic(callee.kind.name(), call_expr.span));
        }
    }
}

struct ClassifiedCallee<'a> {
    kind: CallOrApply,
    applied: &'a Expression<'a>,
}

#[derive(Clone, Copy)]
enum CallOrApply {
    Call,
    Apply,
}

impl CallOrApply {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "call" => Some(Self::Call),
            "apply" => Some(Self::Apply),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Apply => "apply",
        }
    }

    fn has_valid_arguments(self, call_expr: &CallExpression) -> bool {
        match self {
            Self::Call => !call_expr.arguments.is_empty(),
            Self::Apply => {
                call_expr.arguments.len() == 2
                    && call_expr
                        .arguments
                        .get(1)
                        .and_then(|arg| arg.as_expression())
                        .is_some_and(|expr| matches!(expr, Expression::ArrayExpression(_)))
            }
        }
    }
}

fn classify_callee<'a>(callee: &'a Expression<'a>) -> Option<ClassifiedCallee<'a>> {
    if let Expression::StaticMemberExpression(member_expr) = callee {
        let kind = CallOrApply::from_name(member_expr.property.name.as_str())?;
        return Some(ClassifiedCallee { kind, applied: member_expr.object.without_parentheses() });
    }

    let callee = as_member_expression_without_chain_expression(callee.without_parentheses())?;
    if callee.is_computed() {
        return None;
    }

    let kind = CallOrApply::from_name(callee.static_property_name()?)?;
    Some(ClassifiedCallee { kind, applied: callee.object().without_parentheses() })
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
        "(foo.call)(undefined, 1);",
        "obj.foo.call(obj, 1, 2);",
        "a.b.c.foo.call(a.b.c, 1, 2);",
        "a.b(x, y).c.foo.call(a.b(x, y).c, 1, 2);",
        "foo.apply(undefined, [1, 2]);",
        "foo.apply(void 0, [1, 2]);",
        "foo.apply(null, [1, 2]);",
        "obj.foo.apply(obj, [1, 2]);",
        "(obj.foo.apply)(obj, []);",
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
