use oxc_ast::{
    AstKind,
    ast::{MethodDefinition, MethodDefinitionKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class C {
    ///   constructor() { return 42; }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class C {
    ///   constructor() { this.value = 42; }
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
    for ancestor_id in ctx.nodes().ancestor_ids(node_id) {
        match ctx.nodes().kind(ancestor_id) {
            AstKind::Function(_) => {
                return is_constructor(ctx.nodes().parent_node(ancestor_id));
            }
            AstKind::ArrowFunctionExpression(_) => return false,
            _ => {}
        }
    }
    false
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
