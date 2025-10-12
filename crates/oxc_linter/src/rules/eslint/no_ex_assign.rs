use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_ex_assign_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not assign to the exception parameter.").with_help("If a catch clause in a try statement accidentally (or purposely) assigns another value to the exception parameter, it is impossible to refer to the error from that point on. Since there is no arguments object to offer alternative access to this data, assignment of the parameter is absolutely destructive.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExAssign;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow reassigning exceptions in catch clauses
    ///
    /// ### Why is this bad?
    ///
    /// If a catch clause in a try statement accidentally
    /// (or purposely) assigns another value to the exception parameter,
    /// it is impossible to refer to the error from that point on.
    /// Since there is no arguments object to offer alternative access to this data,
    /// assignment of the parameter is absolutely destructive.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// try {
    ///     // code
    /// } catch (e) {
    ///     e = 10;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// try {
    ///     // code
    /// } catch (e) {
    ///     let val = 10;
    /// }
    /// ```
    NoExAssign,
    eslint,
    correctness
);

impl Rule for NoExAssign {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CatchParameter(catch_param) = node.kind() else {
            return;
        };

        let idents = catch_param.pattern.get_binding_identifiers();
        let symbol_table = ctx.scoping();
        for ident in idents {
            let symbol_id = ident.symbol_id();
            // This symbol _should_ always be considered a catch variable (since we got it from a catch param),
            // but we check in debug mode just to be sure.
            debug_assert!(symbol_table.symbol_flags(symbol_id).is_catch_variable());
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    ctx.diagnostic(no_ex_assign_diagnostic(
                        ctx.semantic().reference_span(reference),
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

    Tester::new(NoExAssign::NAME, NoExAssign::PLUGIN, pass, fail).test_and_snapshot();
}
