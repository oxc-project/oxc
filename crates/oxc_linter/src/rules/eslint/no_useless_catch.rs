use oxc_ast::{
    ast::{BindingPatternKind, Expression, Statement},
    AstKind,
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_useless_catch_diagnostic(span0: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-useless-catch): Unnecessary try/catch wrapper").with_labels([
        LabeledSpan::new_with_span(Some("is caught here".into()), span0),
        LabeledSpan::new_with_span(Some("and re-thrown here".into()), span1),
    ])
}

fn no_useless_catch_finalizer_diagnostic(span0: Span, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-useless-catch): Unnecessary catch clause").with_labels([
        LabeledSpan::new_with_span(Some("is caught here".into()), span0),
        LabeledSpan::new_with_span(Some("and re-thrown here".into()), span1),
    ])
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessCatch;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary catch clauses
    ///
    /// ### Why is this bad?
    ///
    /// A catch clause that only rethrows the original error is redundant,
    /// and has no effect on the runtime behavior of the program.
    /// These redundant clauses can be a source of confusion and code bloat,
    /// so it’s better to disallow these unnecessary catch clauses.
    ///
    /// ### Example
    /// ```javascript
    /// try {
    ///   doSomethingThatMightThrow();
    /// } catch (e) {
    ///   throw e;
    /// }
    /// ```
    NoUselessCatch,
    correctness
);

impl Rule for NoUselessCatch {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TryStatement(try_stmt) = node.kind() else {
            return;
        };
        let Some(catch_clause) = &try_stmt.handler else {
            return;
        };
        let Some(BindingPatternKind::BindingIdentifier(binding_ident)) =
            catch_clause.param.as_ref().map(|param| &param.pattern.kind)
        else {
            return;
        };
        let Some(Statement::ThrowStatement(throw_stmt)) = catch_clause.body.body.first() else {
            return;
        };
        let Expression::Identifier(throw_ident) = &throw_stmt.argument else {
            return;
        };
        if binding_ident.name == throw_ident.name {
            if try_stmt.finalizer.is_some() {
                ctx.diagnostic(no_useless_catch_finalizer_diagnostic(
                    binding_ident.span,
                    throw_stmt.span,
                ));
            } else {
                ctx.diagnostic(no_useless_catch_diagnostic(binding_ident.span, throw_stmt.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
      try {
        foo();
      } catch (err) {
        console.error(err);
      }
    ",
        "
      try {
        foo();
      } catch (err) {
        console.error(err);
      } finally {
        bar();
      }
    ",
        "
      try {
        foo();
      } catch (err) {
        doSomethingBeforeRethrow();
        throw err;
      }
    ",
        "
      try {
        foo();
      } catch (err) {
        throw err.msg;
      }
    ",
        "
      try {
        foo();
      } catch (err) {
        throw new Error('whoops!');
      }
    ",
        "
      try {
        foo();
      } catch (err) {
        throw bar;
      }
    ",
        "
      try {
        foo();
      } catch (err) { }
    ",
        "
        try {
          foo();
        } catch ({ err }) {
          throw err;
        }
      ",
        "
        try {
          foo();
        } catch ([ err ]) {
          throw err;
        }
      ",
        "
        async () => {
          try {
            await doSomething();
          } catch (e) {
            doSomethingAfterCatch();
            throw e;
          }
        }
      ",
        "
        try {
          throw new Error('foo');
        } catch {
          throw new Error('foo');
        }
      ",
    ];

    let fail = vec![
        "
        try {
          foo();
        } catch (err) {
          throw err;
        }
      ",
        "
        try {
          foo();
        } catch (err) {
          throw err;
        } finally {
          foo();
        }
      ",
        "
        try {
          foo();
        } catch (err) {
          /* some comment */
          throw err;
        }
      ",
        "
        try {
          foo();
        } catch (err) {
          /* some comment */
          throw err;
        } finally {
          foo();
        }
      ",
        "
        async () => {
          try {
            await doSomething();
          } catch (e) {
            throw e;
          }
        }
      ",
    ];

    Tester::new(NoUselessCatch::NAME, pass, fail).test_and_snapshot();
}
