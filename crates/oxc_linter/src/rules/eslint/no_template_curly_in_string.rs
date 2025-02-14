use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

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
    /// Disallow template literal placeholder syntax in regular strings
    ///
    /// ### Why is this bad?
    ///
    /// ECMAScript 6 allows programmers to create strings containing variable or
    /// expressions using template literals, instead of string concatenation, by
    /// writing expressions like `${variable}` between two backtick quotes. It
    /// can be easy to use the wrong quotes when wanting to use template literals,
    /// by writing `"${variable}"`, and end up with the literal value `"${variable}"`
    /// instead of a string containing the value of the injected expressions.
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
    style,
    conditional_fix
);

impl Rule for NoTemplateCurlyInString {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StringLiteral(literal) = node.kind() else {
            return;
        };

        let text = literal.value.as_str();
        let Some(start) = text.find("${") else { return };

        if text[start + 2..].contains('}') {
            let template = format!("`{text}`");
            let allocator = Allocator::new();
            let ret = Parser::new(&allocator, template.as_str(), *ctx.source_type())
                .with_options(ParseOptions {
                    parse_regular_expression: true,
                    allow_return_outside_function: true,
                    ..ParseOptions::default()
                })
                .parse();

            if !ret.panicked {
                ctx.diagnostic_with_fix(
                    no_template_curly_in_string_diagnostic(literal.span),
                    |fixer| fixer.replace(literal.span, template),
                );
                return;
            }

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

    let fix = vec![
        ("'Hello, ${name}'", "`Hello, ${name}`"),
        (r#""Hello, ${name}""#, r"`Hello, ${name}`"),
        ("'${greeting}, ${name}'", "`${greeting}, ${name}`"),
        ("'Hello, ${index + 1}'", "`Hello, ${index + 1}`"),
        (r#"'Hello, ${name + " foo"}'"#, r#"`Hello, ${name + " foo"}`"#),
        (r#"'Hello, ${name || "foo"}'"#, r#"`Hello, ${name || "foo"}`"#),
        (r#"'Hello, ${{foo: "bar"}.foo}'"#, r#"`Hello, ${{foo: "bar"}.foo}`"#),
    ];

    Tester::new(NoTemplateCurlyInString::NAME, NoTemplateCurlyInString::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
