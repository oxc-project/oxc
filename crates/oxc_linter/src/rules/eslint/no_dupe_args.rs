use oxc_ast::{
    ast::{BindingPatternKind, FormalParameter},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_dupe_args_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(
        "Unexpected duplicate parameter names in function declarations or expressions.",
    )
    .with_help("Parameter names in function declarations or expressions should be unique.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDupeArgs;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow duplicate arguments in function definitions
    ///
    /// ### Why is this bad?
    /// If more than one parameter has the same name in a function definition, the last occurrence “shadows” the preceding occurrences. A duplicated name might be a typing error.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function foo(a, b, a) {
    ///     console.log("value of the second a:", a);
    /// }
    /// var bar = function (a, b, a) {
    ///     console.log("value of the second a:", a);
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function foo(a, b, c) {
    ///     console.log(a, b, c);
    /// }
    /// var bar = function (a, b, c) {
    ///     console.log(a, b, c);
    /// };
    /// ```
    NoDupeArgs,
    correctness,
);

impl Rule for NoDupeArgs {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) => {
                no_dupe_rags(&func.params.items, ctx);
            }
            AstKind::ArrowFunctionExpression(func) => no_dupe_rags(&func.params.items, ctx),
            _ => return,
        }
    }
}

fn no_dupe_rags<'a>(items: &'a [FormalParameter<'a>], ctx: &LintContext<'a>) {
    let mut param_name_set: FxHashSet<CompactStr> = FxHashSet::default();
    for param in items.iter() {
        if let BindingPatternKind::BindingIdentifier(iden) = &param.pattern.kind {
            let param_name: CompactStr = iden.name.to_compact_str();
            if param_name_set.contains(&param_name) {
                ctx.diagnostic(no_dupe_args_diagnostic(iden.span));
            } else {
                param_name_set.insert(param_name);
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function a(a, b, c){}",
        "var a = function(a, b, c){}",
        "function a({a, b}, {c, d}){}",      // { "ecmaVersion": 6 },
        "function a([ , a]) {}",             // { "ecmaVersion": 6 },
        "function foo([[a, b], [c, d]]) {}", // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        "function a(a, b, b) {}",
        "function a(a, a, a) {}",
        "function a(a, b, a) {}",
        "function a(a, b, a, b) {}",
        "var a = function(a, b, b) {}",
        "var a = function(a, a, a) {}",
        "var a = function(a, b, a) {}",
        "var a = function(a, b, a, b) {}",
    ];

    Tester::new(NoDupeArgs::NAME, pass, fail).test_and_snapshot();
}
