use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn consistent_template_literal_escape_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid escape sequence in template literal.")
        .with_help("Use '\\${' to escape '${' in template literals.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentTemplateLiteralEscape;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent style for escaping ${ in template literals.
    ///
    /// ### Why is this bad?
    /// Using `\${` instead of `${` can improve readability and prevent confusion.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    ///const foo = `$\{a}`;
    /// ```
    ///
    /// ```js
    ///const foo = `\$\{a}`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = `\${a}`;
    /// ```
    ConsistentTemplateLiteralEscape,
    unicorn,
    style,
    fix
);

impl Rule for ConsistentTemplateLiteralEscape {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TemplateLiteral(template_literal) = node.kind() else { return };

        if let AstKind::TaggedTemplateExpression(_) =
            ctx.nodes().parent_kind(template_literal.node_id())
        {
            return;
        }

        for quasis in &template_literal.quasis {
            let value = quasis.value.raw.as_ref();
            let chars: Vec<char> = value.chars().collect();
            let len = value.len();
            let mut i = 0;

            while i < len {
                if i + 2 < len && chars[i] == '$' && chars[i + 1] == '\\' && chars[i + 2] == '{' {
                    let mut start_pos = u32::try_from(i).expect("Unable to convert to u32");
                    let end_pos = u32::try_from(i + 3).expect("Unable to convert to u32");
                    let prev_is_backslas = i > 0 && chars[i - 1] == '\\';

                    if prev_is_backslas {
                        start_pos -= 1;
                    }
                    let error_span =
                        Span::new(quasis.span.start + start_pos, quasis.span.start + end_pos);

                    ctx.diagnostic_with_fix(
                        consistent_template_literal_escape_diagnostic(error_span),
                        |fixer| fixer.replace(error_span, "\\${"),
                    );
                }
                i += 1;
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = `\${a}`",
        "const foo = `hello`",
        "const foo = `$`",
        "const foo = `{`",
        "const foo = ``",
        "const foo = `${a}`",
        "const foo = `${a}${b}`",
        r"const foo = String.raw`$\{a}`",
        r"const foo = html`$\{a}`",
        r"const foo = `\\\${a}`",
        r"const foo = '$\{a}'",
    ];

    let fail = vec![
        r"const foo = `$\{a}`",
        r"const foo = `\$\{a}`",
        r"const foo = `$\{a} and $\{b}`",
        r"const foo = `\\$\{a}`",
        r"const foo = `\\\$\{a}`",
        r"const foo = `$\{a}${expr}`",
        r"const foo = `${expr}$\{a}`",
        r"const foo = `$\{a}${expr}$\{b}`",
    ];
    let fix = vec![
        (r"const foo = `$\{a}`", r"const foo = `\${a}`"),
        (r"const foo = `\$\{a}`", r"const foo = `\${a}`"),
        (r"const foo = `$\{a} and $\{b}`", r"const foo = `\${a} and \${b}`"),
        (r"const foo = `\\$\{a}`", r"const foo = `\\${a}`"),
        (r"const foo = `\\\$\{a}`", r"const foo = `\\\${a}`"),
        (r"const foo = `$\{a}${expr}`", r"const foo = `\${a}${expr}`"),
        (r"const foo = `${expr}$\{a}`", r"const foo = `${expr}\${a}`"),
        (r"const foo = `$\{a}${expr}$\{b}`", r"const foo = `\${a}${expr}\${b}`"),
    ];

    Tester::new(
        ConsistentTemplateLiteralEscape::NAME,
        ConsistentTemplateLiteralEscape::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
