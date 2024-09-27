use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_return_assign_diagnostic(span: Span, message: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string()).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoReturnAssign {
    always_disallow_assignment_in_return: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallows assignment operators in return statements
    ///
    /// ### Why is this bad?
    /// Assignment is allowed by js in return expressions, but usually, an expression with only one equal sign is intended to be a comparison.
    /// However, because of the missing equal sign, this turns to assignment, which is valid js code
    /// Because of this ambiguity, it’s considered a best practice to not use assignment in return statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// () => a = b;
    /// function x() { return a = b; }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// () => (a = b)
    /// function x() { var result = a = b; return result; }
    /// ```
    NoReturnAssign,
    style, // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    suggestion  // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

fn is_sentinel_node(ast_kind: AstKind) -> bool {
    (ast_kind.is_statement() && !matches!(&ast_kind, AstKind::ExpressionStatement(_)))
        || matches!(
            &ast_kind,
            AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) | AstKind::Class(_)
        )
}

impl Rule for NoReturnAssign {
    fn from_configuration(value: Value) -> Self {
        let always_disallow_assignment_in_return = value
            .get(0)
            .and_then(Value::as_str)
            .map_or_else(|| false, |value| value != "except-parens");
        Self { always_disallow_assignment_in_return }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AssignmentExpression(_) = node.kind() {
            if !self.always_disallow_assignment_in_return
                && ctx
                    .nodes()
                    .parent_node(node.id())
                    .is_some_and(|node| node.kind().as_parenthesized_expression().is_some())
            {
                return;
            }
            let mut parent_node = ctx.nodes().parent_node(node.id());
            while parent_node.is_some_and(|parent| !is_sentinel_node(parent.kind())) {
                parent_node = ctx.nodes().parent_node(parent_node.unwrap().id());
            }
            if let Some(parent) = parent_node {
                match parent.kind() {
                    AstKind::ReturnStatement(_) => {
                        ctx.diagnostic(no_return_assign_diagnostic(
                            parent.span(),
                            "Return statement should not contain an assignment.",
                        ));
                    }
                    AstKind::ArrowFunctionExpression(_) => {
                        ctx.diagnostic(no_return_assign_diagnostic(
                            parent.span(),
                            "Arrow function should not return an assignment.",
                        ));
                    }
                    _ => (),
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("module.exports = {'a': 1};", None), // {                "sourceType": "module"            },
        ("var result = a * b;", None),
        ("function x() { var result = a * b; return result; }", None),
        ("function x() { return (result = a * b); }", None),
        (
            "function x() { var result = a * b; return result; }",
            Some(serde_json::json!(["except-parens"])),
        ),
        ("function x() { return (result = a * b); }", Some(serde_json::json!(["except-parens"]))),
        (
            "function x() { var result = a * b; return result; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function x() { return function y() { result = a * b }; }",
            Some(serde_json::json!(["always"])),
        ),
        ("() => { return (result = a * b); }", Some(serde_json::json!(["except-parens"]))),
        ("() => (result = a * b)", Some(serde_json::json!(["except-parens"]))),
        ("const foo = (a,b,c) => ((a = b), c)", None),
        (
            "function foo(){
			            return (a = b)
			        }",
            None,
        ),
        (
            "function bar(){
			            return function foo(){
			                return (a = b) && c
			            }
			        }",
            None,
        ),
        ("const foo = (a) => (b) => (a = b)", None), // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        ("function x() { return result = a * b; };", None),
        ("function x() { return (result) = (a * b); };", None),
        ("function x() { return result = a * b; };", Some(serde_json::json!(["except-parens"]))),
        (
            "function x() { return (result) = (a * b); };",
            Some(serde_json::json!(["except-parens"])),
        ),
        ("() => { return result = a * b; }", None),
        ("() => result = a * b", None),
        ("function x() { return result = a * b; };", Some(serde_json::json!(["always"]))),
        ("function x() { return (result = a * b); };", Some(serde_json::json!(["always"]))),
        (
            "function x() { return result || (result = a * b); };",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo(){
			                return a = b
			            }",
            None,
        ),
        (
            "function doSomething() {
			                return foo = bar && foo > 0;
			            }",
            None,
        ),
        (
            "function doSomething() {
			                return foo = function(){
			                    return (bar = bar1)
			                }
			            }",
            None,
        ),
        (
            "function doSomething() {
			                return foo = () => a
			            }",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function doSomething() {
			                return () => a = () => b
			            }",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "function foo(a){
			                return function bar(b){
			                    return a = b
			                }
			            }",
            None,
        ),
        ("const foo = (a) => (b) => a = b", None), // { "ecmaVersion": 6 }
    ];

    Tester::new(NoReturnAssign::NAME, pass, fail).test_and_snapshot();
}
