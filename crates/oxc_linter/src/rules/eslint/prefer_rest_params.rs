use crate::{AstNode, context::LintContext, rule::Rule};
use oxc_ast::{AstKind, AstType};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn prefer_rest_params_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use the rest parameters instead of `arguments`.")
        .with_help("Replace `arguments` with rest parameters (`...args`).")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferRestParams;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of the `arguments` object and instead enforces the use of rest parameters.
    ///
    /// ### Why is this bad?
    ///
    /// The `arguments` object does not have methods from `Array.prototype`, making it inconvenient for array-like operations.
    /// Using rest parameters provides a more intuitive and efficient way to handle variadic arguments.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() {
    ///     console.log(arguments);
    /// }
    ///
    /// function foo(action) {
    ///     var args = Array.prototype.slice.call(arguments, 1);
    ///     action.apply(null, args);
    /// }
    ///
    /// function foo(action) {
    ///     var args = [].slice.call(arguments, 1);
    ///     action.apply(null, args);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo(...args) {
    ///     console.log(args);
    /// }
    ///
    /// function foo(action, ...args) {
    ///     action.apply(null, args); // Or use `action(...args)` (related to `prefer-spread` rule).
    /// }
    ///
    /// // Note: Implicit `arguments` can be shadowed.
    /// function foo(arguments) {
    ///     console.log(arguments); // This refers to the first argument.
    /// }
    /// function foo() {
    ///     var arguments = 0;
    ///     console.log(arguments); // This is a local variable.
    /// }
    /// ```
    PreferRestParams,
    eslint,
    style,
    version = "0.15.4",
    short_description = "Disallows the use of the `arguments` object and instead enforces the use of rest parameters.",
);

impl Rule for PreferRestParams {
    fn run_once(&self, ctx: &LintContext) {
        let Some(references) = ctx.scoping().root_unresolved_references().get("arguments") else {
            return;
        };

        for &reference_id in references {
            let reference = ctx.scoping().get_reference(reference_id);
            let reference_node = ctx.nodes().get_node(reference.node_id());

            // Only references to the implicit `arguments` object are relevant, i.e.
            // those inside a (non-arrow) function.
            let in_function = ctx
                .nodes()
                .ancestors(reference_node.id())
                .any(|ancestor| matches!(ancestor.kind(), AstKind::Function(_)));
            if !in_function {
                continue;
            }

            if !is_normal_member_access(reference_node, ctx) {
                ctx.diagnostic(prefer_rest_params_diagnostic(reference_node.span()));
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.semantic().nodes().contains(AstType::Function)
    }
}

fn is_normal_member_access(identifier: &AstNode, ctx: &LintContext) -> bool {
    if let AstKind::StaticMemberExpression(member) = ctx.nodes().parent_kind(identifier.id()) {
        return member.object.span() == identifier.span();
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "arguments;",
        "function foo(arguments) { arguments; }",
        "function foo() { var arguments; arguments; }",
        "var foo = () => arguments;",
        "function foo(...args) { args; }",
        "function foo() { arguments.length; }",
        "function foo() { arguments.callee; }",
    ];

    let fail = vec![
        "function foo() { arguments; }",
        "function foo() { arguments[0]; }",
        "function foo() { arguments[1]; }",
        "function foo() { arguments[Symbol.iterator]; }",
        "function foo() { function bar() { arguments; } }",
        "function foo() { var bar = () => arguments; }",
    ];

    Tester::new(PreferRestParams::NAME, PreferRestParams::PLUGIN, pass, fail).test_and_snapshot();
}
