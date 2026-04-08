use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn detect_new_buffer_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("new Buffer() is deprecated and unsafe")
        .with_help("Use Buffer.alloc(), Buffer.allocUnsafe(), or Buffer.from() instead of the deprecated Buffer constructor.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DetectNewBuffer;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects use of the `new Buffer()` constructor.
    ///
    /// ### Why is this bad?
    ///
    /// The `Buffer` constructor is deprecated due to security and usability issues.
    /// When called with a number, `new Buffer(n)` returns uninitialized memory that
    /// may contain sensitive data. Use `Buffer.alloc()`, `Buffer.allocUnsafe()`, or
    /// `Buffer.from()` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new Buffer(16);
    /// new Buffer("string");
    /// new Buffer([1, 2, 3]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Buffer.alloc(16);
    /// Buffer.from("string");
    /// Buffer.from([1, 2, 3]);
    /// ```
    DetectNewBuffer,
    oxc,
    suspicious,
    none
);

impl Rule for DetectNewBuffer {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        let Expression::Identifier(callee) = &new_expr.callee else {
            return;
        };

        if callee.name == "Buffer" {
            ctx.diagnostic(detect_new_buffer_diagnostic(new_expr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Buffer.alloc(16)",
        "Buffer.from('string')",
        "Buffer.from([1, 2, 3])",
        "new Map()",
        "Buffer.allocUnsafe(16)",
    ];

    let fail = vec![
        "new Buffer(16)",
        "new Buffer('string')",
        "new Buffer([1, 2, 3])",
        "new Buffer(variable)",
    ];

    Tester::new(DetectNewBuffer::NAME, DetectNewBuffer::PLUGIN, pass, fail).test_and_snapshot();
}
