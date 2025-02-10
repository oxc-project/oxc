use oxc_ast::{ast::BindingPatternKind, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_assign_module_variable_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not assign to the variable `module`.")
        .with_help("See https://nextjs.org/docs/messages/no-assign-module-variable")
        .with_label(span)
}

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
    nextjs,
    correctness
);

impl Rule for NoAssignModuleVariable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclaration(variable_decl) = node.kind() else {
            return;
        };

        for decl in &variable_decl.declarations {
            let BindingPatternKind::BindingIdentifier(binding_ident) = &decl.id.kind else {
                continue;
            };

            if binding_ident.name == "module" {
                ctx.diagnostic(no_assign_module_variable_diagnostic(binding_ident.span));
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

    Tester::new(NoAssignModuleVariable::NAME, NoAssignModuleVariable::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
