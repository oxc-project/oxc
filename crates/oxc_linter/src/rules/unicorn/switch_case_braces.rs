use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::get_preceding_indent_str, context::LintContext, rule::Rule, AstNode};

#[derive(Clone, Copy)]
enum Diagnostic {
    EmptyClause,
    MissingBraces,
    UnnecessaryBraces,
}

fn switch_case_braces_diagnostic(span: Span, diagnostic_type: Diagnostic) -> OxcDiagnostic {
    (match diagnostic_type {
        Diagnostic::EmptyClause => OxcDiagnostic::warn("Unexpected braces in empty case clause.")
            .with_help("Remove braces in empty case clause."),
        Diagnostic::MissingBraces => OxcDiagnostic::warn("Missing braces in case clause.")
            .with_help("Add Braces for case clause."),
        Diagnostic::UnnecessaryBraces => OxcDiagnostic::warn("Unnecessary braces in case clause.")
            .with_help("Remove Braces for case clause."),
    })
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SwitchCaseBraces {
    // true - "always" (default)
    //   - Always report when clause is not a BlockStatement
    // false - "avoid"
    //   - Only allow braces when there are variable declaration or function declaration which requires a scope.
    always_braces: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Require empty switch cases to not have braces. Non-empty braces are required to have braces around them.
    ///
    /// ### Why is this bad?
    /// There is less visual clutter for empty cases and proper scope for non-empty cases.
    ///
    /// ### Example
    /// ```javascript
    /// switch (num) {
    ///     case 1: {
    ///
    ///     }
    ///     case 2:
    ///         console.log('Case 2');
    ///         break;
    /// }
    /// ```
    SwitchCaseBraces,
    unicorn,
    style,
    fix
);

impl Rule for SwitchCaseBraces {
    fn from_configuration(value: serde_json::Value) -> Self {
        let always = value.get(0).map_or(true, |v| v.as_str() != Some("avoid"));

        Self { always_braces: always }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else {
            return;
        };

        if switch.cases.is_empty() {
            return;
        }

        for case in &switch.cases {
            for case_consequent in &case.consequent {
                match case_consequent {
                    Statement::BlockStatement(case_block) => {
                        if case_block.body.is_empty() {
                            ctx.diagnostic_with_fix(
                                switch_case_braces_diagnostic(
                                    case_block.span,
                                    Diagnostic::EmptyClause,
                                ),
                                |fixer| fixer.delete_range(case_block.span),
                            );
                        }

                        if !self.always_braces
                            && !case_block.body.iter().any(|stmt| {
                                matches!(
                                    stmt,
                                    Statement::VariableDeclaration(_)
                                        | Statement::FunctionDeclaration(_)
                                )
                            })
                        {
                            ctx.diagnostic_with_fix(
                                switch_case_braces_diagnostic(
                                    case_block.span(),
                                    Diagnostic::UnnecessaryBraces,
                                ),
                                |fixer| {
                                    fixer.replace(
                                        case_block.span,
                                        fixer.source_range(Span::new(
                                            case_block.span.start + 1,
                                            case_block.span.end - 1,
                                        )),
                                    )
                                },
                            );
                        }
                    }
                    Statement::EmptyStatement(_) => {}
                    _ => {
                        if !self.always_braces
                            && !&case.consequent.iter().any(|stmt| {
                                matches!(
                                    stmt,
                                    Statement::VariableDeclaration(_)
                                        | Statement::FunctionDeclaration(_)
                                )
                            })
                        {
                            return;
                        }

                        let Some(first_statement) = &case.consequent.first() else {
                            return;
                        };
                        let Some(last_statement) = &case.consequent.last() else {
                            return;
                        };

                        let case_body_span =
                            Span::new(first_statement.span().start, last_statement.span().end);

                        ctx.diagnostic_with_fix(
                            switch_case_braces_diagnostic(
                                case_body_span,
                                Diagnostic::MissingBraces,
                            ),
                            |fixer| {
                                let modified_code = {
                                    let mut formatter = fixer.codegen();

                                    if let Some(case_test) = &case.test {
                                        formatter.print_str("case ");
                                        formatter.print_expression(case_test);
                                    } else {
                                        formatter.print_str("default");
                                    }

                                    formatter.print_ascii_byte(b':');
                                    formatter.print_ascii_byte(b' ');
                                    formatter.print_ascii_byte(b'{');

                                    let source_text = ctx.source_text();

                                    for x in &case.consequent {
                                        if matches!(
                                            x,
                                            Statement::ExpressionStatement(_)
                                                | Statement::BreakStatement(_)
                                        ) {
                                            // indent the statement in the case consequent, if needed
                                            if let Some(indent_str) =
                                                get_preceding_indent_str(source_text, x.span())
                                            {
                                                formatter.print_ascii_byte(b'\n');
                                                formatter.print_str(indent_str);
                                            }
                                        }

                                        formatter.print_str(x.span().source_text(source_text));
                                    }

                                    // indent the closing case bracket, if needed
                                    if let Some(case_indent_str) =
                                        get_preceding_indent_str(source_text, case.span())
                                    {
                                        formatter.print_ascii_byte(b'\n');
                                        formatter.print_str(case_indent_str);
                                    }

                                    formatter.print_ascii_byte(b'}');

                                    formatter.into_source_text()
                                };

                                fixer.replace(case.span, modified_code)
                            },
                        );

                        // After first incorrect consequent we have to break to not repeat the work
                        break;
                    }
                }
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
    ];

    let fix = vec![
        (
            "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
            "switch(something) { case 1:  case 2: {console.log('something'); break;}}",
            None,
        ),
        (
            "switch(something) { case 1: {} case 2: console.log('something'); break;}",
            "switch(something) { case 1:  case 2: {console.log('something');break;}}",
            None,
        ),
        (
            "switch(foo) { default: doSomething(); }",
            "switch(foo) { default: {doSomething();} }",
            None,
        ),
        ("switch(s){case'':/]/}", "switch(s){case '': {/]/}}", None),
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
    ];

    Tester::new(SwitchCaseBraces::NAME, SwitchCaseBraces::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
