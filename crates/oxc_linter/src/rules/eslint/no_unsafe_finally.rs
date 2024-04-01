use oxc_ast::{
    ast::{BreakStatement, ContinueStatement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-unsafe-finally): Unsafe finally block")]
#[diagnostic(
    severity(warning),
    help("Control flow inside try or catch blocks will be overwritten by this statement")
)]
struct NoUnsafeFinallyDiagnostic(#[label] Span);

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeFinally;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow control flow statements in finally blocks
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript suspends the control flow statements of try and catch blocks until the execution of finally block finishes.
    /// So, when return, throw, break, or continue is used in finally, control flow statements inside try and catch are overwritten, which is considered as unexpected behavior.
    ///
    /// ### Example
    /// ```javascript
    /// // We expect this function to return 1;
    /// (() => {
    ///     try {
    ///         return 1; // 1 is returned but suspended until finally block ends
    ///     } catch(err) {
    ///         return 2;
    ///     } finally {
    ///         return 3; // 3 is returned before 1, which we did not expect
    ///     }
    /// })();
    ///
    /// // > 3
    /// ```
    NoUnsafeFinally,
    correctness
);

impl Rule for NoUnsafeFinally {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let kind = node.kind();

        let sentinel_node_type = match kind {
            AstKind::BreakStatement(stmt) if stmt.label.is_none() => SentinelNodeType::Break,
            AstKind::ContinueStatement(_) => SentinelNodeType::Continue,
            AstKind::ReturnStatement(_)
            | AstKind::ThrowStatement(_)
            | AstKind::BreakStatement(_) => SentinelNodeType::ReturnThrow,
            _ => return,
        };

        let label_name = match kind {
            AstKind::BreakStatement(BreakStatement { label, .. })
            | AstKind::ContinueStatement(ContinueStatement { label, .. }) => {
                label.as_ref().map(|label| &label.name)
            }
            _ => None,
        };

        let mut label_inside = false;
        for node_id in ctx.nodes().ancestors(node.id()) {
            let ast_kind = ctx.nodes().kind(node_id);

            if sentinel_node_type.test(ast_kind) {
                break;
            }

            let parent_kind = ctx.nodes().parent_kind(node_id);

            if let Some(AstKind::LabeledStatement(labeled_stmt)) = parent_kind {
                if label_name == Some(&labeled_stmt.label.name) {
                    label_inside = true;
                }
            }

            if let Some(AstKind::FinallyClause(_)) = parent_kind {
                if label_name.is_some() && label_inside {
                    break;
                }
                ctx.diagnostic(NoUnsafeFinallyDiagnostic(node.kind().span()));
                return;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SentinelNodeType {
    Break,
    Continue,
    ReturnThrow,
}

impl SentinelNodeType {
    fn test(self, kind: AstKind) -> bool {
        matches!(kind, AstKind::Program(_) | AstKind::FunctionBody(_) | AstKind::Class(_))
            || match self {
                Self::Break => {
                    kind.is_iteration_statement() || matches!(kind, AstKind::SwitchStatement(_))
                }
                Self::Continue => kind.is_iteration_statement(),
                Self::ReturnThrow => false,
            }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "var foo = function() {\n try { \n return 1; \n } catch(err) { \n return 2; \n } finally { \n console.log('hola!') \n } \n }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { console.log('hola!') } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { function a(x) { return x } } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { var a = function(x) { if(!x) { throw new Error() } } } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { var a = function(x) { while(true) { if(x) { break } else { continue } } } } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { var a = function(x) { label: while(true) { if(x) { break label; } else { continue } } } } }",
            None,
        ),
        ("var foo = function() { try {} finally { while (true) break; } }", None),
        ("var foo = function() { try {} finally { while (true) continue; } }", None),
        ("var foo = function() { try {} finally { switch (true) { case true: break; } } }", None),
        ("var foo = function() { try {} finally { do { break; } while (true) } }", None),
        (
            "var foo = function() { try { return 1; } catch(err) { return 2; } finally { var bar = () => { throw new Error(); }; } };",
            None,
        ),
        (
            "var foo = function() { try { return 1; } catch(err) { return 2 } finally { (x) => x } }",
            None,
        ),
        (
            "var foo = function() { try { return 1; } finally { class bar { constructor() {} static ehm() { return 'Hola!'; } } } };",
            None,
        ),
    ];

    let fail = vec![
        (
            "var foo = function() { \n try { \n return 1; \n } catch(err) { \n return 2; \n } finally { \n return 3; \n } \n }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { if(true) { return 3 } else { return 2 } } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { return 3 } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { return function(x) { return y } } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { return { x: function(c) { return c } } } }",
            None,
        ),
        (
            "var foo = function() { try { return 1 } catch(err) { return 2 } finally { throw new Error() } }",
            None,
        ),
        (
            "var foo = function() { try { foo(); } finally { try { bar(); } finally { return; } } };",
            None,
        ),
        (
            "var foo = function() { label: try { return 0; } finally { break label; } return 1; }",
            None,
        ),
        (
            "var foo = function() { \n a: try { \n return 1; \n } catch(err) { \n return 2; \n } finally { \n break a; \n } \n }",
            None,
        ),
        ("var foo = function() { while (true) try {} finally { break; } }", None),
        ("var foo = function() { while (true) try {} finally { continue; } }", None),
        ("var foo = function() { switch (true) { case true: try {} finally { break; } } }", None),
        (
            "var foo = function() { a: while (true) try {} finally { switch (true) { case true: break a; } } }",
            None,
        ),
        (
            "var foo = function() { a: while (true) try {} finally { switch (true) { case true: continue; } } }",
            None,
        ),
        (
            "var foo = function() { a: switch (true) { case true: try {} finally { switch (true) { case true: break a; } } } }",
            None,
        ),
    ];

    Tester::new(NoUnsafeFinally::NAME, pass, fail).test_and_snapshot();
}
