use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::DefaultRuleConfig, rule::Rule};

fn no_sequences_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected use of comma operator")
        .with_help("Do not use the comma operator. If you intended to write a sequence, wrap it in parentheses.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoSequences {
    /// If this option is set to `false`, this rule disallows the comma operator
    /// even when the expression sequence is explicitly wrapped in parentheses.
    /// Default is `true`.
    allow_in_parentheses: bool,
}

impl Default for NoSequences {
    fn default() -> Self {
        Self { allow_in_parentheses: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of the comma operator.
    ///
    /// ### Why is this bad?
    ///
    /// The comma operator evaluates each of its operands (from left to right)
    /// and returns the value of the last operand. However, this frequently
    /// obscures side effects, and its use is often an accident.
    ///
    /// ### Options
    ///
    /// - `allowInParentheses` (default: `true`): If set to `false`, disallows
    ///   the comma operator even when wrapped in parentheses.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo = doSomething(), val;
    ///
    /// 0, eval("doSomething();");
    ///
    /// do {} while ((doSomething(), !!test));
    ///
    /// // with allowInParentheses: false
    /// foo = (doSomething(), val);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// foo = (doSomething(), val);
    ///
    /// (0, eval)("doSomething();");
    ///
    /// do {} while (((doSomething(), !!test)));
    ///
    /// for (i = 0, j = 10; i < j; i++, j--) {}
    /// ```
    NoSequences,
    eslint,
    restriction,
    config = NoSequences,
);

impl Rule for NoSequences {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).unwrap_or_default().into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SequenceExpression(seq_expr) = node.kind() else {
            return;
        };

        // Always allow in for loop init and update
        if Self::is_in_for_loop_init_or_update(node, ctx) {
            return;
        }

        // Check for parentheses if allowed
        if self.allow_in_parentheses && Self::is_allowed_by_parentheses(node, ctx) {
            return;
        }

        // Report at the span of the first comma (between first and second expression)
        let span = if seq_expr.expressions.len() >= 2 {
            // Get span between first and second expression (where the comma is)
            let first_end = seq_expr.expressions[0].span().end;
            let second_start = seq_expr.expressions[1].span().start;
            Span::new(first_end, second_start)
        } else {
            seq_expr.span
        };

        ctx.diagnostic(no_sequences_diagnostic(span));
    }
}

impl NoSequences {
    /// Check if the sequence expression is in a for loop's init or update position.
    /// This walks up the parent chain, skipping ParenthesizedExpression nodes,
    /// to handle cases like `for ((a, b);;)`.
    fn is_in_for_loop_init_or_update(node: &AstNode, ctx: &LintContext) -> bool {
        let nodes = ctx.nodes();
        let mut cur = nodes.parent_node(node.id());

        // Skip through ParenthesizedExpression nodes
        while matches!(cur.kind(), AstKind::ParenthesizedExpression(_)) {
            cur = nodes.parent_node(cur.id());
        }

        // Check if we've reached a ForStatement
        if let AstKind::ForStatement(for_stmt) = cur.kind() {
            let node_span = node.span();

            // Check if in init position (compare with outermost span)
            if let Some(init) = &for_stmt.init
                && init.span().contains_inclusive(node_span)
            {
                return true;
            }

            // Check if in update position
            if let Some(update) = &for_stmt.update
                && update.span().contains_inclusive(node_span)
            {
                return true;
            }
        }

        false
    }

    /// Check if the sequence expression is allowed due to parentheses.
    /// Counts parentheses depth and checks if enough parentheses exist
    /// for positions that grammatically require them.
    fn is_allowed_by_parentheses(node: &AstNode, ctx: &LintContext) -> bool {
        let nodes = ctx.nodes();
        let mut cur = nodes.parent_node(node.id());
        let mut paren_depth = 0;

        // Count consecutive ParenthesizedExpression depth
        while matches!(cur.kind(), AstKind::ParenthesizedExpression(_)) {
            paren_depth += 1;
            cur = nodes.parent_node(cur.id());
        }

        // If not wrapped in any parentheses, not allowed
        if paren_depth == 0 {
            return false;
        }

        // Check for ArrowFunctionExpression body
        // In oxc's AST: SequenceExpr -> ParenthesizedExpr -> ExpressionStatement -> FunctionBody -> ArrowFunctionExpr
        // Arrow body needs double parentheses because the first layer is syntactically required
        // (otherwise `() => a, b` would be parsed as `(() => a), b`)
        let is_arrow_body = matches!(cur.kind(), AstKind::ExpressionStatement(_))
            && matches!(
                nodes.parent_node(nodes.parent_node(cur.id()).id()).kind(),
                AstKind::ArrowFunctionExpression(arrow) if arrow.expression
            );

        if is_arrow_body {
            // Arrow body needs at least 2 levels of parentheses
            // () => (a, b) has paren_depth=1, needs () => ((a, b)) with paren_depth=2
            paren_depth >= 2
        } else {
            // For if/while/do/switch/with and other positions:
            // The grammar-required parentheses (e.g., `if (...)`) don't appear in the AST
            // as ParenthesizedExpression, so paren_depth >= 1 means one extra layer of parens
            // which indicates intentional use of the comma operator.
            // E.g., `if ((a, b))` has paren_depth=1 and should be allowed.
            true
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // For loop init and update are always allowed (even with allowInParentheses: false)
        ("for (i = 0, j = 10; i < j; i++, j--) {}", None),
        ("for (a = 0, b = 10; a < b; a++, b--) foo();", None),
        // For loop with parentheses around init/update - still allowed
        ("for ((a, b);;);", None),
        ("for (;; (a, b));", None),
        ("for ((a, b);;);", Some(serde_json::json!([{ "allowInParentheses": false }]))),
        // Wrapped in parentheses (default allowInParentheses: true)
        ("foo = (doSomething(), val);", None),
        ("(0, eval)(\"doSomething();\");", None),
        ("a = ((b, c), d);", None),
        // Single extra parentheses in conditions is enough (ESLint docs examples)
        // Grammar-required parens (e.g., `if (...)`) don't appear in AST
        ("if ((doSomething(), !!test));", None),
        ("do {} while ((doSomething(), !!test));", None),
        ("while ((a, b)) {}", None),
        ("switch ((val = foo(), val)) {}", None),
        ("with ((doSomething(), val)) {}", None),
        // Arrow function body requires double parentheses
        ("const fn = (x) => ((log(), x));", None),
        // With allowInParentheses: true (explicit)
        ("foo = (doSomething(), val);", Some(serde_json::json!([{ "allowInParentheses": true }]))),
    ];

    let fail = vec![
        // Basic sequence without parentheses
        ("foo = doSomething(), val;", None),
        ("0, eval(\"doSomething();\");", None),
        ("a = b, c;", None),
        // Arrow function body with single parentheses (needs double per ESLint)
        ("const fn = (x) => (log(), x);", None),
        // With allowInParentheses: false, even parenthesized sequences are errors
        ("foo = (doSomething(), val);", Some(serde_json::json!([{ "allowInParentheses": false }]))),
        (
            "(0, eval)(\"doSomething();\");",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
    ];

    Tester::new(NoSequences::NAME, NoSequences::PLUGIN, pass, fail).test_and_snapshot();
}
