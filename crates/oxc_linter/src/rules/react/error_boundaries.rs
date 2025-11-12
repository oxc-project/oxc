use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn error_boundaries_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid constructing JSX within try/catch")
        .with_help(
            "React does not immediately render components when JSX is rendered, so any errors from this component will not be caught by the try/catch. To catch errors in rendering a given component, wrap that component in an error boundary. (https://react.dev/reference/react/Component#catching-rendering-errors-with-an-error-boundary)",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ErrorBoundaries;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates usage of Error Boundaries instead of try/catch for errors in child components.
    ///
    /// ### Why is this bad?
    ///
    /// Try/catch blocks can’t catch errors that happen during React’s rendering process. Errors thrown in rendering methods or hooks bubble up through the component tree. Only Error Boundaries can catch these errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Parent() {
    ///   try {
    ///     return <ChildComponent />;
    ///   } catch (error) {
    ///     return <div>Error occurred</div>;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Parent() {
    ///   try {
    ///     return renderChild();
    ///   } catch (error) {
    ///     return <Fallback error={error} />;
    ///   }
    /// }
    /// ```
    ErrorBoundaries,
    react,
    correctness,
);

impl Rule for ErrorBoundaries {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(_) | AstKind::JSXFragment(_) => {
                if is_blocked_jsx(node, ctx) {
                    ctx.diagnostic(error_boundaries_diagnostic(node.kind().span()));
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn is_blocked_jsx<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let span = node.kind().span();

    for ancestor in ctx.nodes().ancestors(node.id()) {
        let AstKind::TryStatement(try_stmt) = ancestor.kind() else { continue };

        // JSX is inside the `try { ... }` block
        if try_stmt.block.span.contains_inclusive(span) {
            return true;
        }

        // JSX is in the `catch` body of a nested `try { ... } catch { ... }`
        if let Some(handler) = &try_stmt.handler
            && handler.body.span.contains_inclusive(span)
            && is_nested_try(ctx, ancestor)
        {
            return true;
        }
    }

    false
}

fn is_nested_try<'a>(ctx: &LintContext<'a>, try_node: &AstNode<'a>) -> bool {
    ctx.nodes()
        .ancestors(try_node.id())
        .any(|ancestor| matches!(ancestor.kind(), AstKind::TryStatement(_)))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r"function Parent() {
              return (
                <ErrorBoundary>
                  <ChildComponent />
                </ErrorBoundary>
              );
            }",
            None,
        ),
        (
            r"function Parent() {
              try {
                risky();
              } catch (error) {
                return <Fallback error={error} />;
              }
              return null;
            }",
            None,
        ),
        (
            r"function App() {
              return (
                <ErrorBoundary fallback={<div>Failed to load</div>}>
                  <Suspense fallback={<div>Loading...</div>}>
                    <DataComponent promise={fetchData()} />
                  </Suspense>
                </ErrorBoundary>
              );
            }",
            None,
        ),
    ];

    let fail = vec![
        (
            r"function Parent() {
              try {
                return <ChildComponent />; // If this throws, catch won't help
              } catch (error) {
                return <div>Error occurred</div>;
              }
            }",
            None,
        ),
        (
            r"function Nested() {
              try {
                try {
                  doSomething();
                } catch (error) {
                  return <>
                    <Fallback />
                  </>;
                }
              } catch (outer) {
                handle(outer);
              }
            }",
            None,
        ),
        (
            r"function Component({promise}) {
              try {
                const data = use(promise);
                return <div>{data}</div>;
              } catch (error) {
                return <div>Failed to load</div>;
              }
            }",
            None,
        ),
    ];

    Tester::new(ErrorBoundaries::NAME, ErrorBoundaries::PLUGIN, pass, fail).test_and_snapshot();
}
