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
    /// // Arrow function body needs double parentheses
    /// const fn = () => (doSomething(), val);
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
    /// // Single extra parentheses is enough for conditions
    /// do {} while ((doSomething(), !!test));
    ///
    /// for (i = 0, j = 10; i < j; i++, j--) {}
    ///
    /// // Arrow function body needs double parentheses
    /// const fn = () => ((doSomething(), val));
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
        ("var arr = [1, 2];", None),
        ("var obj = {a: 1, b: 2};", None),
        ("var a = 1, b = 2;", None),
        ("var foo = (1, 2);", None),
        (r#"(0,eval)("foo()");"#, None),
        ("for (i = 1, j = 2;; i++, j++);", None),
        ("foo(a, (b, c), d);", None),
        ("do {} while ((doSomething(), !!test));", None),
        ("for ((doSomething(), somethingElse()); (doSomething(), !!test); );", None),
        ("if ((doSomething(), !!test));", None),
        ("switch ((doSomething(), val)) {}", None),
        ("while ((doSomething(), !!test));", None),
        ("with ((doSomething(), val)) {}", None),
        ("a => ((doSomething(), a))", None),
        ("var foo = (1, 2);", Some(serde_json::json!([{}]))),
        ("var foo = (1, 2);", Some(serde_json::json!([{ "allowInParentheses": true }]))),
        (
            "for ((i = 0, j = 0); test; );",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        ("for (; test; (i++, j++));", Some(serde_json::json!([{ "allowInParentheses": false }]))),
        ("const foo = () => { return ((bar = 123), 10) }", None),
        ("const foo = () => (((bar = 123), 10));", None),
    ];

    let fail = vec![
        ("1, 2;", None),
        ("a = 1, 2", None),
        ("do {} while (doSomething(), !!test);", None),
        ("for (; doSomething(), !!test; );", None),
        ("if (doSomething(), !!test);", None),
        ("switch (doSomething(), val) {}", None),
        ("while (doSomething(), !!test);", None),
        ("with (doSomething(), val) {}", None),
        ("a => (doSomething(), a)", None),
        ("(1), 2", None),
        ("((1)) , (2)", None),
        ("while((1) , 2);", None),
        ("var foo = (1, 2);", Some(serde_json::json!([{ "allowInParentheses": false }]))),
        (r#"(0,eval)("foo()");"#, Some(serde_json::json!([{ "allowInParentheses": false }]))),
        ("foo(a, (b, c), d);", Some(serde_json::json!([{ "allowInParentheses": false }]))),
        (
            "do {} while ((doSomething(), !!test));",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        (
            "for (; (doSomething(), !!test); );",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        (
            "if ((doSomething(), !!test));",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        (
            "switch ((doSomething(), val)) {}",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        (
            "while ((doSomething(), !!test));",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        (
            "with ((doSomething(), val)) {}",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
        ("a => ((doSomething(), a))", Some(serde_json::json!([{ "allowInParentheses": false }]))), // { "ecmaVersion": 6 }
    ];

    Tester::new(NoSequences::NAME, NoSequences::PLUGIN, pass, fail).test_and_snapshot();
}
