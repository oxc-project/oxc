use oxc_ast::{AstKind, ast::TSEnumMemberName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::RuleFixer,
    rule::Rule,
};

fn prefer_enum_initializers_diagnostic(member_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The value of the member {member_name:?} should be explicitly defined."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferEnumInitializers;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require each enum member value to be explicitly initialized.
    ///
    /// ### Why is this bad?
    ///
    /// In projects where the value of `enum` members are important, allowing implicit values for enums can cause bugs if enums are modified over time.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// // wrong, the value of `Close` is not constant
    /// enum Status {
    ///  Open = 1,
    ///  Close,
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// enum Status {
    ///  Open = 1,
    ///  Close = 2,
    /// }
    /// ```
    PreferEnumInitializers,
    typescript,
    pedantic,
    suggestion
);

impl Rule for PreferEnumInitializers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumBody(enum_body) = node.kind() else {
            return;
        };

        for (index, member) in enum_body.members.iter().enumerate() {
            if member.initializer.is_none()
                && let TSEnumMemberName::Identifier(i) = &member.id
            {
                let member_name = i.name.as_str();
                let name_span = i.span;
                let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
                ctx.diagnostic_with_suggestions(
                    prefer_enum_initializers_diagnostic(member_name, member.span),
                    [
                        fixer
                            .replace(name_span, format!("{member_name} = {index}"))
                            .with_message(format!("Initialize to `{index}` (the enum index).")),
                        fixer
                            .replace(name_span, format!("{member_name} = {}", index + 1))
                            .with_message(format!(
                                "Initialize to `{}` (the enum index + 1).",
                                index + 1
                            )),
                        fixer
                            .replace(name_span, format!("{member_name} = '{member_name}'"))
                            .with_message(format!(
                                "Initialize to `'{member_name}'` (the enum member name)."
                            )),
                    ],
                );
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};

    let pass = vec![
        "
            enum Direction {}
                ",
        "
            enum Direction {
              Up = 1,
            }
                ",
        "
            enum Direction {
              Up = 1,
              Down = 2,
            }
                ",
        "
            enum Direction {
              Up = 'Up',
              Down = 'Down',
            }
                ",
    ];

    let fail = vec![
        "
            enum Direction {
              Up,
            }
                  ",
        "
            enum Direction {
              Up,
              Down,
            }
                  ",
        "
            enum Direction {
              Up = 'Up',
              Down,
            }
                  ",
        "
            enum Direction {
              Up,
              Down = 'Down',
            }
                  ",
    ];

    // Each test case provides 3 suggestions: index, index+1, and member name as string
    // When multiple members are uninitialized, all fixes for the same suggestion type are applied
    let fix: Vec<ExpectFixTestCase> = vec![
        (
            "enum Direction { Up, }",
            (
                "enum Direction { Up = 0, }",
                "enum Direction { Up = 1, }",
                "enum Direction { Up = 'Up', }",
            ),
        )
            .into(),
        (
            "enum Direction { Up, Down, }",
            (
                "enum Direction { Up = 0, Down = 1, }",
                "enum Direction { Up = 1, Down = 2, }",
                "enum Direction { Up = 'Up', Down = 'Down', }",
            ),
        )
            .into(),
        (
            "enum Direction { Up = 'Up', Down, }",
            (
                "enum Direction { Up = 'Up', Down = 1, }",
                "enum Direction { Up = 'Up', Down = 2, }",
                "enum Direction { Up = 'Up', Down = 'Down', }",
            ),
        )
            .into(),
        (
            "enum Direction { Up, Down = 'Down', }",
            (
                "enum Direction { Up = 0, Down = 'Down', }",
                "enum Direction { Up = 1, Down = 'Down', }",
                "enum Direction { Up = 'Up', Down = 'Down', }",
            ),
        )
            .into(),
    ];

    Tester::new(PreferEnumInitializers::NAME, PreferEnumInitializers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
