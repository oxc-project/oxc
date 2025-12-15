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
    /// Check if the sequence expression is in a for loop's init or update position
    fn is_in_for_loop_init_or_update(node: &AstNode, ctx: &LintContext) -> bool {
        let nodes = ctx.nodes();
        let parent = nodes.parent_node(node.id());

        // Check if the parent is a ForStatement and this is in init or update
        if let AstKind::ForStatement(for_stmt) = parent.kind() {
            let node_span = node.span();

            // Check if in init position
            if let Some(init) = &for_stmt.init
                && init.span() == node_span
            {
                return true;
            }

            // Check if in update position
            if let Some(update) = &for_stmt.update
                && update.span() == node_span
            {
                return true;
            }
        }

        false
    }

    /// Check if the sequence expression is allowed due to parentheses
    fn is_allowed_by_parentheses(node: &AstNode, ctx: &LintContext) -> bool {
        let nodes = ctx.nodes();
        let parent = nodes.parent_node(node.id());

        // Check if wrapped in parentheses
        let is_wrapped_once = matches!(parent.kind(), AstKind::ParenthesizedExpression(_));

        if !is_wrapped_once {
            return false;
        }

        // For most cases, single parentheses are enough
        // But for grammar positions that require parentheses (if, while, do-while, switch, with),
        // we need double parentheses
        let grandparent = nodes.parent_node(parent.id());

        let requires_extra_parens = matches!(
            grandparent.kind(),
            AstKind::IfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::SwitchStatement(_)
                | AstKind::WithStatement(_)
        );

        if !requires_extra_parens {
            return true;
        }

        // Need double parentheses for these cases
        // Check if grandparent is also a ParenthesizedExpression
        matches!(grandparent.kind(), AstKind::ParenthesizedExpression(_))
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // For loop init and update are allowed
        ("for (i = 0, j = 10; i < j; i++, j--) {}", None),
        ("for (a = 0, b = 10; a < b; a++, b--) foo();", None),
        // Wrapped in parentheses (default allowInParentheses: true)
        ("foo = (doSomething(), val);", None),
        ("(0, eval)(\"doSomething();\");", None),
        ("a = ((b, c), d);", None),
        // Double parentheses for conditions
        ("do {} while (((doSomething(), !!test)));", None),
        ("while (((a, b))) {}", None),
        ("if (((a, b))) {}", None),
        ("switch (((a, b))) {}", None),
        // Arrow function with parentheses (single is enough since body doesn't require parens)
        ("const fn = (x) => (log(), x);", None),
        // With allowInParentheses: true (explicit)
        ("foo = (doSomething(), val);", Some(serde_json::json!([{ "allowInParentheses": true }]))),
    ];

    let fail = vec![
        // Basic sequence without parentheses
        ("foo = doSomething(), val;", None),
        ("0, eval(\"doSomething();\");", None),
        ("a = b, c;", None),
        // Single parentheses in condition (needs double)
        ("do {} while ((doSomething(), !!test));", None),
        ("while ((a, b)) {}", None),
        ("if ((a, b)) {}", None),
        ("switch ((a, b)) {}", None),
        // With allowInParentheses: false, even parenthesized sequences are errors
        ("foo = (doSomething(), val);", Some(serde_json::json!([{ "allowInParentheses": false }]))),
        (
            "(0, eval)(\"doSomething();\");",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
        ),
    ];

    Tester::new(NoSequences::NAME, NoSequences::PLUGIN, pass, fail).test_and_snapshot();
}
