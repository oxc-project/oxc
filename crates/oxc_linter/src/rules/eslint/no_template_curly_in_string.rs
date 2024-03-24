use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-template-curly-in-string): Unexpected template string expression")]
#[diagnostic(
    severity(warning),
    help("Disallow template literal placeholder syntax in regular strings")
)]
struct NoTemplateCurlyInStringDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoTemplateCurlyInString;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow template literal placeholder syntax in regular strings
    ///
    /// ### Why is this bad?
    /// ECMAScript 6 allows programmers to create strings containing variable or expressions using template literals, instead of string concatenation, by writing expressions like ${variable} between two backtick quotes (`). It can be easy to use the wrong quotes when wanting to use template literals, by writing "${variable}", and end up with the literal value "${variable}" instead of a string containing the value of the injected expressions.
    ///
    /// ### Example
    /// ```javascript
    /// /*eslint no-template-curly-in-string: "error"*/
    /// "Hello ${name}!";
    /// 'Hello ${name}!';
    /// "Time: ${12 * 60 * 60 * 1000}";
    /// ```
    NoTemplateCurlyInString,
    style
);

impl Rule for NoTemplateCurlyInString {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::StringLiteral(literal) = node.kind() {
            let text = literal.value.as_str();
            if let Some(start_index) = text.find("${") {
                let mut open_braces_count = 0;
                let mut end_index = None;

                for (i, c) in text[start_index..].char_indices() {
                    let real_index = start_index + i;
                    if c == '{' {
                        open_braces_count += 1;
                    } else if c == '}' && open_braces_count > 0 {
                        open_braces_count -= 1;
                        if open_braces_count == 0 {
                            end_index = Some(real_index);
                            break;
                        }
                    }
                }

                if let Some(end_index) = end_index {
                    let literal_span_start = literal.span.start + 1;
                    let match_start = u32::try_from(start_index)
                        .expect("Conversion from usize to u32 failed for match_start");
                    let match_end = u32::try_from(end_index + 1)
                        .expect("Conversion from usize to u32 failed for match_end");
                    ctx.diagnostic(NoTemplateCurlyInStringDiagnostic(Span::new(
                        literal_span_start + match_start,
                        literal_span_start + match_end,
                    )));
                }
            }
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
        r#""Hello, ${name}""#,
        "'${greeting}, ${name}'",
        "'Hello, ${index + 1}'",
        r#"'Hello, ${name + " foo"}'"#,
        r#"'Hello, ${name || "foo"}'"#,
        r#"'Hello, ${{foo: "bar"}.foo}'"#,
    ];

    Tester::new(NoTemplateCurlyInString::NAME, pass, fail).test_and_snapshot();
}
