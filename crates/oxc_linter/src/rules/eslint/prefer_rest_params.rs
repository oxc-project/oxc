use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

fn prefer_rest_params_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use the rest parameters instead of 'arguments'.").with_label(span)
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
);

impl Rule for PreferRestParams {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::IdentifierReference(identifier) = node.kind() {
            if identifier.name != "arguments"
                || !is_inside_of_function(node, ctx)
                || is_not_normal_member_access(node, ctx)
            {
                return;
            }
            let binding = ctx.scopes().find_binding(node.scope_id(), "arguments");
            if binding.is_none() {
                ctx.diagnostic(prefer_rest_params_diagnostic(node.span()));
            }
        }
    }
}

fn is_inside_of_function(node: &AstNode, ctx: &LintContext) -> bool {
    let mut current = node;
    while let Some(parent) = ctx.nodes().parent_node(current.id()) {
        if matches!(parent.kind(), AstKind::Function(_)) {
            return true;
        }
        current = parent;
    }
    false
}

fn is_not_normal_member_access(identifier: &AstNode, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(identifier.id());
    if let Some(parent) = parent {
        if let AstKind::MemberExpression(member) = parent.kind() {
            return member.object().span() == identifier.span() && !member.is_computed();
        }
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
    ];

    Tester::new(PreferRestParams::NAME, PreferRestParams::PLUGIN, pass, fail).test_and_snapshot();
}
