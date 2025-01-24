use cow_utils::CowUtils;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_script_url_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `javascript:` url")
        .with_help("Execute the code directly instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoScriptUrl;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow javascript: urls
    ///
    /// ### Why is this bad?
    /// Using `javascript:` URLs is considered by some as a form of `eval`. Code
    /// passed in `javascript:` URLs must be parsed and evaluated by the browser
    /// in the same way that `eval` is processed. This can lead to security and
    /// performance issues.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule
    /// ```javascript
    /// /*eslint no-script-url: "error"*/
    ///
    /// location.href = "javascript:void(0)";
    ///
    /// location.href = `javascript:void(0)`;
    /// ```
    NoScriptUrl,
    eslint,
    style
);

impl Rule for NoScriptUrl {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(literal)
                if literal.value.cow_to_ascii_lowercase().starts_with("javascript:") =>
            {
                emit_diagnostic(ctx, literal.span);
            }
            AstKind::TemplateLiteral(literal)
                if !is_tagged_template_expression(ctx, node, literal.span) =>
            {
                if literal.quasis.len() == 1
                    && literal
                        .quasis
                        .first()
                        .unwrap()
                        .value
                        .raw
                        .cow_to_ascii_lowercase()
                        .starts_with("javascript:")
                {
                    emit_diagnostic(ctx, literal.span);
                }
            }
            _ => {}
        }
    }
}

fn emit_diagnostic(ctx: &LintContext, span: Span) {
    ctx.diagnostic(no_script_url_diagnostic(Span::new(span.start, span.end)));
}

fn is_tagged_template_expression(ctx: &LintContext, node: &AstNode, literal_span: Span) -> bool {
    matches!(
        ctx.nodes().parent_kind(node.id()),
        Some(AstKind::TaggedTemplateExpression(expr)) if expr.quasi.span == literal_span
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 'Hello World!';",
        "var a = 10;",
        "var url = 'xjavascript:'",
        "var url = `xjavascript:`",
        "var url = `${foo}javascript:`",
        "var a = foo`javaScript:`;",
    ];

    let fail = vec![
        "var a = 'javascript:void(0);';",
        "var a = 'javascript:';",
        "var a = `javascript:`;",
        "var a = `JavaScript:`;",
    ];

    Tester::new(NoScriptUrl::NAME, NoScriptUrl::PLUGIN, pass, fail).test_and_snapshot();
}
