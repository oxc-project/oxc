use oxc_ast::{
    ast::{Argument, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_reflect_apply_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer Reflect.apply() over Function#apply()")
        .with_help("Reflect.apply() is less verbose and easier to understand.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferReflectApply;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    /// Reflect.apply() is arguably less verbose and easier to understand.
    /// In addition, when you accept arbitrary methods,
    /// it's not safe to assume .apply() exists or is not overridden.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.apply(null, [42]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// Reflect.apply(foo, null);
    /// ```
    PreferReflectApply,
    unicorn,
    style
);

fn is_apply_signature(first_arg: &Argument, second_arg: &Argument) -> bool {
    match first_arg {
        Argument::ThisExpression(_) | Argument::NullLiteral(_) => {
            matches!(second_arg, Argument::ArrayExpression(_))
                || matches!(second_arg, Argument::Identifier(ident) if ident.name == "arguments")
        }
        _ => false,
    }
}

fn is_static_property_name_equal(expr: &MemberExpression, value: &str) -> bool {
    expr.static_property_name().is_some_and(|name| name == value)
}

impl Rule for PreferReflectApply {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if call_expr.optional
            || matches!(
                member_expr.object(),
                Expression::ArrayExpression(_) | Expression::ObjectExpression(_)
            )
            || member_expr.object().is_literal()
        {
            return;
        }

        if is_static_property_name_equal(member_expr, "apply")
            && matches!(call_expr.arguments.as_slice(), [first, second] if is_apply_signature(first, second))
        {
            ctx.diagnostic(prefer_reflect_apply_diagnostic(call_expr.span));
            return;
        }

        if is_static_property_name_equal(member_expr, "call") {
            let Some(member_expr_obj) = member_expr.object().as_member_expression() else {
                return;
            };
            if is_static_property_name_equal(member_expr_obj, "apply") {
                let Some(member_expr_obj_obj) = member_expr_obj.object().as_member_expression()
                else {
                    return;
                };

                if is_static_property_name_equal(member_expr_obj_obj, "prototype") {
                    let Expression::Identifier(iden) = member_expr_obj_obj.object() else {
                        return;
                    };
                    if iden.name == "Function"
                        && matches!(call_expr.arguments.as_slice(), [_, second, third] if is_apply_signature(second, third))
                    {
                        ctx.diagnostic(prefer_reflect_apply_diagnostic(call_expr.span));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo.apply();", None),
        ("foo.apply(null);", None),
        ("foo.apply(this);", None),
        ("foo.apply(null, 42);", None),
        ("foo.apply(this, 42);", None),
        ("foo.apply(bar, arguments);", None),
        ("[].apply(null, [42]);", None),
        ("foo.apply(bar);", None),
        ("foo.apply(bar, []);", None),
        ("foo.apply;", None),
        ("apply;", None),
        ("Reflect.apply(foo, null);", None),
        ("Reflect.apply(foo, null, [bar]);", None),
        ("const apply = \"apply\"; foo[apply](null, [42]);", None),
    ];

    let fail = vec![
        ("foo.apply(null, [42]);", None),
        ("foo.bar.apply(null, [42]);", None),
        ("Function.prototype.apply.call(foo, null, [42]);", None),
        ("Function.prototype.apply.call(foo.bar, null, [42]);", None),
        ("foo.apply(null, arguments);", None),
        ("Function.prototype.apply.call(foo, null, arguments);", None),
        ("foo.apply(this, [42]);", None),
        ("Function.prototype.apply.call(foo, this, [42]);", None),
        ("foo.apply(this, arguments);", None),
        ("Function.prototype.apply.call(foo, this, arguments);", None),
        ("foo[\"apply\"](null, [42]);", None),
    ];

    Tester::new(PreferReflectApply::NAME, PreferReflectApply::PLUGIN, pass, fail)
        .test_and_snapshot();
}
