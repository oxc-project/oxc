use oxc_ast::{ast::FormalParameter, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn default_param_last_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Default parameters should be last")
        .with_help("Enforce default parameters to be last.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct DefaultParamLast;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce default parameters to be last
    ///
    /// ### Why is this bad?
    /// Putting default parameter at last allows function calls to omit optional tail arguments.
    ///
    /// ### Example
    /// ```javascript
    /// // Correct: optional argument can be omitted
    /// function createUser(id, isAdmin = false) {}
    /// createUser("tabby")
    ///
    /// // Incorrect: optional argument can **not** be omitted
    /// function createUser(isAdmin = false, id) {}
    /// createUser(undefined, "tabby")
    /// ```
    DefaultParamLast,
    style
);

impl Rule for DefaultParamLast {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(function) => {
                if !function.is_declaration() && !function.is_expression() {
                    return;
                }
                check_params(&function.params.items, ctx);
            }
            AstKind::ArrowFunctionExpression(function) => check_params(&function.params.items, ctx),
            _ => {}
        }
    }
}

fn check_params<'a>(items: &'a [FormalParameter<'a>], ctx: &LintContext<'a>) {
    let mut has_seen_plain_param = false;
    for param in items.iter().rev() {
        if !param.pattern.kind.is_assignment_pattern() {
            has_seen_plain_param = true;
            continue;
        }
        if has_seen_plain_param && param.pattern.kind.is_assignment_pattern() {
            ctx.diagnostic(default_param_last_diagnostic(param.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function f() {}",
        "function f(a) {}",
        "function f(a = 5) {}",
        "function f(a, b) {}",
        "function f(a, b = 5) {}",
        "function f(a, b = 5, c = 5) {}",
        "function f(a, b = 5, ...c) {}",
        "const f = () => {}",
        "const f = (a) => {}",
        "const f = (a = 5) => {}",
        "const f = function f() {}",
        "const f = function f(a) {}",
        "const f = function f(a = 5) {}",
    ];

    let fail = vec![
        "function f(a = 5, b) {}",
        "function f(a = 5, b = 6, c) {}",
        "function f (a = 5, b, c = 6, d) {}",
        "function f(a = 5, b, c = 5) {}",
        "const f = (a = 5, b, ...c) => {}",
        "const f = function f (a, b = 5, c) {}",
        "const f = (a = 5, { b }) => {}",
        "const f = ({ a } = {}, b) => {}",
        "const f = ({ a, b } = { a: 1, b: 2 }, c) => {}",
        "const f = ([a] = [], b) => {}",
        "const f = ([a, b] = [1, 2], c) => {}",
    ];

    Tester::new(DefaultParamLast::NAME, pass, fail).test_and_snapshot();
}
