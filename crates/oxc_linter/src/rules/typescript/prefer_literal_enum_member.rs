use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn prefer_literal_enum_member_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Explicit enum value must only be a literal value (string, number, boolean, etc).",
    )
    .with_help("Require all enum members to be literal values.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferLiteralEnumMember {
    allow_bitwise_expressions: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Explicit enum value must only be a literal value (string, number, boolean, etc).
    ///
    /// ### Why is this bad?
    /// TypeScript allows the value of an enum member to be many different kinds of valid JavaScript expressions.
    /// However, because enums create their own scope whereby each enum member becomes a variable in that scope, developers are often surprised at the resultant values.
    ///
    /// ### Example
    /// ```ts
    /// const imOutside = 2;
    /// const b = 2;
    /// enum Foo {
    ///   outer = imOutside,
    ///   a = 1,
    ///   b = a,
    ///   c = b,
    /// }
    /// ```
    PreferLiteralEnumMember,
    typescript,
    restriction
);

impl Rule for PreferLiteralEnumMember {
    fn from_configuration(value: serde_json::Value) -> Self {
        let options: Option<&serde_json::Value> = value.get(0);

        Self {
            allow_bitwise_expressions: options
                .and_then(|x| x.get("allowBitwiseExpressions"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumMember(decl) = node.kind() else {
            return;
        };
        let Some(initializer) = &decl.initializer else {
            return;
        };
        if initializer.is_literal() {
            return;
        }

        if let Expression::TemplateLiteral(template) = initializer {
            if template.expressions.len() == 0 {
                return;
            }
        }

        if let Expression::UnaryExpression(unary_expr) = initializer {
            if unary_expr.argument.is_literal() {
                if matches!(
                    unary_expr.operator,
                    UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation,
                ) {
                    return;
                }

                if self.allow_bitwise_expressions
                    && matches!(unary_expr.operator, UnaryOperator::BitwiseNot)
                {
                    return;
                }
            }
        }

        if self.allow_bitwise_expressions {
            if let Expression::BinaryExpression(binary_expr) = initializer {
                if matches!(
                    binary_expr.operator,
                    BinaryOperator::BitwiseOR
                        | BinaryOperator::BitwiseAnd
                        | BinaryOperator::BitwiseXOR
                        | BinaryOperator::ShiftLeft
                        | BinaryOperator::ShiftRight
                        | BinaryOperator::ShiftRightZeroFill
                ) && binary_expr.left.is_literal()
                    && binary_expr.right.is_literal()
                {
                    return;
                }
            }
        }

        ctx.diagnostic(prefer_literal_enum_member_diagnostic(decl.span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
        	enum ValidRegex {
                A = /test/,
            }
        	    ",
            None,
        ),
        (
            "
        	enum ValidString {
        	  A = 'test',
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidLiteral {
        	  A = `test`,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidNumber {
        	  A = 42,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidNumber {
        	  A = -42,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidNumber {
        	  A = +42,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidNull {
        	  A = null,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidPlain {
        	  A,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidQuotedKey {
        	  'a',
        	}
        	    ",
            None,
        ),
        (
            "
        	enum ValidQuotedKeyWithAssignment {
        	  'a' = 1,
        	}
        	    ",
            None,
        ),
        (
            "
        	enum Foo {
        	  A = 1 << 0,
        	  B = 1 >> 0,
        	  C = 1 >>> 0,
        	  D = 1 | 0,
        	  E = 1 & 0,
        	  F = 1 ^ 0,
        	  G = ~1,
        	}
        	      ",
            Some(serde_json::json!([{ "allowBitwiseExpressions": true }])),
        ),
    ];

    let fail = vec![
        (
            "
        	enum InvalidObject {
        	  A = {},
        	}
        	      ",
            None,
        ),
        (
            "
        	enum InvalidArray {
        	  A = [],
        	}
        	      ",
            None,
        ),
        (
            "
        	enum InvalidTemplateLiteral {
        	  A = `foo ${0}`,
        	}
        	      ",
            None,
        ),
        (
            "
        	enum InvalidConstructor {
        	  A = new Set(),
        	}
        	      ",
            None,
        ),
        (
            "
        	enum InvalidExpression {
        	  A = 2 + 2,
        	}
        	      ",
            None,
        ),
        (
            "
        	enum InvalidExpression {
        	  A = delete 2,
        	  B = -a,
        	  C = void 2,
        	  D = ~2,
        	  E = !0,
        	}
        	      ",
            None,
        ),
        (
            "
        	const variable = 'Test';
        	enum InvalidVariable {
        	  A = 'TestStr',
        	  B = 2,
        	  C,
        	  V = variable,
        	}
        	      ",
            None,
        ),
        (
            "
        	enum InvalidEnumMember {
        	  A = 'TestStr',
        	  B = A,
        	}
        	      ",
            None,
        ),
        (
            "
        	const Valid = { A: 2 };
        	enum InvalidObjectMember {
        	  A = 'TestStr',
        	  B = Valid.A,
        	}
        	      ",
            None,
        ),
        (
            "
        	enum Valid {
        	  A,
        	}
        	enum InvalidEnumMember {
        	  A = 'TestStr',
        	  B = Valid.A,
        	}
        	      ",
            None,
        ),
        (
            "
        	const obj = { a: 1 };
        	enum InvalidSpread {
        	  A = 'TestStr',
        	  B = { ...a },
        	}
        	      ",
            None,
        ),
        (
            "
        	enum Foo {
        	  A = 1 << 0,
        	  B = 1 >> 0,
        	  C = 1 >>> 0,
        	  D = 1 | 0,
        	  E = 1 & 0,
        	  F = 1 ^ 0,
        	  G = ~1,
        	}
        	      ",
            Some(serde_json::json!([{ "allowBitwiseExpressions": false }])),
        ),
        (
            "
        	const x = 1;
        	enum Foo {
        	  A = x << 0,
        	  B = x >> 0,
        	  C = x >>> 0,
        	  D = x | 0,
        	  E = x & 0,
        	  F = x ^ 0,
        	  G = ~x,
        	}
        	      ",
            Some(serde_json::json!([{ "allowBitwiseExpressions": true }])),
        ),
    ];

    Tester::new(PreferLiteralEnumMember::NAME, PreferLiteralEnumMember::PLUGIN, pass, fail)
        .test_and_snapshot();
}
