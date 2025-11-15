use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::is_somewhere_inside_component_or_hook,
};

fn with_statement_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("JavaScript 'with' syntax is not supported")
        .with_help("'with' syntax is considered deprecated and removed from JavaScript standards, consider alternatives")
        .with_label(span)
}

fn eval_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The 'eval' function is not supported")
        .with_help("Eval is an anti-pattern in JavaScript, and the code executed cannot be evaluated by React Compiler")
        .with_label(span)
}

fn inline_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Inline `class` declarations are not supported")
        .with_help("Move class declarations outside of components/hooks")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UnsupportedSyntax;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates against syntax that React Compiler does not support. If you need to, you can still use this syntax outside of React, such as in a standalone utility function.
    ///
    /// ### Why is this bad?
    ///
    /// React Compiler needs to statically analyze your code to apply optimizations. Features like eval and with make it impossible to statically understand what the code does at compile time, so the compiler can’t optimize components that use them.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // ❌ Using eval in component
    /// function Component({ code }) {
    ///   const result = eval(code); // Can't be analyzed
    ///   return <div>{result}</div>;
    /// }
    ///
    /// // ❌ Using with statement
    /// function Component() {
    ///   with (Math) { // Changes scope dynamically
    ///     return <div>{sin(PI / 2)}</div>;
    ///   }
    /// }
    ///
    /// // ❌ Dynamic property access with eval
    /// function Component({propName}) {
    ///   const value = eval(`props.${propName}`);
    ///   return <div>{value}</div>;
    /// }
    ///
    /// // ❌ Inline class declaration
    /// function Component() {
    ///   class Helper {}
    ///   return <Helper />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // ✅ Use normal property access
    /// function Component({propName, props}) {
    ///   const value = props[propName]; // Analyzable
    ///   return <div>{value}</div>;
    /// }
    ///
    /// // ✅ Use standard Math methods
    /// function Component() {
    ///   return <div>{Math.sin(Math.PI / 2)}</div>;
    /// }
    /// ```
    UnsupportedSyntax,
    react,
    suspicious
);

impl Rule for UnsupportedSyntax {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::WithStatement(with_stmt) => {
                if is_somewhere_inside_component_or_hook(ctx.nodes(), node.id()) {
                    ctx.diagnostic(with_statement_diagnostic(with_stmt.span()));
                }
            }
            AstKind::IdentifierReference(ident) => {
                if ident.name == "eval"
                    && ctx.is_reference_to_global_variable(ident)
                    && is_somewhere_inside_component_or_hook(ctx.nodes(), node.id())
                {
                    ctx.diagnostic(eval_function_diagnostic(ident.span));
                }
            }
            AstKind::Class(class_decl) if class_decl.is_declaration() => {
                if is_somewhere_inside_component_or_hook(ctx.nodes(), node.id()) {
                    ctx.diagnostic(inline_class_diagnostic(class_decl.span));
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"function Component({propName, props}) {
          const value = props[propName]; // Analyzable
            return <div>{value}</div>;
          }",
        r"function Component() {
          return <div>{Math.sin(Math.PI / 2)}</div>;
          }",
        r"class Helper {}
          function Component() {
            return <Helper />;
          }",
        r"with ([1, 2, 3]) {
          console.log(toString());
          }",
    ];

    let fail = vec![
        r"function Component() {
          with ([1, 2, 3]) {
            console.log(toString());
          }
          }",
        r"function Component({ code }) {
          const result = eval(code); // Can't be analyzed
          return <div>{result}</div>;
          }",
        r"function Component() {
          class Inline {}
          return <Inline />;
          }",
        r"const useSomething = () => {
          class Inline {}
          return Inline;
          };",
        r"const Component = React.memo(() => {
          eval('test');
          });",
        r"const Component = React.forwardRef((props, ref) => {
          class Inline {}
          return <Inline />;
          });",
    ];

    Tester::new(UnsupportedSyntax::NAME, UnsupportedSyntax::PLUGIN, pass, fail).test_and_snapshot();
}
