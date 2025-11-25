use oxc_ast::{AstKind, ast::BindingPatternKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// Prevents the assignment or declaration of variables named `module` in Next.js applications.
    ///
    /// ### Why is this bad?
    ///
    /// The variable name `module` is reserved in Next.js for internal use and module system
    /// functionality. Declaring your own `module` variable can conflict with Next.js's internal
    /// module system, lead to unexpected behavior in your application, and cause issues with code
    /// splitting and hot module replacement.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // Declaring module variable
    /// let module = {};
    ///
    /// // Using module in variable declaration
    /// const module = {
    ///   exports: {}
    /// };
    ///
    /// // Assigning to module
    /// module = { id: 'my-module' };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // Use a different variable name
    /// let myModule = {};
    ///
    /// // Use a more descriptive name
    /// const customModule = {
    ///   exports: {}
    /// };
    ///
    /// // Access actual module object (when available)
    /// console.log(module.exports);
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
        .test_and_snapshot();
}
