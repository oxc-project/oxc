use oxc_ast::{ast::UpdateOperator, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_plusplus_diagnostic(span: Span, operator: UpdateOperator) -> OxcDiagnostic {
    let diagnostic = OxcDiagnostic::warn(format!(
        "Unary operator '{operator}' used.",
        operator = operator.as_str()
    ))
    .with_label(span);

    match operator {
        UpdateOperator::Increment => {
            diagnostic.with_help("Use the assignment operator `+=` instead.")
        }
        UpdateOperator::Decrement => {
            diagnostic.with_help("Use the assignment operator `-=` instead.")
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoPlusplus {
    /// Whether to allow `++` and `--` in for loop afterthoughts.
    allow_for_loop_afterthoughts: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the unary operators `++`` and `--`.
    ///
    /// ### Why is this bad?
    ///
    /// Because the unary `++` and `--` operators are subject to automatic semicolon insertion, differences in whitespace
    /// can change the semantics of source code. For example, these two code blocks are not equivalent:
    ///
    /// ```js
    /// var i = 10;
    /// var j = 20;
    ///
    /// i ++
    /// j
    /// // => i = 11, j = 20
    /// ```
    ///
    /// ```js
    /// var i = 10;
    /// var j = 20;
    ///
    /// i
    /// ++
    /// j
    /// // => i = 10, j = 21
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var x = 0; x++;
    /// var y = 0; y--;
    /// for (let i = 0; i < l; i++) {
    ///     doSomething(i);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var x = 0; x += 1;
    /// var y = 0; y -= 1;
    /// for (let i = 0; i < l; i += 1) {
    ///    doSomething(i);
    /// }
    /// ```
    NoPlusplus,
    restriction,
    // This is not guaranteed to rewrite the code in a way that is equivalent.
    // For example, `++i` and `i++` will be rewritten as `i += 1` even though they are not the same.
    // If the code depends on the order of evaluation, then this might break it.
    conditional_suggestion,
);

impl Rule for NoPlusplus {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self {
            allow_for_loop_afterthoughts: obj
                .and_then(|v| v.get("allowForLoopAfterthoughts"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UpdateExpression(expr) = node.kind() else {
            return;
        };

        if self.allow_for_loop_afterthoughts && is_for_loop_afterthought(node, ctx).unwrap_or(false)
        {
            return;
        }

        let ident = expr.argument.get_identifier();

        if let Some(ident) = ident {
            let operator = match expr.operator {
                UpdateOperator::Increment => "+=",
                UpdateOperator::Decrement => "-=",
            };
            ctx.diagnostic_with_suggestion(
                no_plusplus_diagnostic(expr.span, expr.operator),
                |fixer| fixer.replace(expr.span, format!("{ident} {operator} 1")),
            );
        } else {
            ctx.diagnostic(no_plusplus_diagnostic(expr.span, expr.operator));
        }
    }
}

/// Determines whether the given node is considered to be a for loop "afterthought" by the logic of this rule.
/// In particular, it returns `true` if the given node is either:
///   - The update node of a `ForStatement`: for (;; i++) {}
///   - An operand of a sequence expression that is the update node: for (;; foo(), i++) {}
///   - An operand of a sequence expression that is child of another sequence expression, etc.,
///     up to the sequence expression that is the update node: for (;; foo(), (bar(), (baz(), i++))) {}
fn is_for_loop_afterthought(node: &AstNode, ctx: &LintContext) -> Option<bool> {
    let mut cur = ctx.nodes().parent_node(node.id())?;

    while let AstKind::SequenceExpression(_) | AstKind::ParenthesizedExpression(_) = cur.kind() {
        cur = ctx.nodes().parent_node(cur.id())?;
    }

    Some(matches!(cur.kind(), AstKind::ForStatement(stmt) if stmt.update.is_some()))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = 0; foo=+1;", None),
        ("var foo = 0; foo+=1;", None),
        ("var foo = 0; foo-=1;", None),
        ("var foo = 0; foo=+1;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (i = 0; i < l; i++) { console.log(i); }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (var i = 0, j = i + 1; j < example.length; i++, j++) {}",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        ("for (;; i--, foo());", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; foo(), --i);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (;; foo(), ++i, bar);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; i++, (++j, k--));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; foo(), (bar(), i++), baz());",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; (--i, j += 2), bar = j + 1);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; a, (i--, (b, ++j, c)), d);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
    ];

    let fail = vec![
        ("var foo = 0; foo++;", None),
        ("var foo = 0; foo--;", None),
        ("var foo = 0; --foo;", None),
        ("var foo = 0; ++foo;", None),
        ("for (i = 0; i < l; i++) { console.log(i); }", None),
        ("for (i = 0; i < l; foo, i++) { console.log(i); }", None),
        ("var foo = 0; foo++;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (i = 0; i < l; i++) { v++; }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        ("for (i++;;);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;--i;);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;;) ++i;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; i = j++);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; i++, f(--j));", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (;; foo + (i++, bar));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
    ];

    let fix = vec![
        ("var foo = 0; foo++;", "var foo = 0; foo += 1;", None),
        ("var foo = 0; foo--;", "var foo = 0; foo -= 1;", None),
        ("var foo = 0; --foo;", "var foo = 0; foo -= 1;", None),
        ("var foo = 0; ++foo;", "var foo = 0; foo += 1;", None),
        (
            "for (i = 0; i < l; i++) { console.log(i); }",
            "for (i = 0; i < l; i += 1) { console.log(i); }",
            None,
        ),
        (
            "for (i = 0; i < l; foo, i++) { console.log(i); }",
            "for (i = 0; i < l; foo, i += 1) { console.log(i); }",
            None,
        ),
        (
            "var foo = 0; foo++;",
            "var foo = 0; foo += 1;",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (i = 0; i < l; i++) { v++; }",
            "for (i = 0; i < l; i++) { v += 1; }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (i++;;);",
            "for (i += 1;;);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;--i;);",
            "for (;i -= 1;);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;;) ++i;",
            "for (;;) i += 1;",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; i = j++);",
            "for (;; i = j += 1);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            // Do not fix if part of a function call like f(--j)
            "for (;; i++, f(--j));",
            "for (;; i++, f(j -= 1));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; foo + (i++, bar));",
            "for (;; foo + (i += 1, bar));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            // Do not fix if part of property definition
            "let x = 0; let y = { foo: x++ };",
            "let x = 0; let y = { foo: x += 1 };",
            None,
        ),
    ];

    Tester::new(NoPlusplus::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
