use oxc_ast::{ast::BindingPatternKind, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-next(no-assign-module-variable): Do not assign to the variable `module`.")]
#[diagnostic(
    severity(warning),
    help("See https://nextjs.org/docs/messages/no-assign-module-variable")
)]
struct NoAssignModuleVariableDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAssignModuleVariable;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoAssignModuleVariable,
    correctness
);

impl Rule for NoAssignModuleVariable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(variable_decl) = node.kind() else { return };

        for decl in &variable_decl.declarations {
            let BindingPatternKind::BindingIdentifier(binding_ident) = &decl.id.kind else {
                continue;
            };

            if binding_ident.name == "module" {
                ctx.diagnostic(NoAssignModuleVariableDiagnostic(binding_ident.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
			      let myModule = {};
			
			      export default function MyComponent() {
			        return <></>
			      }
			    ",
    ];

    let fail = vec![
        r"
			      let module = {};
			
			      export default function MyComponent() {
			        return <></>
			      }
			      ",
    ];

    Tester::new_without_config(NoAssignModuleVariable::NAME, pass, fail).test_and_snapshot();
}
