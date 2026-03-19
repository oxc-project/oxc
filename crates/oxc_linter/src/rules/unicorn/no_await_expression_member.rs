use oxc_ast::{
    AstKind, MemberExpressionKind,
    ast::{BindingPattern, Expression},
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
                    if parent.type_annotation.is_some() {
                        return fixer.noop();
                    }
                    let BindingPattern::BindingIdentifier(id) = &parent.id else {
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
        "const foo = await promise",
        "const {foo: bar} = await promise",
        "const foo = !await promise",
        "const foo = typeof await promise",
        "const foo = await notPromise.method()",
        "const foo = foo[await promise]",
        // These await expression need parenthesized, but rarely used
        "new (await promiseReturnsAClass)",
        "(await promiseReturnsAFunction)()",
        // typescript
        "async function foo () {return (await promise) as string;}",
    ];

    let fail = vec![
        "(await promise)[0]",
        "(await promise).property",
        "const foo = (await promise).bar()",
        "const foo = (await promise).bar?.()",
        "const foo = (await promise)?.bar()",
        "const firstElement = (await getArray())[0]",
        "const secondElement = (await getArray())[1]",
        "const thirdElement = (await getArray())[2]",
        "const optionalFirstElement = (await getArray())?.[0]",
        "const {propertyOfFirstElement} = (await getArray())[0]",
        "const [firstElementOfFirstElement] = (await getArray())[0]",
        "let foo, firstElement = (await getArray())[0]",
        "var firstElement = (await getArray())[0], bar",
        "const property = (await getObject()).property",
        "const renamed = (await getObject()).property",
        "const property = (await getObject())[property]",
        "const property = (await getObject())?.property",
        "const {propertyOfProperty} = (await getObject()).property",
        "const {propertyOfProperty} = (await getObject()).propertyOfProperty",
        "const [firstElementOfProperty] = (await getObject()).property",
        "const [firstElementOfProperty] = (await getObject()).firstElementOfProperty",
        "firstElement = (await getArray())[0]",
        "property = (await getArray()).property",
        // typescript
        "const foo: Type = (await promise)[0]",
        "const foo: Type | A = (await promise).foo",
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
        .change_rule_path_extension("mts")
        .expect_fix(fix)
        .test_and_snapshot();
}
