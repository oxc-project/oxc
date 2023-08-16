use oxc_ast::ast::{Expression, TSLiteral, TSType};
use oxc_ast::AstKind;

use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Expected a `const` assertion instead of a literal type annotation.")]
#[diagnostic(severity(warning), help("You should use `as const` instead of type annotation."))]
struct PreferAsConstDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferAsConst;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce the use of as const over literal type.
    /// ### Why is this bad?
    /// There are two common ways to tell TypeScript that a literal value should be interpreted as its literal type (e.g. 2) rather than general primitive type (e.g. number);
    ///
    /// as const: telling TypeScript to infer the literal type automatically
    /// as with the literal type: explicitly telling the literal type to TypeScript
    ///
    /// as const is generally preferred, as it doesn't require re-typing the literal value.
    /// This rule reports when an as with an explicit literal type can be replaced with an as const.
    ///
    /// ### Example
    /// ```javascript
    /// let bar: 2 = 2;
    /// let foo = { bar: 'baz' as 'baz' };
    /// ```
    PreferAsConst,
    correctness
);

impl Rule for PreferAsConst {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_declarator) => {
                let Some(type_annotation) = &variable_declarator.id.type_annotation else { return; };
                let Some(initial_value_expression) = &variable_declarator.init else { return; };
                check_and_report(
                    &type_annotation.type_annotation,
                    initial_value_expression,
                    ctx,
                    false,
                );
            }
            AstKind::PropertyDefinition(property_definition) => {
                let Some(type_annotation) = &property_definition.type_annotation else { return; };
                let Some(initial_value_expression) = &property_definition.value else { return; };
                check_and_report(
                    &type_annotation.type_annotation,
                    initial_value_expression,
                    ctx,
                    false,
                );
            }
            AstKind::TSAsExpression(as_expression) => {
                check_and_report(
                    &as_expression.type_annotation,
                    &as_expression.expression,
                    ctx,
                    true,
                );
            }
            _ => {}
        }
    }
}

fn check_and_report(
    ts_type: &TSType,
    initial_value_expression: &Expression,
    ctx: &LintContext,
    can_fix: bool,
) {
    if let TSType::TSLiteralType(literal_type) = &ts_type {
        let error_span = match &literal_type.literal {
            TSLiteral::StringLiteral(string_literal) => match initial_value_expression {
                Expression::StringLiteral(initial_string) => {
                    if string_literal.value.eq(&initial_string.value) {
                        Some(string_literal.span)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            TSLiteral::NullLiteral(null_literal) => match initial_value_expression {
                Expression::NullLiteral(_) => Some(null_literal.span),
                _ => None,
            },
            TSLiteral::NumberLiteral(number_literal) => match initial_value_expression {
                Expression::NumberLiteral(initial_number) => {
                    if (number_literal.value - initial_number.value).abs() < f64::EPSILON {
                        Some(number_literal.span)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(span) = error_span {
            if can_fix {
                ctx.diagnostic_with_fix(PreferAsConstDiagnostic(span), || {
                    let start = span.start;
                    let end = span.end;
                    Fix::new("const", Span { start, end })
                });
            } else {
                ctx.diagnostic(PreferAsConstDiagnostic(span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let foo = 'baz' as const;",
        "let foo = 1 as const;",
        "let foo = { bar: 'baz' as const };",
        "let foo = { bar: 1 as const };",
        "let foo = { bar: 'baz' };",
        "let foo = { bar: 2 };",
        "let foo = 'bar' as string;",
        "let foo = `bar` as `bar`;",
        "let foo = `bar` as `foo`;",
        "let foo = `bar` as 'bar';",
        "let foo: string = 'bar';",
        "let foo: number = 1;",
        "let foo: 'bar' = baz;",
        "let foo = 'bar';",
        "let foo: 'bar';",
        "let foo = { bar };",
        "let foo: 'baz' = 'baz' as const;",
        "class foo { bar = 'baz'; }",
        "class foo { bar: 'baz'; }",
        "class foo { bar; }",
        "class foo { bar: string = 'baz'; }",
        "class foo { bar: number = 1; }",
        "class foo { bar = 'baz' as const; }",
        "class foo { bar = 2 as const; }",
        "class foo { get bar(): 'bar' {} set bar(bar: 'bar') {} }",
        "class foo { bar = () => 'bar' as const; }",
        "type BazFunction = () => 'baz'; class foo { bar: BazFunction = () => 'bar'; }",
        "class foo { bar(): void {} }",
        // NOTE: OXC does not parse these format yet.
        // "let foo = <bar>'bar';",
        // "let foo = <string>'bar';",
        // "class foo { bar = <baz>'baz'; }",
    ];

    let fail = vec![
        "let []: 'bar' = 'bar';",
        "let foo: 'bar' = 'bar';",
        "let foo: 2 = 2;",
        "class foo { bar: 'baz' = 'baz';}",
        "class foo { bar: 2 = 2;}",
    ];

    let fix = vec![
        ("let foo = { bar: 'baz' as 'baz' };", "let foo = { bar: 'baz' as const };", None),
        ("let foo = { bar: 1 as 1 };", "let foo = { bar: 1 as const };", None),
        ("let foo: 'bar' = 'bar' as 'bar';", "let foo: 'bar' = 'bar' as const;", None),
        ("let foo = 'bar' as 'bar';", "let foo = 'bar' as const;", None),
        ("let foo = 5 as 5;", "let foo = 5 as const;", None),
        ("class foo { foo = 'bar' as 'bar'; }", "class foo { foo = 'bar' as const; }", None),
        ("class foo { foo = 5 as 5; }", "class foo { foo = 5 as const; }", None),
        // NOTE: OXC does not parse these format yet.
        // ("let foo = <4>4;", "let foo = <const>4;", None),
        // ("let foo = <'bar'>'bar';", "let foo = <const>'bar';", None),
        // ("class foo { foo = <'bar'>'bar'; }", "class foo { foo = <const>'bar'; }", None),
    ];

    let mut tester = Tester::new_without_config(PreferAsConst::NAME, pass, fail);
    tester.test_and_snapshot();
    tester.test_fix(fix);
}
