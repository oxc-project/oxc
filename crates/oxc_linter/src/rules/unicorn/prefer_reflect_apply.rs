use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_reflect_apply_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `Reflect.apply()` over `Function#apply()`.")
        .with_help("`Reflect.apply()` is less verbose and easier to understand.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferReflectApply;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `Function.prototype.apply()` and suggests using `Reflect.apply()` instead.
    ///
    /// ### Why is this bad?
    ///
    /// `Reflect.apply()` is arguably less verbose and easier to understand.
    /// In addition, when you accept arbitrary methods,
    /// it's not safe to assume `.apply()` exists or is not overridden.
    ///
    /// ### Examples
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
    style,
    suggestion,
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
            ctx.diagnostic_with_suggestion(
                prefer_reflect_apply_diagnostic(call_expr.span),
                |fixer| {
                    create_reflect_apply_fix(
                        ctx,
                        fixer,
                        call_expr.span,
                        member_expr.object().span(),
                        call_expr.arguments[0].span(),
                        call_expr.arguments[1].span(),
                    )
                },
            );
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
                        ctx.diagnostic_with_suggestion(
                            prefer_reflect_apply_diagnostic(call_expr.span),
                            |fixer| {
                                create_reflect_apply_fix(
                                    ctx,
                                    fixer,
                                    call_expr.span,
                                    call_expr.arguments[0].span(),
                                    call_expr.arguments[1].span(),
                                    call_expr.arguments[2].span(),
                                )
                            },
                        );
                    }
                }
            }
        }
    }
}

fn create_reflect_apply_fix(
    ctx: &LintContext,
    fixer: RuleFixer,
    call_span: Span,
    func_span: Span,
    this_arg_span: Span,
    args_span: Span,
) -> RuleFix {
    let func = ctx.source_range(func_span);
    let this_arg = ctx.source_range(this_arg_span);
    let args = ctx.source_range(args_span);
    fixer.replace(call_span, format!("Reflect.apply({func}, {this_arg}, {args})"))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.apply();",
        "foo.apply(null);",
        "foo.apply(this);",
        "foo.apply(null, 42);",
        "foo.apply(this, 42);",
        "foo.apply(bar, arguments);",
        "[].apply(null, [42]);",
        "foo.apply(bar);",
        "foo.apply(bar, []);",
        "foo.apply;",
        "apply;",
        "Reflect.apply(foo, null);",
        "Reflect.apply(foo, null, [bar]);",
        r#"const apply = "apply"; foo[apply](null, [42]);"#,
    ];

    let fail = vec![
        "foo.apply(null, [42]);",
        "foo.bar.apply(null, [42]);",
        "Function.prototype.apply.call(foo, null, [42]);",
        "Function.prototype.apply.call(foo.bar, null, [42]);",
        "foo.apply(null, arguments);",
        "Function.prototype.apply.call(foo, null, arguments);",
        "foo.apply(this, [42]);",
        "Function.prototype.apply.call(foo, this, [42]);",
        "foo.apply(this, arguments);",
        "Function.prototype.apply.call(foo, this, arguments);",
        r#"foo["apply"](null, [42]);"#,
    ];

    let fix = vec![
        ("foo.apply(null, [42]);", "Reflect.apply(foo, null, [42]);"),
        ("foo.bar.apply(null, [42]);", "Reflect.apply(foo.bar, null, [42]);"),
        ("Function.prototype.apply.call(foo, null, [42]);", "Reflect.apply(foo, null, [42]);"),
        (
            "Function.prototype.apply.call(foo.bar, null, [42]);",
            "Reflect.apply(foo.bar, null, [42]);",
        ),
        ("foo.apply(null, arguments);", "Reflect.apply(foo, null, arguments);"),
        (
            "Function.prototype.apply.call(foo, null, arguments);",
            "Reflect.apply(foo, null, arguments);",
        ),
        ("foo.apply(this, [42]);", "Reflect.apply(foo, this, [42]);"),
        ("Function.prototype.apply.call(foo, this, [42]);", "Reflect.apply(foo, this, [42]);"),
        ("foo.apply(this, arguments);", "Reflect.apply(foo, this, arguments);"),
        (
            "Function.prototype.apply.call(foo, this, arguments);",
            "Reflect.apply(foo, this, arguments);",
        ),
        (r#"foo["apply"](null, [42]);"#, "Reflect.apply(foo, null, [42]);"),
    ];

    Tester::new(PreferReflectApply::NAME, PreferReflectApply::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
