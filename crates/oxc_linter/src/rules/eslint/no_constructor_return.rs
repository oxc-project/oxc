use oxc_ast::{
    ast::{MethodDefinition, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_constructor_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected return statement in constructor.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConstructorReturn;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow returning value from constructor
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, returning a value in the constructor of a class may be a mistake.
    /// Forbidding this pattern prevents mistakes resulting from unfamiliarity with the language or a copy-paste error.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```rust
    /// class C {
    ///     constructor() { return 42; }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```rust
    /// class C {
    ///     constructor() { this.value = 42; }
    /// }
    /// ```
    NoConstructorReturn,
    eslint,
    pedantic
);

impl Rule for NoConstructorReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ReturnStatement(ret) = node.kind() else { return };
        if ret.argument.is_none() {
            return;
        }

        if is_definitely_in_constructor(ctx, node.id()) {
            ctx.diagnostic(no_constructor_return_diagnostic(ret.span));
        }
    }
}

fn is_constructor(node: &AstNode<'_>) -> bool {
    matches!(
        node.kind(),
        AstKind::MethodDefinition(MethodDefinition { kind: MethodDefinitionKind::Constructor, .. })
    )
}

fn is_definitely_in_constructor(ctx: &LintContext, node_id: NodeId) -> bool {
    ctx.nodes()
        .ancestor_ids(node_id)
        .map(|id| ctx.nodes().get_node(id))
        .skip_while(|node| !node.kind().is_function_like())
        .nth(1)
        .is_some_and(is_constructor)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function fn() { return }",
        "function fn(kumiko) { if (kumiko) { return kumiko } }",
        "const fn = function () { return }",
        "const fn = function () { if (kumiko) { return kumiko } }",
        "const fn = () => { return }",
        "const fn = () => { if (kumiko) { return kumiko } }",
        "return 'Kumiko Oumae'",
        "class C {  }",
        "class C { constructor() {} }",
        "class C { constructor() { let v } }",
        "class C { method() { return '' } }",
        "class C { get value() { return '' } }",
        "class C { constructor(a) { if (!a) { return } else { a() } } }",
        "class C { constructor() { function fn() { return true } } }",
        "class C { constructor() { this.fn = function () { return true } } }",
        "class C { constructor() { this.fn = () => { return true } } }",
        "class C { constructor() { return } }",
        "class C { constructor() { { return } } }",
    ];

    let fail = vec![
        "class C { constructor() { return '' } }",
        "class C { constructor(a) { if (!a) { return '' } else { a() } } }",
    ];

    Tester::new(NoConstructorReturn::NAME, NoConstructorReturn::PLUGIN, pass, fail)
        .test_and_snapshot();
}
