use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-ex-assign): Do not assign to the exception parameter.")]
#[diagnostic(severity(warning))]
struct NoExAssignDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoExAssign;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow reassigning exceptions in catch clauses
    ///
    /// ### Why is this bad?
    /// If a catch clause in a try statement accidentally 
    /// (or purposely) assigns another value to the exception parameter, 
    /// it is impossible to refer to the error from that point on. 
    /// Since there is no arguments object to offer alternative access to this data, 
    /// assignment of the parameter is absolutely destructive.
    ///
    /// ### Example
    /// ```javascript
    // try {
    //     // code
    // } catch (e) {
    //     e = 10;
    // }
    /// ```
    NoExAssign,
    correctness
);

impl Rule for NoExAssign {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol_table = ctx.semantic().symbols();
        if symbol_table.get_flag(symbol_id).is_catch_variable() {
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    ctx.diagnostic(NoExAssignDiagnostic(
                        reference.span(),
                    ));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("try { } catch (e) { three = 2 + 1; }", None),
        ("try { } catch ({e}) { this.something = 2; }", None),
        ("function foo() { try { } catch (e) { return false; } }", None),
    ];

    let fail = vec![
        ("try { } catch (e) { e = 10; }", None),
        ("try { } catch (ex) { ex = 10; }", None),
        ("try { } catch (ex) { [ex] = []; }", None),
        ("try { } catch (ex) { ({x: ex = 0} = {}); }", None),
        ("try { } catch ({message}) { message = 10; }", None),
    ];

    Tester::new(NoExAssign::NAME, pass, fail).test_and_snapshot();
}
