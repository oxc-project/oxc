use oxc_ast::{ast::MethodDefinitionKind, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_constructor_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint(no-constructor-return): Unexpected return statement in constructor.",
    )
    .with_labels([span.into()])
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
    /// Bad:
    /// ```rust
    /// class C {
    ///     constructor() { return 42; }
    /// }
    /// ```
    ///
    /// Good:
    /// ```rust
    /// class C {
    ///     constructor() { this.value = 42; }
    /// }
    /// ```
    NoConstructorReturn,
    correctness
);

impl Rule for NoConstructorReturn {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ReturnStatement(return_stmt) = node.kind() {
            let nodes = ctx.semantic().nodes();
            let mut parent_node = nodes.parent_node(node.id());
            let mut in_constructor = false;

            // Allow `return;` in constructor for flow control
            if return_stmt.argument.is_none() {
                if let Some(parent) = parent_node {
                    if let AstKind::BlockStatement(_) = parent.kind() {
                        return;
                    }
                }
            }

            while let Some(parent) = parent_node {
                match parent.kind() {
                    AstKind::MethodDefinition(method)
                        if method.kind == MethodDefinitionKind::Constructor =>
                    {
                        in_constructor = true;
                        break;
                    }
                    AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                        parent_node = nodes.parent_node(parent.id());
                        if let Some(parent) = parent_node {
                            match parent.kind() {
                                AstKind::MethodDefinition(method)
                                    if method.kind == MethodDefinitionKind::Constructor =>
                                {
                                    in_constructor = true;
                                }
                                _ => {}
                            }
                        }

                        break;
                    }
                    _ => {}
                }

                parent_node = nodes.parent_node(parent.id());
            }

            if in_constructor {
                ctx.diagnostic(no_constructor_return_diagnostic(return_stmt.span));
            }
        }
    }
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
        "class C { constructor() { { { return } } } }",
        "class C {  }",
        "class C { constructor() {} }",
        "class C { constructor() { let v } }",
        "class C { method() { return '' } }",
        "class C { get value() { return '' } }",
        "class C { constructor(a) { if (!a) { return } else { a() } } }",
        "class C { constructor() { function fn() { return true } } }",
        "class C { constructor() { this.fn = function () { return true } } }",
        "class C { constructor() { this.fn = () => { return true } } }",
    ];

    let fail = vec![
        "class C { constructor() { return } }",
        "class C { constructor() { return '' } }",
        "class C { constructor(a) { if (!a) { return '' } else { a() } } }",
    ];

    Tester::new(NoConstructorReturn::NAME, pass, fail).test_and_snapshot();
}
