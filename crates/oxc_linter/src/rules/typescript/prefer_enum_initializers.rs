use oxc_ast::{ast::TSEnumMemberName, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    AstNode,
};

fn prefer_enum_initializers_diagnostic(
    member_name: &str,
    init: usize,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The value of the member {member_name:?} should be explicitly defined."
    ))
    .with_help(format!("Can be fixed to {member_name:?} = {init:?}."))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferEnumInitializers;

declare_oxc_lint!(
    /// ### What it does
    /// Require each enum member value to be explicitly initialized.
    ///
    /// ### Why is this bad?
    /// In projects where the value of `enum` members are important, allowing implicit values for enums can cause bugs if enums are modified over time.
    ///
    /// ### Example
    /// ```typescript
    /// // wrong, the value of `Close` is not constant
    /// enum Status {
    ///  Open = 1,
    ///  Close,
    /// }
    /// ```
    PreferEnumInitializers,
    typescript,
    pedantic,
    pending
);

impl Rule for PreferEnumInitializers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSEnumDeclaration(decl) = node.kind() else {
            return;
        };

        for (index, member) in decl.members.iter().enumerate() {
            if member.initializer.is_none() {
                if let TSEnumMemberName::Identifier(i) = &member.id {
                    ctx.diagnostic(prefer_enum_initializers_diagnostic(
                        i.name.as_str(),
                        index + 1,
                        member.span,
                    ));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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

    Tester::new(PreferEnumInitializers::NAME, PreferEnumInitializers::PLUGIN, pass, fail)
        .test_and_snapshot();
}
