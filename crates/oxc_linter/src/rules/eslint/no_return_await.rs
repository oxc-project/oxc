use oxc_diagnostics::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-return-await): Redundant use of `await` on a return value.")]
#[diagnostic(severity(warning), help("Remove redundant `await`."))]
struct NoReturnAwaitDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoReturnAwait;

declare_oxc_lint!(
  /// ### What it does
  /// Disallow unnecessary return await
  ///
  /// ### Why is this bad?
  /// This rule aims to prevent a likely common performance hazard due to a lack of understanding of the semantics of async function.
  /// https://eslint.org/docs/latest/rules/no-return-await
  ///
  /// ### Example
  /// ```javascript
  /// async function foo() {
  ///   return await bar();
  /// }
  /// ```
  NoReturnAwait,
  correctness
);

impl Rule for NoReturnAwait {
  fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
      let symbol_table = ctx.semantic().symbols();
      if symbol_table.get_flag(symbol_id).is_const_variable() {
          for reference_id in symbol_table.get_resolved_references(symbol_id) {
              let reference = symbol_table.get_reference(*reference_id);
              if reference.is_write() {
                  ctx.diagnostic(NoReturnAwaitDiagnostic(
                      symbol_table.get_name(symbol_id).clone(),
                      symbol_table.get_span(symbol_id),
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
    ("async function foo() { return bar(); }", None),
    ("async function foo() { await bar(); return; }", None),
    // This is essentially the same as `return await bar();`, but the rule checks only `await` in `return` statements
    ("async function foo() { const x = await bar(); return x; }", None),
    // In this example the `await` is necessary to be able to catch errors thrown from `bar()`
    ("async function foo() { try { return await bar(); } catch (error) { } }", None),
  ];

  let fail = vec![
    ("async function foo() { return await bar(); }", None),
  ];

  Tester::new(NoReturnAwait::NAME, pass, fail).test_and_snapshot();
}
