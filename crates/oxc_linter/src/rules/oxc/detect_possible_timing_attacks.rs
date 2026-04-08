use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_possible_timing_attacks_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Possible timing attack")
        .with_help("Use a constant-time comparison function (e.g., crypto.timingSafeEqual) instead of == or === when comparing security-sensitive values like passwords, tokens, or secrets.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectPossibleTimingAttacks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects direct string comparisons using `==` or `===` with variables whose
    /// names suggest they contain security-sensitive values (passwords, secrets,
    /// tokens, etc.).
    ///
    /// ### Why is this bad?
    ///
    /// Standard string comparison operators like `==` and `===` may short-circuit,
    /// returning early when the first differing character is found. An attacker can
    /// measure the time these comparisons take to infer correct characters one by
    /// one. Use a constant-time comparison function instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (password === userInput) {}
    /// if (token == guess) {}
    /// if (secret === input) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (crypto.timingSafeEqual(Buffer.from(password), Buffer.from(userInput))) {}
    /// ```
    DetectPossibleTimingAttacks,
    oxc,
    suspicious,
    none
);

const SENSITIVE_NAMES: &[&str] = &[
    "password",
    "secret",
    "api_key",
    "apiKey",
    "api_secret",
    "apiSecret",
    "token",
    "auth_token",
    "authToken",
    "access_token",
    "accessToken",
    "refresh_token",
    "refreshToken",
    "hash",
    "signature",
    "csrf",
    "csrfToken",
    "csrf_token",
    "session_id",
    "sessionId",
    "session",
    "privateKey",
    "private_key",
    "secretKey",
    "secret_key",
];

fn is_sensitive_name(name: &str) -> bool {
    let lower = name.cow_to_ascii_lowercase();
    SENSITIVE_NAMES.iter().any(|s| lower.contains(&*s.cow_to_ascii_lowercase()))
}

fn is_sensitive_expression(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(ident) => is_sensitive_name(ident.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            is_sensitive_name(member.property.name.as_str())
        }
        _ => false,
    }
}

fn is_non_literal(expr: &Expression) -> bool {
    !matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
    )
}

impl Rule for DetectPossibleTimingAttacks {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(bin_expr) = node.kind() else {
            return;
        };

        if !matches!(
            bin_expr.operator,
            oxc_syntax::operator::BinaryOperator::Equality
                | oxc_syntax::operator::BinaryOperator::StrictEquality
                | oxc_syntax::operator::BinaryOperator::Inequality
                | oxc_syntax::operator::BinaryOperator::StrictInequality
        ) {
            return;
        }

        let left_sensitive = is_sensitive_expression(&bin_expr.left);
        let right_sensitive = is_sensitive_expression(&bin_expr.right);

        if !left_sensitive && !right_sensitive {
            return;
        }

        // Both sides must be non-literal for a timing attack to be relevant
        if !is_non_literal(&bin_expr.left) || !is_non_literal(&bin_expr.right) {
            return;
        }

        ctx.diagnostic(detect_possible_timing_attacks_diagnostic(bin_expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"if (password === "literal") {}"#,
        "if (x === y) {}",
        "if (count === 5) {}",
        r#"if (name === "admin") {}"#,
        "if (password === null) {}",
    ];

    let fail = vec![
        "if (password === userInput) {}",
        "if (token == guess) {}",
        "if (secret === input) {}",
        "if (user.apiKey === req.body.key) {}",
        "if (hash !== computedHash) {}",
    ];

    Tester::new(DetectPossibleTimingAttacks::NAME, DetectPossibleTimingAttacks::PLUGIN, pass, fail)
        .test_and_snapshot();
}
