use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};

use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};

fn bad_bitwise_operator_diagnostic(
    bad_operator: &str,
    suggestion: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Bad bitwise operator")
        .with_help(format!(
            "Bitwise operator '{bad_operator}' seems unintended. Did you mean logical operator '{suggestion}'?"
        ))
        .with_label(span)
}

fn bad_bitwise_or_operator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Bad bitwise operator")
        .with_help("Bitwise operator '|=' seems unintended. Did you mean logical operator '||='?")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BadBitwiseOperator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule applies when bitwise operators are used where logical operators are expected.
    ///
    /// ### Why is this bad?
    ///
    /// Bitwise operators have different results from logical operators and a `TypeError` exception may be thrown because short-circuit evaluation is not applied.
    /// (In short-circuit evaluation, right operand evaluation is skipped according to left operand value, e.g. `x` is `false` in `x && y`.)
    ///
    /// It is obvious that logical operators are expected in the following code patterns:
    /// ```javascript
    /// e && e.x
    /// e || {}
    /// e || ''
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (obj & obj.prop) {
    ///  console.log(obj.prop);
    /// }
    /// options = options | {};
    /// input |= '';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (obj && obj.prop) {
    ///  console.log(obj.prop);
    /// }
    /// options = options || {};
    /// input ||= '';
    /// ```
    BadBitwiseOperator,
    oxc,
    restriction, // Restricted because there are false positives for enum bitflags in TypeScript, e.g. in the vscode repo
    suggestion,
    version = "0.0.3",
);

impl Rule for BadBitwiseOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                if is_mistype_short_circuit(node) {
                    let start = bin_expr.left.span().end;
                    ctx.diagnostic_with_suggestion(
                        bad_bitwise_operator_diagnostic("&", "&&", bin_expr.span),
                        |fixer| Self::fix_binary_operator(fixer, start, "&", "&&", ctx),
                    );
                } else if is_mistype_option_fallback(node) {
                    let start = bin_expr.left.span().end;
                    ctx.diagnostic_with_suggestion(
                        bad_bitwise_operator_diagnostic("|", "||", bin_expr.span),
                        |fixer| Self::fix_binary_operator(fixer, start, "|", "||", ctx),
                    );
                }
            }
            AstKind::AssignmentExpression(assign_expr)
                if assign_expr.operator == AssignmentOperator::BitwiseOR
                    && !is_numeric_expr(&assign_expr.right, true) =>
            {
                let start = assign_expr.left.span().end;
                ctx.diagnostic_with_suggestion(
                    bad_bitwise_or_operator_diagnostic(assign_expr.span),
                    |fixer| Self::fix_assignment_operator(fixer, start, ctx),
                );
            }
            _ => {}
        }
    }
}

impl BadBitwiseOperator {
    #[expect(clippy::cast_possible_truncation)]
    fn fix_binary_operator(
        fixer: RuleFixer<'_, '_>,
        start: u32,
        bad: &str,
        good: &'static str,
        ctx: &LintContext<'_>,
    ) -> crate::fixer::RuleFix {
        let Some(offset) = ctx.find_next_token_from(start, bad) else {
            return fixer.noop();
        };
        let op_start = start + offset;
        let op_span = Span::new(op_start, op_start + bad.len() as u32);
        fixer.replace(op_span, good)
    }

    fn fix_assignment_operator(
        fixer: RuleFixer<'_, '_>,
        start: u32,
        ctx: &LintContext<'_>,
    ) -> crate::fixer::RuleFix {
        let Some(offset) = ctx.find_next_token_from(start, "|=") else {
            return fixer.noop();
        };
        let op_start = start + offset;
        let op_span = Span::new(op_start, op_start + 2);
        fixer.replace(op_span, "||=")
    }
}

fn is_mistype_short_circuit(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::BinaryExpression(bin_expr) => {
            if bin_expr.operator != BinaryOperator::BitwiseAnd {
                return false;
            }

            let Expression::Identifier(left_ident) = &bin_expr.left else {
                return false;
            };

            if let Some(member_expr) = bin_expr.right.as_member_expression()
                && let Expression::Identifier(ident) = member_expr.object()
            {
                return ident.name == left_ident.name;
            }

            false
        }
        _ => false,
    }
}

fn is_mistype_option_fallback(node: &AstNode) -> bool {
    let AstKind::BinaryExpression(binary_expr) = node.kind() else {
        return false;
    };
    if binary_expr.operator == BinaryOperator::BitwiseOR
        && let Expression::Identifier(_) = &binary_expr.left
    {
        return !is_numeric_expr(&binary_expr.right, true);
    }
    false
}

fn is_numeric_expr(expr: &Expression, is_outer_most: bool) -> bool {
    match expr {
        Expression::NumericLiteral(_)
        | Expression::NullLiteral(_)
        // TODO: handle type inference
        | Expression::Identifier(_) => true,
        Expression::UnaryExpression(unary_expr) => {
            if is_outer_most {
                unary_expr.operator != UnaryOperator::Typeof && unary_expr.operator != UnaryOperator::LogicalNot
            } else {
                unary_expr.operator != UnaryOperator::Typeof
            }
        }
        Expression::BinaryExpression(binary_expr) => !is_string_concat(binary_expr),
        Expression::ParenthesizedExpression(paren_expr) => {
            is_numeric_expr(&paren_expr.expression, false)
        }
        _ => {
            if is_outer_most {
                expr.is_undefined()
            } else {
                !expr.is_string_literal()
            }
        }
    }
}

fn is_string_concat(binary_expr: &BinaryExpression) -> bool {
    binary_expr.operator == BinaryOperator::Addition
        && (contains_string_literal(&binary_expr.left)
            || contains_string_literal(&binary_expr.right))
}

fn contains_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral(_) => true,
        Expression::UnaryExpression(unary_expr) => unary_expr.operator == UnaryOperator::Typeof,

        Expression::BinaryExpression(binary_expr) => is_string_concat(binary_expr),
        Expression::ParenthesizedExpression(paren_expr) => {
            contains_string_literal(&paren_expr.expression)
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = obj && obj.a",
        "var a = obj || obj.a",
        "var a = obj1 & obj2.a",
        "var a = b | c",
        "var a = options || {}",
        "var a = options | 1",
        "var a = options | undefined",
        "var a = options | null",
        "var a = options | ~{}",
        "var a = options | (1 + 2 + (3 + !''))",
        "var a = options | (1 + 2 + (3 + !4))",
        "var a = options | (1 + 2 + (3 + +false))",
        "var a = options | (1 + 2 + (3 * '4'))",
        "var a = options | (1 + 2 + (3 * ('4' + 5)))",
        "var a = options | (1 + 2 + (3 + 4))",
        "var a = options | (1 + {})",
        "var a = 1 | {}",
        "var a = 1 | ''",
        "var a = 1 | true",
        "var a = {} | 1",
        "var a = '' | 1",
        "var a = true | 1",
        "var a = b | (1 + 2 + (3 + c))",
        "var a = 1 + '2' - 3",
        "var a = '1' + 2 - 3",
        "input |= 1",
        "input |= undefined",
        "input |= null",
        "input |= (1 + 1)",
        "input |= (1 + true)",
        "input |= (1 + 2 * '3')",
        "input |= (1 + (2 + {}))",
        "input |= ('1' + 2 - 3)",
        "input |= (1 + '2' - 3)",
        "input |= a",
        "input |= ~{}",
        // TODO
        // "var a = 1; input |= a",
        // "var a = 1; var b = a | {}",
    ];

    let fail = vec![
        "var a = obj & obj.a",
        "var a = options | {}",
        "var a = options | !{}",
        "var a = options | typeof {}",
        "var a = options | ''",
        "var a = options | true",
        "var a = options | false",
        "var a = options | (1 + 2 + typeof {})",
        "var a = options | (1 + 2 + (3 + ''))",
        "var a = options | (1 + 2 + (3 + '4'))",
        "input |= ''",
        "input |= (1 + '')",
        "input |= (1 + (3 + '1'))",
        "input |= !{}",
        "input |= typeof {}",
        // TODO
        // "var input; var a = '1'; input |= a",
        // "var a = '1'; var b = a | {}",
    ];

    let fix = vec![
        ("var a = obj & obj.a", "var a = obj && obj.a"),
        ("var a = options | {}", "var a = options || {}"),
        ("var a = options | !{}", "var a = options || !{}"),
        ("var a = options | typeof {}", "var a = options || typeof {}"),
        ("var a = options | ''", "var a = options || ''"),
        ("var a = options | true", "var a = options || true"),
        ("var a = options | false", "var a = options || false"),
        ("var a = options | (1 + 2 + typeof {})", "var a = options || (1 + 2 + typeof {})"),
        ("var a = options | (1 + 2 + (3 + ''))", "var a = options || (1 + 2 + (3 + ''))"),
        ("var a = options | (1 + 2 + (3 + '4'))", "var a = options || (1 + 2 + (3 + '4'))"),
        ("input |= ''", "input ||= ''"),
        ("input |= (1 + '')", "input ||= (1 + '')"),
        ("input |= (1 + (3 + '1'))", "input ||= (1 + (3 + '1'))"),
        ("input |= !{}", "input ||= !{}"),
        ("input |= typeof {}", "input ||= typeof {}"),
        ("var a = obj /* comment */ & obj.a", "var a = obj /* comment */ && obj.a"),
        ("var a = obj &/* comment */ obj.a", "var a = obj &&/* comment */ obj.a"),
        ("var a = obj & /* comment */ obj.a", "var a = obj && /* comment */ obj.a"),
        ("var a = obj/* comment */& obj.a", "var a = obj/* comment */&& obj.a"),
        (
            "var a = obj /* before */ & /* after */ obj.a",
            "var a = obj /* before */ && /* after */ obj.a",
        ),
        ("var a = options /* comment */ | {}", "var a = options /* comment */ || {}"),
        ("var a = options |/* comment */ {}", "var a = options ||/* comment */ {}"),
        ("var a = options | /* comment */ {}", "var a = options || /* comment */ {}"),
        (
            "var a = options /* before */ | /* after */ {}",
            "var a = options /* before */ || /* after */ {}",
        ),
        ("var a = obj\n  & obj.a", "var a = obj\n  && obj.a"),
        ("var a = obj &\n  obj.a", "var a = obj &&\n  obj.a"),
        ("var a = obj /* comment */\n  & obj.a", "var a = obj /* comment */\n  && obj.a"),
        ("var a = options\n  | {}", "var a = options\n  || {}"),
        ("input /* comment */ |= ''", "input /* comment */ ||= ''"),
        ("input |=/* comment */ ''", "input ||=/* comment */ ''"),
        ("input |= /* comment */ ''", "input ||= /* comment */ ''"),
        ("input /* before */ |= /* after */ ''", "input /* before */ ||= /* after */ ''"),
        ("input\n  |= ''", "input\n  ||= ''"),
        ("input |=\n  ''", "input ||=\n  ''"),
        ("input /* comment */\n  |= ''", "input /* comment */\n  ||= ''"),
        (
            "var a = obj /* a */ /* b */ & /* c */ /* d */ obj.a",
            "var a = obj /* a */ /* b */ && /* c */ /* d */ obj.a",
        ),
        ("var a = obj // comment\n  & obj.a", "var a = obj // comment\n  && obj.a"),
        ("var a = options // comment\n  | {}", "var a = options // comment\n  || {}"),
        (
            "var a = obj /* use & for bitwise */ & obj.a",
            "var a = obj /* use & for bitwise */ && obj.a",
        ),
        ("var a = options /* | is bitwise */ | {}", "var a = options /* | is bitwise */ || {}"),
        ("input /* |= assigns */ |= ''", "input /* |= assigns */ ||= ''"),
        (
            "var a = obj & /* use & for bitwise */ obj.a",
            "var a = obj && /* use & for bitwise */ obj.a",
        ),
        ("var a = options | /* | is bitwise */ {}", "var a = options || /* | is bitwise */ {}"),
        ("input |= /* |= assigns */ ''", "input ||= /* |= assigns */ ''"),
    ];

    Tester::new(BadBitwiseOperator::NAME, BadBitwiseOperator::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
