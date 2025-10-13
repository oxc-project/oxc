use oxc_ast::{
    AstKind, MemberExpressionKind,
    ast::{BindingPatternKind, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_await_expression_member_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not access a member directly from an await expression.")
        .with_help("Assign the result of the await expression to a variable, then access the member from that variable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAwaitExpressionMember;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows member access from `await` expressions.
    ///
    /// ### Why is this bad?
    ///
    /// When accessing a member from an `await` expression,
    /// the `await` expression has to be parenthesized, which is not readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// async function bad() {
    ///     const secondElement = (await getArray())[1];
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function good() {
    ///     const [, secondElement] = await getArray();
    /// }
    /// ```
    NoAwaitExpressionMember,
    unicorn,
    style,
    fix_dangerous,
);

impl Rule for NoAwaitExpressionMember {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(member_expr) = node.kind().as_member_expression_kind() else {
            return;
        };

        let Expression::ParenthesizedExpression(paren_expr) = member_expr.object() else {
            return;
        };

        if matches!(paren_expr.expression, Expression::AwaitExpression(_)) {
            let node_span = member_expr.span();
            ctx.diagnostic_with_dangerous_fix(
                no_await_expression_member_diagnostic(node_span),
                |fixer| {
                    if member_expr.optional() {
                        return fixer.noop();
                    }
                    let AstKind::VariableDeclarator(parent) =
                        ctx.nodes().parent_kind(node.id())
                    else {
                        return fixer.noop();
                    };
                    if parent.id.type_annotation.is_some() {
                        return fixer.noop();
                    }
                    let BindingPatternKind::BindingIdentifier(id) = &parent.id.kind else {
                        return fixer.noop();
                    };
                    let name = id.name.as_str();
                    let inner_text = ctx.source_range(Span::new(
                        paren_expr.span.start + 1,
                        paren_expr.span.end - 1,
                    ));
                    let fixer = fixer.for_multifix();
                    let mut rule_fixes = fixer.new_fix_with_capacity(5);

                    match member_expr {
                        // e.g. "const a = (await b())[0]" => "const {a} = await b()"
                        MemberExpressionKind::Computed(computed_member_expr) => {
                            let Expression::NumericLiteral(prop) = &computed_member_expr.expression
                            else {
                                return fixer.noop();
                            };
                            let Some(value) = prop.raw.map(|v| v.as_str()) else {
                                return fixer.noop();
                            };
                            if value != "0" && value != "1" {
                                return fixer.noop();
                            }
                            // a => [a] or [, a]
                            let replacement = if value == "0" {
                                format!("[{name}]")
                            } else {
                                format!("[, {name}]")
                            };
                            rule_fixes.push(fixer.replace(id.span, replacement));
                        }
                        MemberExpressionKind::Static(static_member_expr)
                            if static_member_expr.property.name.as_str() == name =>
                        {
                            // e.g. "const a = (await b()).a" => "const {a} = await b()"
                            rule_fixes.push(fixer.replace(id.span, format!("{{{name}}}")));
                        }
                        _ => {
                            return fixer.noop();
                        }
                    }
                    // (await b())[0] => await b()
                    // (await b()).a => await b()
                    rule_fixes.push(fixer.replace(member_expr.span(), inner_text.to_owned()));
                    rule_fixes.with_message("Assign the result of the await expression to a variable, then access the member from that variable.")
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r"const foo = await promise", None),
        (r"const {foo: bar} = await promise", None),
        (r"const foo = !await promise", None),
        (r"const foo = typeof await promise", None),
        (r"const foo = await notPromise.method()", None),
        (r"const foo = foo[await promise]", None),
        // These await expression need parenthesized, but rarely used
        (r"new (await promiseReturnsAClass)", None),
        (r"(await promiseReturnsAFunction)()", None),
        // typescript
        (r"async function foo () {return (await promise) as string;}", None),
    ];

    let fail = vec![
        (r"(await promise)[0]", None),
        (r"(await promise).property", None),
        (r"const foo = (await promise).bar()", None),
        (r"const foo = (await promise).bar?.()", None),
        (r"const foo = (await promise)?.bar()", None),
        (r"const firstElement = (await getArray())[0]", None),
        (r"const secondElement = (await getArray())[1]", None),
        (r"const thirdElement = (await getArray())[2]", None),
        (r"const optionalFirstElement = (await getArray())?.[0]", None),
        (r"const {propertyOfFirstElement} = (await getArray())[0]", None),
        (r"const [firstElementOfFirstElement] = (await getArray())[0]", None),
        (r"let foo, firstElement = (await getArray())[0]", None),
        (r"var firstElement = (await getArray())[0], bar", None),
        (r"const property = (await getObject()).property", None),
        (r"const renamed = (await getObject()).property", None),
        (r"const property = (await getObject())[property]", None),
        (r"const property = (await getObject())?.property", None),
        (r"const {propertyOfProperty} = (await getObject()).property", None),
        (r"const {propertyOfProperty} = (await getObject()).propertyOfProperty", None),
        (r"const [firstElementOfProperty] = (await getObject()).property", None),
        (r"const [firstElementOfProperty] = (await getObject()).firstElementOfProperty", None),
        (r"firstElement = (await getArray())[0]", None),
        (r"property = (await getArray()).property", None),
        // typescript
        (r"const foo: Type = (await promise)[0]", None),
        (r"const foo: Type | A = (await promise).foo", None),
    ];

    let fix = vec![
        ("const a = (await b()).a", "const {a} = await b()"),
        ("const {a} = (await b()).a", "const {a} = (await b()).a"),
        ("const a = (await b()).b", "const a = (await b()).b"),
        ("const [a] = (await foo()).a", "const [a] = (await foo()).a"),
        ("const a = (await b())[0]", "const [a] = await b()"),
        ("const a = (await b())[1]", "const [, a] = await b()"),
        ("const a = (await b())[2]", "const a = (await b())[2]"),
        ("const [a] = (await b())[1]", "const [a] = (await b())[1]"),
        ("let b, a = (await f()).a", "let b, {a} = await f()"),
        ("const a = (/** comments */await b())[1]", "const [, a] = /** comments */await b()"),
        (
            "const a = (/** comments */await b() /** comments */)[1]",
            "const [, a] = /** comments */await b() /** comments */",
        ),
        ("const foo: Type = (await promise)[0]", "const foo: Type = (await promise)[0]"),
        ("const a = (await b())?.a", "const a = (await b())?.a"),
        ("const a = (await b())?.[0]", "const a = (await b())?.[0]"),
    ];

    Tester::new(NoAwaitExpressionMember::NAME, NoAwaitExpressionMember::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
