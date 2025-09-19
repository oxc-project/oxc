use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Reference;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_useless_assignment_diagnostic(name: &str, first_write: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Useless assignment to variable {name}"))
        .with_labels([first_write.label(format!("{name} is written to here but never read"))])
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessAssignment;

declare_oxc_lint!(
  /// # What it does
  ///
  /// Disallow variable assignments when the value is not used
  ///
  /// ## Why is this bad?
  ///
  /// “Dead stores” waste processing and memory, so it is better to remove unnecessary assignments to variables.
  /// Also, if the author intended the variable to be used, there is likely a mistake around the dead store.
  NoUselessAssignment,
  eslint,
  correctness
);

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let semantic = ctx.semantic();
        let symbol_table = semantic.scoping();
        for scope_id in symbol_table.scope_ancestors(symbol_table.root_scope_id()) {
            check_scope_for_dead_stores(scope_id, ctx);
        }
    }
}

fn check_scope_for_dead_stores(scope_id: oxc_semantic::ScopeId, ctx: &LintContext) {
    let semantic = ctx.semantic();
    let symbol_table = semantic.scoping();

    for symbol_id in symbol_table.iter_bindings_in(scope_id) {
        let symbol_name = symbol_table.symbol_name(symbol_id);
        let references = symbol_table.get_resolved_references(symbol_id);

        let mut last_write: Option<&Reference> = None;
        for reference in references {
            if reference.is_write() {
                if let Some(write) = last_write {
                    ctx.diagnostic(no_useless_assignment_diagnostic(
                        symbol_name,
                        semantic.reference_span(write),
                    ));
                }
                last_write = Some(reference);
            } else if reference.is_read() {
                last_write = None;
            }
        }

        if let Some(last_write) = last_write {
            ctx.diagnostic(no_useless_assignment_diagnostic(
                symbol_name,
                semantic.reference_span(last_write),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let x = 5; console.log(x);", None),
        (
            "function fn1() {
    let v = 'used';
    doSomething(v);
    v = 'used-2';
    doSomething(v);
}",
            None,
        ),
        (
            "function fn2() {
    let v = 'used';
    if (condition) {
        v = 'used-2';
        doSomething(v);
        return
    }
    doSomething(v);
}",
            None,
        ),
        (
            "function fn3() {
    let v = 'used';
    if (condition) {
        doSomething(v);
    } else {
        v = 'used-2';
        doSomething(v);
    }
}",
            None,
        ),
        (
            "function fn4() {
    let v = 'used';
    for (let i = 0; i < 10; i++) {
        doSomething(v);
        v = 'used in next iteration';
    }
}",
            None,
        ),
    ];
    let fail = vec![
        ("let x = 5; x = 7;", None),
        (
            "function fn1() {
    let v = 'used';
    doSomething(v);
    v = 'unused';
}
,",
            None,
        ),
        (
            "function fn2() {
    let v = 'used';
    if (condition) {
        v = 'unused';
        return
    }
    doSomething(v);
}",
            None,
        ),
        (
            "function fn3() {
    let v = 'used';
    if (condition) {
        doSomething(v);
    } else {
        v = 'unused';
    }
}",
            None,
        ),
        (
            "function fn4() {
    let v = 'unused';
    if (condition) {
        v = 'used';
        doSomething(v);
        return
    }
}",
            None,
        ),
        (
            "function fn5() {
    let v = 'used';
    if (condition) {
        let v = 'used';
        console.log(v);
        v = 'unused';
    }
    console.log(v);
}",
            None,
        ),
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
