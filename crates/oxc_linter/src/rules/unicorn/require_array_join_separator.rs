use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(require-array-join-separator): Enforce using the separator argument with Array#join()")]
#[diagnostic(severity(warning), help("Missing the separator argument."))]
struct RequireArrayJoinSeparatorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireArrayJoinSeparator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using the separator argument with Array#join()
    ///
    /// ### Why is this bad?
    ///
    /// It's better to make it clear what the separator is when calling Array#join(),
    /// instead of relying on the default comma (',') separator.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// foo.join()
    ///
    /// // Good
    /// foo.join(",")
    /// ```
    RequireArrayJoinSeparator,
    style
);

// ref: https://github.com/sindresorhus/eslint-plugin-unicorn/blob/main/rules/utils/array-or-object-prototype-property.js
fn is_prototype_property(
    member_expr: &MemberExpression,
    property: &str,
    object: Option<&str>,
) -> bool {
    if !member_expr.static_property_name().is_some_and(|name| name == property)
        || member_expr.optional()
    {
        return false;
    }

    // `Object.prototype.method` or `Array.prototype.method`
    if let Expression::MemberExpression(member_expr_obj) = member_expr.object() {
        if let Expression::Identifier(iden) = member_expr_obj.object() {
            if member_expr_obj.static_property_name().is_some_and(|name| name == "prototype")
                && object.is_some_and(|val| val == iden.name)
                && !member_expr.optional()
                && !member_expr_obj.optional()
            {
                return true;
            }
        }
    };

    match object {
        // `[].method`
        Some("Array") => {
            if let Expression::ArrayExpression(array_expr) = member_expr.object() {
                array_expr.elements.len() == 0
            } else {
                false
            }
        }

        // `{}.method`
        Some("Object") => {
            if let Expression::ObjectExpression(obj_expr) = member_expr.object() {
                obj_expr.properties.len() == 0
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_array_prototype_property(member_expr: &MemberExpression, property: &str) -> bool {
    is_prototype_property(member_expr, property, Some("Array"))
}

impl Rule for RequireArrayJoinSeparator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        // foo.join()
        if is_method_call(call_expr, None, Some(&["join"]), Some(0), Some(0))
            && !call_expr.optional
            && !matches!(member_expr, MemberExpression::ComputedMemberExpression(_))
        {
            ctx.diagnostic(RequireArrayJoinSeparatorDiagnostic(Span {
                start: member_expr.span().end,
                end: call_expr.span.end,
            }));
        }

        // `[].join.call(foo)` and `Array.prototype.join.call(foo)`
        if let Expression::MemberExpression(member_expr_obj) = member_expr.object() {
            if is_method_call(call_expr, None, Some(&["call"]), Some(1), Some(1))
                && !member_expr.optional()
                && !call_expr.optional
                && !call_expr.arguments.iter().any(oxc_ast::ast::Argument::is_spread)
                && is_array_prototype_property(member_expr_obj, "join")
            {
                ctx.diagnostic(RequireArrayJoinSeparatorDiagnostic(Span {
                    start: member_expr.span().end,
                    end: call_expr.span.end,
                }));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo.join(\",\")", None),
        (r"join()", None),
        (r"foo.join(...[])", None),
        (r"foo.join?.()", None),
        (r"foo?.join?.()", None),
        (r"foo[join]()", None),
        ("foo[\"join\"]()", None),
        ("[].join.call(foo, \",\")", None),
        (r"[].join.call()", None),
        (r"[].join.call(...[foo])", None),
        (r"[].join?.call(foo)", None),
        (r"[]?.join.call(foo)", None),
        (r"[].join[call](foo)", None),
        (r"[][join].call(foo)", None),
        (r"[,].join.call(foo)", None),
        (r"[].join.notCall(foo)", None),
        (r"[].notJoin.call(foo)", None),
        ("Array.prototype.join.call(foo, \"\")", None),
        (r"Array.prototype.join.call()", None),
        (r"Array.prototype.join.call(...[foo])", None),
        (r"Array.prototype.join?.call(foo)", None),
        (r"Array.prototype?.join.call(foo)", None),
        (r"Array?.prototype.join.call(foo)", None),
        ("Array.prototype.join[call](foo, \"\")", None),
        (r"Array.prototype[join].call(foo)", None),
        (r"Array[prototype].join.call(foo)", None),
        (r"Array.prototype.join.notCall(foo)", None),
        (r"Array.prototype.notJoin.call(foo)", None),
        (r"Array.notPrototype.join.call(foo)", None),
        (r"NotArray.prototype.join.call(foo)", None),
        ("path.join(__dirname, \"./foo.js\")", None),
    ];

    let fail = vec![
        (r"foo.join()", None),
        (r"[].join.call(foo)", None),
        (r"[].join.call(foo,)", None),
        (r"[].join.call(foo , );", None),
        (r"Array.prototype.join.call(foo)", None),
        (r"Array.prototype.join.call(foo, )", None),
        (r"foo?.join()", None),
    ];

    Tester::new(RequireArrayJoinSeparator::NAME, pass, fail).test_and_snapshot();
}
