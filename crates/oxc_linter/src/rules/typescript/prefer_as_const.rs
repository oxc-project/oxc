use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, TSLiteral, TSType, TSTypeAnnotation},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::pad_fix_with_token_boundary,
};

fn literal_type_annotation_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a `const` assertion instead of a literal type annotation.")
        .with_help("You should use `as const` instead of type annotation.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferAsConst;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the use of `as const` over literal types.
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
    /// This rule reports when an explicit literal type can be replaced with a const assertion,
    /// including angle-bracket assertions in `.ts` files.
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
    recommended,
    conditional_fix_suggestion,
    version = "0.0.8",
    short_description = "Enforce the use of `as const` over literal types.",
);

impl Rule for PreferAsConst {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSLiteralType(literal_type) = node.kind() else { return };
        let Some(parent) = ctx
            .nodes()
            .ancestors(node.id())
            .find(|ancestor| !matches!(ancestor.kind(), AstKind::TSParenthesizedType(_)))
        else {
            return;
        };
        let owner = match parent.kind() {
            AstKind::TSTypeAnnotation(_) => ctx.nodes().parent_kind(parent.id()),
            kind => kind,
        };
        match owner {
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
                    property_definition.decorators.is_empty(),
                    type_annotation,
                    initial_value_expression,
                    ctx,
                );
            }
            AstKind::TSAsExpression(as_expression) => {
                if literal_type_matches(&as_expression.type_annotation, &as_expression.expression) {
                    report_assertion(as_expression.type_annotation.span(), ctx);
                }
            }
            AstKind::TSTypeAssertion(type_assertion) => {
                let expression = type_assertion.expression.without_parentheses();
                let source_text = ctx.source_text();
                if expression.is_literal()
                    && expression.span().source_text(source_text)
                        == literal_type.span.source_text(source_text)
                {
                    report_assertion(type_assertion.type_annotation.span(), ctx);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[expect(clippy::float_cmp, reason = "numeric literal types only match identical values")]
fn literal_type_matches(ts_type: &TSType, initial_value_expression: &Expression) -> bool {
    let TSType::TSLiteralType(literal_type) = ts_type.without_parenthesized() else {
        return false;
    };
    match (&literal_type.literal, initial_value_expression.without_parentheses()) {
        (TSLiteral::StringLiteral(literal), Expression::StringLiteral(value)) => {
            literal.value == value.value
        }
        (TSLiteral::NumericLiteral(literal), Expression::NumericLiteral(value)) => {
            literal.value == value.value
        }
        (TSLiteral::BooleanLiteral(literal), Expression::BooleanLiteral(value)) => {
            literal.value == value.value
        }
        (TSLiteral::BigIntLiteral(literal), Expression::BigIntLiteral(value)) => {
            literal.value == value.value
        }
        _ => false,
    }
}

fn report_assertion(span: Span, ctx: &LintContext) {
    ctx.diagnostic_with_fix(
        OxcDiagnostic::warn("Expected a `const` instead of a literal type assertion.")
            .with_label(span),
        |fixer| {
            let mut replacement = String::from("const");
            pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
            fixer.replace(span, replacement)
        },
    );
}

fn check_and_report_type_annotation(
    can_fix: bool,
    type_annotation: &TSTypeAnnotation<'_>,
    initial_value_expression: &Expression,
    ctx: &LintContext,
) {
    if !literal_type_matches(&type_annotation.type_annotation, initial_value_expression) {
        return;
    }

    let fix_kind = if can_fix { FixKind::SafeFix } else { FixKind::Suggestion };
    ctx.diagnostic_with_fix_of_kind(
        literal_type_annotation_diagnostic(type_annotation.type_annotation.span()),
        fix_kind,
        |fixer| {
            let fixer = fixer.for_multifix();
            let mut fix = fixer.new_fix_with_capacity(2);
            fix.push(fixer.delete(type_annotation));
            fix.push(fixer.insert_text_after(initial_value_expression, " as const"));
            fix.with_message("Use `as const` instead of a literal type annotation.")
        },
    );
}

#[test]
fn test() {
    use crate::{fixer::FixKind, tester::Tester};

    let pass = vec![
        "let foo = 'baz' as const;",
        "let foo = 1 as const;",
        "let foo = { bar: 'baz' as const };",
        "let foo = { bar: 1 as const };",
        "let foo = { bar: 'baz' };",
        "let foo = { bar: 2 };",
        "let foo = <bar>'bar';",
        "let foo = <string>'bar';",
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
        "let foo: 'bar' | 'baz' = 'bar';",
        "let foo = 'bar' as ('bar' | 'baz');",
        "function foo(value: 'bar' = 'bar') {}",
        "class Foo { accessor value: 'bar' = 'bar'; }",
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
        "
                  class foo {
                    bar = <baz>'baz';
                  }
                ",
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
        "let foo = <'bar'>'baz';",
        "let foo = <4>5;",
        "let foo = <const>'bar';",
        r#"let foo = <'bar'>"bar";"#,
        "let foo = <4>4.0;",
        "let foo = <-4>-4;",
        "let foo = 0 as 1e-16;",
        "let foo = true as false;",
        "let foo = 1n as 2n;",
        "let foo: 1n = 2n;",
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
        r#"export const x = <"baz">"baz";"#,
        "let foo = <true>true;",
        "let foo = <4n>4n;",
        "let foo = <'bar'>('bar');",
        "let a = (1) as 1;",
        "let b = 1 as (1);",
        "let c = <('bar')>'bar';",
        "let foo = true as true;",
        "let foo = 1n as 1n;",
        "let foo: 1n = 1n;",
        "let foo: (1) = 1;",
        "class C { @dec x: 1 = 1; }",
    ];

    let fix = vec![
        (
            "let a: 'a' = 'a', b: 'b' = 'b';",
            "let a = 'a' as const, b = 'b' as const;",
            None,
            FixKind::SafeFix,
        ),
        (
            "let foo = { bar: 'baz' as 'baz' };",
            "let foo = { bar: 'baz' as const };",
            None,
            FixKind::SafeFix,
        ),
        ("let foo = { bar: 1 as 1 };", "let foo = { bar: 1 as const };", None, FixKind::SafeFix),
        (
            "let foo: 'bar' = 'bar' as 'bar';",
            "let foo: 'bar' = 'bar' as const;",
            None,
            FixKind::SafeFix,
        ),
        ("let foo: 'bar' = 'bar';", "let foo = 'bar' as const;", None, FixKind::SafeFix),
        ("let foo: 2 = 2;", "let foo = 2 as const;", None, FixKind::SafeFix),
        (
            "const example: 'hello' = 'hello';",
            "const example = 'hello' as const;",
            None,
            FixKind::SafeFix,
        ),
        (r#"let foo: 'bar' = "bar";"#, r#"let foo = "bar" as const;"#, None, FixKind::SafeFix),
        ("const foo: 2 = 2;", "const foo = 2 as const;", None, FixKind::SafeFix),
        ("let foo = <'bar'>'bar';", "let foo = <const>'bar';", None, FixKind::SafeFix),
        ("let foo = <4>4;", "let foo = <const>4;", None, FixKind::SafeFix),
        ("let foo = 'bar' as 'bar';", "let foo = 'bar' as const;", None, FixKind::SafeFix),
        ("let foo = 5 as 5;", "let foo = 5 as const;", None, FixKind::SafeFix),
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
            None,
            FixKind::SafeFix,
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
            None,
            FixKind::SafeFix,
        ),
        (
            "
            class foo {
              foo = <'bar'>'bar';
            }
                  ",
            "
            class foo {
              foo = <const>'bar';
            }
                  ",
            None,
            FixKind::SafeFix,
        ),
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
            None,
            FixKind::SafeFix,
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
            None,
            FixKind::SafeFix,
        ),
        (
            r#"export const x = <"baz">"baz";"#,
            r#"export const x = <const>"baz";"#,
            None,
            FixKind::SafeFix,
        ),
        ("let foo = <true>true;", "let foo = <const>true;", None, FixKind::SafeFix),
        ("let foo = <4n>4n;", "let foo = <const>4n;", None, FixKind::SafeFix),
        ("let foo = <'bar'>('bar');", "let foo = <const>('bar');", None, FixKind::SafeFix),
        ("let s = 'x' as\"x\";", "let s = 'x' as const;", None, FixKind::SafeFix),
        ("let s = 'x' as'x'in o;", "let s = 'x' as const in o;", None, FixKind::SafeFix),
        ("let a = (1) as 1;", "let a = (1) as const;", None, FixKind::SafeFix),
        ("let b = 1 as (1);", "let b = 1 as const;", None, FixKind::SafeFix),
        ("let c = <('bar')>'bar';", "let c = <const>'bar';", None, FixKind::SafeFix),
        ("let foo: (1) = 1;", "let foo = 1 as const;", None, FixKind::SafeFix),
        ("let foo: 1 = (1);", "let foo = (1) as const;", None, FixKind::SafeFix),
        ("let foo = true as true;", "let foo = true as const;", None, FixKind::SafeFix),
        ("let foo: 1n = 1n;", "let foo = 1n as const;", None, FixKind::SafeFix),
        ("class C { @dec x: 1 = 1; }", "class C { @dec x: 1 = 1; }", None, FixKind::SafeFix),
        (
            "class C { @dec x: 1 = 1; }",
            "class C { @dec x = 1 as const; }",
            None,
            FixKind::Suggestion,
        ),
        ("let []: 'bar' = 'bar';", "let [] = 'bar' as const;", None, FixKind::Suggestion),
    ];

    Tester::new(PreferAsConst::NAME, PreferAsConst::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .expect_fix(fix)
        .test_and_snapshot();
}
