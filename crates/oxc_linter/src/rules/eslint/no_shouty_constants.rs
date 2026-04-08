use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_shouty_constants_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "SCREAMING_SNAKE_CASE variable `{name}` is assigned a simple string literal."
    ))
    .with_help("UPPER_CASE names are conventionally reserved for true constants. Use camelCase or rename if this value is not a meaningful constant.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoShoutyConstants;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Discourages declaring SCREAMING_SNAKE_CASE variables that are assigned
    /// a simple string or number literal, suggesting they may not truly be constants.
    ///
    /// ### Why is this bad?
    ///
    /// UPPER_CASE naming is conventionally reserved for meaningful constants.
    /// Using it for trivial string assignments (like `const FOO = "foo"`) adds
    /// noise without value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const FOO = "foo";
    /// const BAR_BAZ = "bar_baz";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const MAX_RETRIES = 3;
    /// const API_URL = process.env.API_URL;
    /// const foo = "foo";
    /// ```
    NoShoutyConstants,
    eslint,
    style,
    pending
);

impl Rule for NoShoutyConstants {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };

        // Only check const declarations
        if declarator.kind != oxc_ast::ast::VariableDeclarationKind::Const {
            return;
        }

        let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &declarator.id else {
            return;
        };

        let name = ident.name.as_str();

        // Check if name is SCREAMING_SNAKE_CASE (all uppercase + underscores, at least 2 chars)
        if name.len() < 2 || !is_screaming_snake_case(name) {
            return;
        }

        // Check if the initializer is a simple string literal whose value matches the name
        let Some(init) = &declarator.init else {
            return;
        };

        match init {
            Expression::StringLiteral(lit) => {
                // Flag if the string value is basically the same as the name (case-insensitive)
                let val = lit.value.as_str();
                if val.eq_ignore_ascii_case(name)
                    || val.eq_ignore_ascii_case(&name.to_lowercase())
                    || val.eq_ignore_ascii_case(&name.replace('_', "-"))
                    || val.eq_ignore_ascii_case(&name.replace('_', ""))
                {
                    ctx.diagnostic(no_shouty_constants_diagnostic(ident.span, name));
                }
            }
            _ => {}
        }
    }
}

fn is_screaming_snake_case(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && s.contains('_')
        && s.chars().any(|c| c.is_ascii_uppercase())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const MAX_RETRIES = 3;",
        "const API_URL = process.env.API_URL;",
        "const foo = 'foo';",
        "const FOO = getSomething();",
        "const FOO = 'completely_different';",
    ];

    let fail = vec![
        r#"const FOO_BAR = "foo_bar";"#,
        r#"const BAR_BAZ = "bar-baz";"#,
        r#"const HELLO_WORLD = "helloworld";"#,
    ];

    Tester::new(NoShoutyConstants::NAME, NoShoutyConstants::PLUGIN, pass, fail).test_and_snapshot();
}
