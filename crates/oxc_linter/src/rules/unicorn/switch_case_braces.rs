use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::get_preceding_indent_str, context::LintContext, rule::Rule};

fn switch_case_braces_diagnostic_empty_clause(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected braces in empty case clause.")
        .with_help("Remove braces in empty case clause.")
        .with_label(span)
}

fn switch_case_braces_diagnostic_missing_braces(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing braces in case clause.")
        .with_help("Add Braces for case clause.")
        .with_label(span)
}

fn switch_case_braces_diagnostic_unnecessary_braces(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unnecessary braces in case clause.")
        .with_help("Remove Braces for case clause.")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct SwitchCaseBraces {
    always_braces: bool,
}

impl Default for SwitchCaseBraces {
    fn default() -> Self {
        Self { always_braces: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires empty switch cases to omit braces, while non-empty cases must use braces.
    /// This reduces visual clutter for empty cases and enforces proper scoping for non-empty ones.
    ///
    /// ### Why is this bad?
    ///
    /// Using braces unnecessarily for empty cases adds visual noise,
    /// while omitting braces in non-empty cases can lead to scoping issues.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// switch (num) {
    ///   case 1: { }
    ///   case 2:
    ///     console.log('Case 2');
    ///     break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// switch (num) {
    ///   case 1:
    ///   case 2: {
    ///     console.log('Case 2');
    ///     break;
    ///   }
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// `{ type: "always" | "avoid", default: "always" }`
    ///
    /// - `"always"`
    ///   Always report when clause is not a `BlockStatement`.
    ///
    /// - `"avoid"`
    ///   Allows braces only when needed for scoping (e.g., variable or function declarations).
    ///
    /// Example:
    /// ```json
    /// "unicorn/switch-case-braces": ["error", "avoid"]
    /// ```
    SwitchCaseBraces,
    unicorn,
    style,
    fix
);

impl Rule for SwitchCaseBraces {
    fn from_configuration(value: serde_json::Value) -> Self {
        let always_braces = value.get(0).is_none_or(|v| v.as_str() != Some("avoid"));
        Self { always_braces }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else {
            return;
        };

        if switch.cases.is_empty() {
            return;
        }

        for case in &switch.cases {
            if case.consequent.is_empty() {
                continue;
            }
            let missing_braces = match &case.consequent[0] {
                Statement::BlockStatement(block_stmt) if case.consequent.len() == 1 => {
                    if block_stmt.body.is_empty() {
                        ctx.diagnostic_with_fix(
                            switch_case_braces_diagnostic_empty_clause(block_stmt.span),
                            |fixer| fixer.delete_range(block_stmt.span),
                        );
                        continue;
                    }
                    if !self.always_braces
                        && !block_stmt.body.iter().any(|stmt| {
                            matches!(
                                stmt,
                                Statement::VariableDeclaration(_)
                                    | Statement::FunctionDeclaration(_)
                            )
                        })
                    {
                        ctx.diagnostic_with_fix(
                            switch_case_braces_diagnostic_unnecessary_braces(block_stmt.span()),
                            |fixer| {
                                fixer.replace(
                                    block_stmt.span,
                                    fixer.source_range(block_stmt.span.shrink(1)).to_owned(),
                                )
                            },
                        );
                        continue;
                    }
                    false
                }
                _ => true,
            };

            if self.always_braces && missing_braces {
                let colon = u32::try_from(
                    ctx.source_range(Span::new(case.span.start, case.consequent[0].span().start))
                        .rfind(':')
                        .unwrap(),
                )
                .unwrap();
                let span = Span::sized(case.span.start, colon + 1);
                ctx.diagnostic_with_fix(
                    switch_case_braces_diagnostic_missing_braces(span),
                    |fixer| {
                        let fixer = fixer.for_multifix();
                        let mut fix = fixer.new_fix_with_capacity(2);

                        fix.push(fixer.insert_text_after_range(span, " {"));

                        // Indent the closing case bracket, if needed
                        let code = match get_preceding_indent_str(fixer.source_text(), case.span) {
                            Some(indent) => {
                                let mut code = String::with_capacity(2 + indent.len());
                                code.push('\n');
                                code.push_str(indent);
                                code.push('}');
                                code
                            }
                            None => String::from('}'),
                        };

                        fix.push(fixer.insert_text_after_range(case.span, code));
                        fix.with_message("Add Braces for case clause.")
                    },
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "switch(something) { case 1: case 2: {console.log('something'); break;}}",
        "switch(foo){ case 1: { break; } }",
        "switch(foo){ case 1: { ; /* <-- not empty */} }",
        "switch(foo){ case 1: { {} /* <-- not empty */} }",
        "switch(foo){ case 1: { break; } }",
        "switch(foo){ default: { doSomething(); } }",
    ];

    let fail = vec![
        "switch(s){case'':/]/}",
        "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
        "switch(something) { case 1: case 2: console.log('something'); break;}",
        "switch(foo) { case 1: {} case 2: {} default: { doSomething(); } }",
        "switch(foo) { case 1: { /* fallthrough */ } default: {}/* fallthrough */ case 3: { doSomething(); break; } }",
        "switch(foo) { default: doSomething(); }",
        "switch(foo) { case 1: { doSomething(); } break; /* <-- This should be between braces */ }",
        "switch(foo) { default: label: {} }",
        "switch(something) { case 1: case 2: { console.log('something'); break; } case 3: console.log('something else'); }",
        "switch(foo){ case 1: {}; break; }",
    ];

    let fix = vec![
        (
            "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
            "switch(something) { case 1:  case 2: {console.log('something'); break;}}",
            None,
        ),
        (
            "switch(something) { case 1: {} case 2: console.log('something'); break;}",
            "switch(something) { case 1:  case 2: { console.log('something'); break;}}",
            None,
        ),
        (
            "switch(foo) { default: doSomething(); }",
            "switch(foo) { default: { doSomething();} }",
            None,
        ),
        ("switch(s){case'':/]/}", "switch(s){case'': {/]/}}", None),
        (
            "switch(foo) { default: {doSomething();} }",
            "switch(foo) { default: doSomething(); }",
            Some(serde_json::json!(["avoid"])),
        ),
        // Issue: https://github.com/oxc-project/oxc/issues/8491
        (
            "
                const alpha = 7
                let beta = ''
                let gamma = 0

                switch (alpha) {
                    case 1:
                        beta = 'one'
                        gamma = 1
                        break
                }
            ",
            "
                const alpha = 7
                let beta = ''
                let gamma = 0

                switch (alpha) {
                    case 1: {
                        beta = 'one'
                        gamma = 1
                        break
                    }
                }
            ",
            None,
        ),
        ("switch(foo){ case 1: {}; break; }", "switch(foo){ case 1: { {}; break;} }", None),
        (
            "switch(something) { case `scope:type`: doSomething();}",
            "switch(something) { case `scope:type`: { doSomething();}}",
            None,
        ),
        (
            "switch(something) { case \"scope:type\": doSomething();}",
            "switch(something) { case \"scope:type\": { doSomething();}}",
            None,
        ),
        (
            "switch(something) { case 'scope:type': doSomething();}",
            "switch(something) { case 'scope:type': { doSomething();}}",
            None,
        ),
    ];

    Tester::new(SwitchCaseBraces::NAME, SwitchCaseBraces::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
