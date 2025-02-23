use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, Expression, MethodDefinitionKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_redundant_constructor_init_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Explicit initialization of public members is redundant")
        .with_help("Remove the explicit initialization")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRedundantConstructorInit;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents redundant initialization of class members within a constructor.
    ///
    /// ### Why is this bad?
    ///
    /// Arguments marked as `public` within a constructor are automatically initialized.
    /// Providing an explicit initialization is redundant and can be removed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class Foo {
    ///   constructor(public name: unknown) {
    ///     this.name = name;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class Foo {
    ///   constructor(public name: unknown) {}
    /// }
    /// ```
    NoRedundantConstructorInit,
    oxc,
    correctness,
    pending,
);

impl Rule for NoRedundantConstructorInit {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::MethodDefinition(method) = node.kind() else {
            return;
        };
        if method.kind != MethodDefinitionKind::Constructor {
            return;
        }
        let public_members = method
            .value
            .params
            .items
            .iter()
            .filter_map(|param| match param.is_public() {
                true => param.pattern.get_identifier_name(),
                _ => None,
            })
            .collect::<Vec<_>>();
        if public_members.is_empty() {
            return;
        }
        let Some(body) = method.value.body.as_ref() else {
            return;
        };
        body.statements.iter().for_each(|stmt| {
            let Statement::ExpressionStatement(expr_stmt) = stmt else {
                return;
            };
            let Expression::AssignmentExpression(assignment_expr) = &expr_stmt.expression else {
                return;
            };

            // check for assigning to this: this.x = ?
            let AssignmentTarget::StaticMemberExpression(static_member_expr) =
                &assignment_expr.left
            else {
                return;
            };
            let Expression::ThisExpression(_this_expr) = &static_member_expr.object else {
                return;
            };
            let assignment_name = static_member_expr.property.name;

            // check both sides of assignment have the same name: this.x = x
            let Expression::Identifier(assignment_target_ident) = &assignment_expr.right else {
                return;
            };
            if assignment_target_ident.name != assignment_name {
                return;
            }

            // check if this was a public param
            if public_members.iter().any(|param| param == &assignment_name) {
                ctx.diagnostic(no_redundant_constructor_init_diagnostic(expr_stmt.span));
            }
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"
        class Foo {
          constructor(public name: unknown) {}
        }
        "#,
        r#"
        class Foo {
          constructor(public name: unknown, other: unknown) {
            this.other = other;
          }
        }
        "#,
        r#"
        class Foo {
          constructor(public name: unknown) {
            this.other = name;
          }
        }
        "#,
        r#"
        class Foo {
          constructor(name: unknown) {
            this.name = name;
          }
        }
        "#,
    ];

    let fail = vec![
        r#"
        class Foo {
          constructor(public name: unknown) {
            this.name = name;
          }
        }
        "#,
        r#"
        class Foo {
          constructor(other: unknown, public name: unknown) {
            this.other = other;
            this.name = name;
          }
        }
        "#,
    ];

    Tester::new(NoRedundantConstructorInit::NAME, NoRedundantConstructorInit::PLUGIN, pass, fail)
        .test_and_snapshot();
}
