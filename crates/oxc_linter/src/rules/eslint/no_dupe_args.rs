use oxc_ast::{AstKind, ast::{BindingPatternKind, Function}};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Span, Atom};
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-dupe-args): Disallow duplicate arguments in function definitions")]
#[diagnostic(severity(warning))]
struct NoDupeArgsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDupeArgs;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow duplicate arguments in function definitions
    ///
    /// ### Why is this bad?
    /// If more than one parameter has the same name in a function definition,
    /// the last occurrence “shadows” the preceding occurrences. 
    /// A duplicated name might be a typing error.
    ///
    /// ### Example
    /// ```javascript
    /// function foo(a, b, a) {
    ///     console.log("value of the second a:", a);
    /// }
    /// 
    /// var bar = function (a, b, a) {
    ///     console.log("value of the second a:", a);
    /// }
    /// ```
    NoDupeArgs,
    correctness
);


fn has_dupe_args(params: &Function) -> bool {
    let items = &params.params.items;

    let mut seen: FxHashMap<Atom, u32> = FxHashMap::default();

    for item in items {
        if let BindingPatternKind::BindingIdentifier(ident) = &item.pattern.kind {
            let count = seen.get(&ident.name);

            match count {
                Some(cnt) => {
                  seen.insert(ident.name.clone(), cnt + 1);
                }
                None => {
                  seen.insert(ident.name.clone(), 1);
                }
            }
        }
    }

    seen.values().any(|v| *v > 1)
}

impl Rule for NoDupeArgs {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::Function(stmt) = node.kind() {

            if has_dupe_args(stmt) {
                ctx.diagnostic(NoDupeArgsDiagnostic(stmt.params.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
      ("function a(a, b, c){}", None),
      ("function a({a, b}, {c, d}){}", None),
      ("function a([ , a]) {}", None),
      ("function foo([[a, b], [c, d]]) {}", None)
    ];

    let fail = vec![
      ("function a(a, b, b) {}", None),
      ("function a(a, a, a) {}", None),
      ("function a(a, b, a) {}", None),
      ("function a(a, b, a, b) {}", None),
      ("var a = function(a, b, b) {}", None),
      ("var a = function(a, a, a) {}", None),
      ("var a = function(a, b, a) {}", None),
      ("var a = function(a, b, a, b) {}", None)
    ];

    Tester::new(NoDupeArgs::NAME, pass, fail).test_and_snapshot();
}
