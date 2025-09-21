use oxc_ast::{
    AstKind,
    ast::{BindingPatternKind, ExportDefaultDeclarationKind, Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::get_declaration_of_variable, context::LintContext, rule::Rule};

fn no_async_client_component_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prevent client components from being async functions.")
        .with_help("See: https://nextjs.org/docs/messages/no-async-client-component")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncClientComponent;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents the use of async functions for client components in Next.js applications.
    /// This rule checks for any async function that:
    /// - Is marked with "use client" directive
    /// - Has a name starting with an uppercase letter (indicating it's a component)
    /// - Is either exported as default or assigned to a variable
    ///
    /// ### Why is this bad?
    ///
    /// Using async functions for client components can cause hydration mismatches between server and client,
    /// can break component rendering lifecycle, and can lead to unexpected behavior with React's concurrent features.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// "use client"
    ///
    /// // Async component with default export
    /// export default async function MyComponent() {
    ///   return <></>
    /// }
    ///
    /// // Async component with named export
    /// async function MyComponent() {
    ///   return <></>
    /// }
    /// export default MyComponent
    ///
    /// // Async arrow function component
    /// const MyComponent = async () => {
    ///   return <></>
    /// }
    /// export default MyComponent
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// "use client"
    ///
    /// // Regular synchronous component
    /// export default function MyComponent() {
    ///   return <></>
    /// }
    ///
    /// // Handling async operations in effects
    /// export default function MyComponent() {
    ///   useEffect(() => {
    ///     async function fetchData() {
    ///       // async operations here
    ///     }
    ///     fetchData();
    ///   }, []);
    ///   return <></>
    /// }
    ///
    /// // Async operations in event handlers
    /// export default function MyComponent() {
    ///   const handleClick = async () => {
    ///     // async operations here
    ///   }
    ///   return <button onClick={handleClick}>Click me</button>
    /// }
    /// ```
    NoAsyncClientComponent,
    nextjs,
    correctness
);

impl Rule for NoAsyncClientComponent {
    fn run_once(&self, ctx: &LintContext) {
        let program = ctx.nodes().program();

        if program.directives.iter().any(|directive| directive.directive.as_str() == "use client") {
            for node in &program.body {
                let Statement::ExportDefaultDeclaration(export_default_decl) = &node else {
                    continue;
                };

                // export default async function MyComponent() {...}
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) =
                    &export_default_decl.declaration
                {
                    if func_decl.r#async
                        && func_decl
                            .id
                            .as_ref()
                            .is_some_and(|v| v.name.chars().next().unwrap().is_uppercase())
                    {
                        ctx.diagnostic(no_async_client_component_diagnostic(
                            func_decl.id.as_ref().unwrap().span,
                        ));
                    }
                    continue;
                }

                // async function MyComponent() {...}; export default MyComponent;
                let ExportDefaultDeclarationKind::Identifier(export_default_id) =
                    &export_default_decl.declaration
                else {
                    continue;
                };
                let Some(decl) = get_declaration_of_variable(export_default_id, ctx) else {
                    continue;
                };

                if let AstKind::Function(func) = decl.kind()
                    && func.r#async
                    && func
                        .id
                        .as_ref()
                        // `func.id.name` MUST be > 0 chars
                        .is_some_and(|v| v.name.chars().next().unwrap().is_uppercase())
                {
                    ctx.diagnostic(no_async_client_component_diagnostic(
                        func.id.as_ref().unwrap().span,
                    ));
                }

                let AstKind::VariableDeclarator(var_declarator) = decl.kind() else {
                    continue;
                };

                let BindingPatternKind::BindingIdentifier(binding_ident) = &var_declarator.id.kind
                else {
                    continue;
                };
                // `binding_ident.name` MUST be > 0 chars
                if binding_ident.name.chars().next().unwrap().is_uppercase()
                    && let Some(Expression::ArrowFunctionExpression(arrow_expr)) =
                        &var_declarator.init
                    && arrow_expr.r#async
                {
                    ctx.diagnostic(no_async_client_component_diagnostic(binding_ident.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
			    export default async function MyComponent() {
			      return <></>
			    }
			    ",
        r#"
			    "use client"
			
			    export default async function myFunction() {
			      return ''
			    }
			    "#,
        r"
			    async function MyComponent() {
			      return <></>
			    }
			
			    export default MyComponent
			    ",
        r#"
			    "use client"
			
			    async function myFunction() {
			      return ''
			    }
			
			    export default myFunction
			    "#,
        r#"
			    "use client"
			
			    const myFunction = () => {
			      return ''
			    }
			
			    export default myFunction
			    "#,
    ];

    let fail = vec![
        r#"
			      "use client"
			
			      export default async function MyComponent() {
			        return <></>
			      }
			      "#,
        r#"
			      "use client"
			
			      export default async function MyFunction() {
			        return ''
			      }
			      "#,
        r#"
			      "use client"
			
			      async function MyComponent() {
			        return <></>
			      }
			
			      export default MyComponent
			      "#,
        r#"
			      "use client"
			
			      async function MyFunction() {
			        return ''
			      }
			
			      export default MyFunction
			      "#,
        r#"
			      "use client"
			
			      const MyFunction = async () => {
			        return '123'
			      }
			
			      export default MyFunction
			      "#,
    ];

    Tester::new(NoAsyncClientComponent::NAME, NoAsyncClientComponent::PLUGIN, pass, fail)
        .with_nextjs_plugin(true)
        .test_and_snapshot();
}
