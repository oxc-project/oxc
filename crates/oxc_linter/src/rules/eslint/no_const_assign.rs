use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_const_assign_diagnostic(name: &str, decl_span: Span, assign_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "eslint(no-const-assign): Unexpected re-assignment of const variable {name}"
    ))
    .with_labels([
        decl_span.label(format!("{name} is declared here as const")),
        assign_span.label(format!("{name} is re-assigned here")),
    ])
}

#[derive(Debug, Default, Clone)]
pub struct NoConstAssign;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow reassigning `const` variables.
    ///
    /// ### Why is this bad?
    /// We cannot modify variables that are declared using const keyword.
    /// It will raise a runtime error.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const a = 0;
    /// a = 1;
    ///
    /// const b = 0;
    /// b += 1;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const a = 0;
    /// console.log(a);
    ///
    /// var b = 0;
    /// b += 1;
    /// ```
    NoConstAssign,
    eslint,
    correctness
);

impl Rule for NoConstAssign {
    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let symbol_table = ctx.semantic().symbols();
        if symbol_table.get_flags(symbol_id).is_const_variable() {
            for reference in symbol_table.get_resolved_references(symbol_id) {
                if reference.is_write() {
                    ctx.diagnostic(no_const_assign_diagnostic(
                        symbol_table.get_name(symbol_id),
                        symbol_table.get_span(symbol_id),
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
        ("const x = 0; { let x; x = 1; }", None),
        ("const x = 0; function a(x) { x = 1; }", None),
        ("const x = 0; foo(x);", None),
        ("for (const x in [1,2,3]) { foo(x); }", None),
        ("for (const x of [1,2,3]) { foo(x); }", None),
        ("const x = {key: 0}; x.key = 1;", None),
        ("var x = 0; x = 1;", None),
        ("let x = 0; x = 1;", None),
        ("function x() {} x = 1;", None),
        ("function foo(x) { x = 1; }", None),
        ("class X {} X = 1;", None),
        ("try {} catch (x) { x = 1; }", None),
        ("const a = 1; { let a = 2; { a += 1; } }", None),
        ("const foo = 1;let bar;bar[foo ?? foo] = 42;", None),
        ("const FOO = 1; ({ files = FOO } = arg1); ", None),
    ];

    let fail = vec![
        ("const x = 0; x = 1;", None),
        ("const {a: x} = {a: 0}; x = 1;", None),
        ("const x = 0; ({x} = {x: 1});", None),
        ("const x = 0; ({a: x = 1} = {});", None),
        ("const x = 0; x += 1;", None),
        ("const x = 0; ++x;", None),
        ("for (const i = 0; i < 10; ++i) { foo(i); }", None),
        ("const x = 0; x = 1; x = 2;", None),
        ("const x = 0; function foo() { x = x + 1; }", None),
        ("const x = 0; function foo(a) { x = a; }", None),
        ("const x = 0; while (true) { x = x + 1; }", None),
        ("const x = 0; function foo(a) { function bar(b) { x = b; } bar(123); }", None),
        // Error even if the declaration comes after the assignment, which aligns with eslint
        ("x = 123; const x = 1;", None),
        // Binding patterns
        ("const [a, b, ...[c, ...d]] = [1, 2, 3, 4, 5]; d = 123", None),
        ("const d = 123; [a, b, ...[c, ...d]] = [1, 2, 3, 4, 5]", None),
        ("const b = 0; ({a, ...b} = {a: 1, c: 2, d: 3})", None),
    ];

    Tester::new(NoConstAssign::NAME, NoConstAssign::PLUGIN, pass, fail).test_and_snapshot();
}
