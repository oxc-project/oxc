use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, TSLiteral, TSType, TSTypeAnnotation},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn prefer_as_const_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a `const` assertion instead of a literal type annotation.")
        .with_help("You should use `as const` instead of type annotation.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferAsConst;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the use of `as const` over literal type.
    ///
    /// ### Why is this bad?
    ///
    /// There are two common ways to tell TypeScript that a literal value should be interpreted as
    /// its literal type (e.g. `2`) rather than general primitive type (e.g. `number`);
    ///
    /// `as const`: telling TypeScript to infer the literal type automatically
    /// `as` with the literal type: explicitly telling the literal type to TypeScript
    ///
    /// `as const` is generally preferred, as it doesn't require re-typing the literal value.
    /// This rule reports when an `as` with an explicit literal type can be replaced with an `as const`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// let bar: 2 = 2;
    /// let foo = { bar: 'baz' as 'baz' };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// let foo = 'bar';
    /// let foo = 'bar' as const;
    /// let foo: 'bar' = 'bar' as const;
    /// let bar = 'bar' as string;
    /// let foo = { bar: 'baz' };
    /// ```
    PreferAsConst,
    typescript,
    correctness,
    conditional_fix
);

impl Rule for PreferAsConst {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_declarator) => {
                let Some(type_annotation) = &variable_declarator.type_annotation else {
                    return;
                };
                let Some(initial_value_expression) = &variable_declarator.init else {
                    return;
                };
                check_and_report_type_annotation(
                    matches!(&variable_declarator.id, BindingPattern::BindingIdentifier(_)),
                    type_annotation,
                    &type_annotation.type_annotation,
                    initial_value_expression,
                    ctx,
                );
            }
            AstKind::PropertyDefinition(property_definition) => {
                let Some(type_annotation) = &property_definition.type_annotation else {
                    return;
                };
                let Some(initial_value_expression) = &property_definition.value else {
                    return;
                };
                check_and_report_type_annotation(
                    true,
                    type_annotation,
                    &type_annotation.type_annotation,
                    initial_value_expression,
                    ctx,
                );
            }
            AstKind::TSAsExpression(as_expression) => {
                check_and_report_as_expression(
                    &as_expression.type_annotation,
                    &as_expression.expression,
                    ctx,
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn literal_type_span_if_matches(
    ts_type: &TSType,
    initial_value_expression: &Expression,
) -> Option<Span> {
    let TSType::TSLiteralType(literal_type) = &ts_type else { return None };
    match &literal_type.literal {
        TSLiteral::StringLiteral(string_literal) => match initial_value_expression {
            Expression::StringLiteral(initial_string) => {
                string_literal.value.eq(&initial_string.value).then_some(string_literal.span)
            }
            _ => None,
        },
        TSLiteral::NumericLiteral(number_literal) => match initial_value_expression {
            Expression::NumericLiteral(initial_number) => {
                ((number_literal.value - initial_number.value).abs() < f64::EPSILON)
                    .then_some(number_literal.span)
            }
            _ => None,
        },
        _ => None,
    }
}

fn check_and_report_as_expression(
    ts_type: &TSType,
    initial_value_expression: &Expression,
    ctx: &LintContext,
) {
    let Some(span) = literal_type_span_if_matches(ts_type, initial_value_expression) else {
        return;
    };
    ctx.diagnostic_with_fix(prefer_as_const_diagnostic(span), |fixer| fixer.replace(span, "const"));
}

fn check_and_report_type_annotation(
    can_fix: bool,
    type_annotation: &TSTypeAnnotation<'_>,
    ts_type: &TSType,
    initial_value_expression: &Expression,
    ctx: &LintContext,
) {
    let Some(span) = literal_type_span_if_matches(ts_type, initial_value_expression) else {
        return;
    };

    if can_fix {
        ctx.diagnostic_with_fix(prefer_as_const_diagnostic(span), |fixer| {
            let fixer = fixer.for_multifix();
            let mut fix = fixer.new_fix_with_capacity(2);
            fix.push(fixer.delete(type_annotation));
            fix.push(fixer.insert_text_after(initial_value_expression, " as const"));
            fix.with_message("Use `as const` instead of a literal type annotation.")
        });
    } else {
        ctx.diagnostic(prefer_as_const_diagnostic(span));
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
        // "let foo = <bar>'bar';",
        // "let foo = <string>'bar';",
        "let foo = 'bar' as string;",
        "let foo = `bar` as `bar`;",
        "let foo = `bar` as `foo`;",
        "let foo = `bar` as 'bar';",
        "let foo: string = 'bar';",
        "let foo: number = 1;",
        "let foo: 'bar' = baz;",
        "let foo: 'bar' = 'baz';",
        "let foo: 2 = 3;",
        "let foo = 'bar';",
        "let foo: 'bar';",
        "let foo = { bar };",
        "let foo: 'baz' = 'baz' as const;",
        "
                  class foo {
                    bar = 'baz';
                  }
                ",
        "
                  class foo {
                    bar: 'baz';
                  }
                ",
        "
                  class foo {
                    bar;
                  }
                ",
        // "
        //           class foo {
        //             bar = <baz>'baz';
        //           }
        //         ",
        "
                  class foo {
                    bar: string = 'baz';
                  }
                ",
        "
                  class foo {
                    bar: number = 1;
                  }
                ",
        "
                  class foo {
                    bar = 'baz' as const;
                  }
                ",
        "
                  class foo {
                    bar = 2 as const;
                  }
                ",
        "
                  class foo {
                    get bar(): 'bar' {}
                    set bar(bar: 'bar') {}
                  }
                ",
        "
                  class foo {
                    bar = () => 'bar' as const;
                  }
                ",
        "
                  type BazFunction = () => 'baz';
                  class foo {
                    bar: BazFunction = () => 'bar';
                  }
                ",
        "
                  class foo {
                    bar(): void {}
                  }
                ",
    ];

    let fail = vec![
        "let foo = { bar: 'baz' as 'baz' };",
        "let foo = { bar: 1 as 1 };",
        "let []: 'bar' = 'bar';",
        "let foo: 'bar' = 'bar';",
        "let foo: 2 = 2;",
        "const example: 'hello' = 'hello';",
        r#"let foo: 'bar' = "bar";"#,
        "const foo: 2 = 2;",
        "
            class foo {
              readonly bar: 'baz' = 'baz';
            }
                  ",
        "
            class foo {
              static bar: 2 = 2;
            }
                  ",
        "let foo: 'bar' = 'bar' as 'bar';",
        "let foo = <'bar'>'bar';",
        "let foo = <4>4;",
        "let foo = 'bar' as 'bar';",
        "let foo = 5 as 5;",
        "
            class foo {
              bar: 'baz' = 'baz';
            }
                  ",
        "
            class foo {
              bar: 2 = 2;
            }
                  ",
        "
            class foo {
              foo = <'bar'>'bar';
            }
                  ",
        "
            class foo {
              foo = 'bar' as 'bar';
            }
                  ",
        "
            class foo {
              foo = 5 as 5;
            }
                  ",
    ];

    let fix = vec![
        ("let foo = { bar: 'baz' as 'baz' };", "let foo = { bar: 'baz' as const };"),
        ("let foo = { bar: 1 as 1 };", "let foo = { bar: 1 as const };"),
        ("let foo: 'bar' = 'bar' as 'bar';", "let foo: 'bar' = 'bar' as const;"),
        ("let foo: 'bar' = 'bar';", "let foo = 'bar' as const;"),
        ("let foo: 2 = 2;", "let foo = 2 as const;"),
        ("const example: 'hello' = 'hello';", "const example = 'hello' as const;"),
        (r#"let foo: 'bar' = "bar";"#, r#"let foo = "bar" as const;"#),
        ("const foo: 2 = 2;", "const foo = 2 as const;"),
        // ("let foo = <'bar'>'bar';", "let foo = <const>'bar';"),
        // ("let foo = <4>4;", "let foo = <const>4;"),
        ("let foo = 'bar' as 'bar';", "let foo = 'bar' as const;"),
        ("let foo = 5 as 5;", "let foo = 5 as const;"),
        (
            "
            class foo {
              readonly bar: 'baz' = 'baz';
            }
                  ",
            "
            class foo {
              readonly bar = 'baz' as const;
            }
                  ",
        ),
        (
            "
            class foo {
              static bar: 2 = 2;
            }
                  ",
            "
            class foo {
              static bar = 2 as const;
            }
                  ",
        ),
        // (
        //     "
        //     class foo {
        //       foo = <'bar'>'bar';
        //     }
        //           ",
        //     "
        //     class foo {
        //       foo = <const>'bar';
        //     }
        //           ",
        // ),
        (
            "
            class foo {
              foo = 'bar' as 'bar';
            }
                  ",
            "
            class foo {
              foo = 'bar' as const;
            }
                  ",
        ),
        (
            "
            class foo {
              foo = 5 as 5;
            }
                  ",
            "
            class foo {
              foo = 5 as const;
            }
                  ",
        ),
    ];

    Tester::new(PreferAsConst::NAME, PreferAsConst::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
