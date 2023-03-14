use oxc_ast::{ast::BindingPatternKind, AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use rustc_hash::FxHashMap;

use crate::{ast_util::calculate_hash, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-dupe-args): Disallow duplicate arguments in function definitions")]
#[diagnostic(severity(warning), help("Consider removing the duplicated argument"))]
struct NoDupeArgsDiagnostic(#[label] pub Span, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDupeArgs;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate arguments in function definitions
    ///
    /// ### Why is this bad?
    ///
    /// If more than one parameter has the same name in a function definition,
    /// the last occurrence “shadows” the preceding occurrences. A duplicated name might be a typing error.
    ///
    /// ### Example
    /// ```javascript
    /// function foo(a, b, b) {
    ///     console.log("value of the second a:", a);
    /// }
    /// ```
    NoDupeArgs,
    correctness
);

impl Rule for NoDupeArgs {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::FormalParameters(params) = node.get().kind() {
            println!("{:#?}", node.get());
            let mut map = FxHashMap::default();
            map.reserve(params.items.len());
            for formal_param in params.items.iter() {
                let hash = calculate_hash(formal_param);
                if let Some(prev_span) = map.insert(hash, formal_param.span) {
                    ctx.diagnostic(NoDupeArgsDiagnostic(prev_span, formal_param.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function a(a, b, c){}", None),
        ("var a = function(a, b, c){}", None),
        ("function a({a, b}, {c, d}){}", None),
        ("function a([ , a]) {}", None),
        ("function foo([[a, b], [c, d]]) {}", None),
    ];

    let fail = vec![
        ("function a(a, b, b) {}", None),
        ("function a(a, a, a) {}", None),
        ("function a(a, b, a) {}", None),
        ("function a(a, b, a, b) {}", None),
        ("var a = function(a, b, b) {}", None),
        ("var a = function(a, a, a) {}", None),
        ("var a = function(a, b, a) {}", None),
        ("var a = function(a, b, a, b) {}", None),
    ];

    Tester::new(NoDupeArgs::NAME, pass, fail).test_and_snapshot();
}
