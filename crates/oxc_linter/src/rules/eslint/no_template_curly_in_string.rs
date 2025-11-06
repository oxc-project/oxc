use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_template_curly_in_string_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Template placeholders will not interpolate in regular strings")
        .with_help("Did you mean to use a template string literal?")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoTemplateCurlyInString;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow template literal placeholder syntax in regular strings. This rule ensures that
    /// expressions like `${variable}` are only used within template literals, avoiding incorrect
    /// usage in regular strings.
    ///
    /// ### Why is this bad?
    ///
    /// ECMAScript 6 allows programmers to create strings containing variables or expressions using
    /// template literals. This is done by embedding expressions like `${variable}` between backticks.
    /// If regular quotes (`'` or `"`) are used with template literal syntax, it results in the literal
    /// string `"${variable}"` instead of evaluating the expression. This rule helps to avoid this mistake,
    /// ensuring that expressions are correctly evaluated inside template literals.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// "Hello ${name}!";
    /// 'Hello ${name}!';
    /// "Time: ${12 * 60 * 60 * 1000}";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// `Hello ${name}!`;
    /// `Time: ${12 * 60 * 60 * 1000}`;
    /// templateFunction`Hello ${name}`;
    /// ```
    NoTemplateCurlyInString,
    eslint,
    style
);

impl Rule for NoTemplateCurlyInString {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StringLiteral(literal) = node.kind() else {
            return;
        };

        let text = literal.value.as_str();
        let Some(start) = text.find("${") else { return };

        if text[start + 2..].contains('}') {
            ctx.diagnostic(no_template_curly_in_string_diagnostic(literal.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "`Hello, ${name}`;",
        "templateFunction`Hello, ${name}`;",
        "`Hello, name`;",
        "'Hello, name';",
        "'Hello, ' + name;",
        "`Hello, ${index + 1}`",
        r#"`Hello, ${name + " foo"}`"#,
        r#"`Hello, ${name || "foo"}`"#,
        r#"`Hello, ${{foo: "bar"}.foo}`"#,
        "'$2'",
        "'${'",
        "'$}'",
        "'{foo}'",
        r#"'{foo: "bar"}'"#,
        "const number = 3",
    ];

    let fail = vec![
        "'Hello, ${name}'",
        "'Hello, ${{name}'",
        r#""Hello, ${name}""#,
        "'${greeting}, ${name}'",
        "'Hello, ${index + 1}'",
        r#"'Hello, ${name + " foo"}'"#,
        r#"'Hello, ${name || "foo"}'"#,
        r#"'Hello, ${{foo: "bar"}.foo}'"#,
    ];

    Tester::new(NoTemplateCurlyInString::NAME, NoTemplateCurlyInString::PLUGIN, pass, fail)
        .test_and_snapshot();
}
