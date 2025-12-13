use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_return_assign_diagnostic(span: Span, message: &'static str) -> OxcDiagnostic {
    OxcDiagnostic::warn(message)
        .with_label(span)
        .with_help("Did you mean to use `==` instead of `=`?")
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoReturnAssign(NoReturnAssignMode);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoReturnAssignMode {
    /// Disallow all assignments in return statements.
    Always,
    /// Allow assignments in return statements only if they are enclosed in parentheses.
    /// This is the default mode.
    #[default]
    ExceptParens,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assignment operators in return statements.
    ///
    /// ### Why is this bad?
    ///
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
    eslint,
    style,
    pending, // TODO: add a suggestion
    config = NoReturnAssignMode,
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
        serde_json::from_value::<DefaultRuleConfig<NoReturnAssign>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assign) = node.kind() else {
            return;
        };

        // Skip if mode is ExceptParens and the assignment is parenthesized
        if matches!(self.0, NoReturnAssignMode::ExceptParens)
            && ctx.nodes().parent_kind(node.id()).as_parenthesized_expression().is_some()
        {
            return;
        }

        let mut parent_node = ctx.nodes().parent_node(node.id());
        while !is_sentinel_node(parent_node.kind()) {
            if matches!(parent_node.kind(), AstKind::Program(_)) {
                break;
            }
            parent_node = ctx.nodes().parent_node(parent_node.id());
        }

        match parent_node.kind() {
            AstKind::ReturnStatement(_) => {
                ctx.diagnostic(no_return_assign_diagnostic(
                    assign.span(),
                    "Return statements should not contain an assignment.",
                ));
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if arrow.expression {
                    ctx.diagnostic(no_return_assign_diagnostic(
                        assign.span(),
                        "Arrow functions should not return an assignment.",
                    ));
                }
            }
            _ => (),
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
        ("() => { a = b; }", None),
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

    Tester::new(NoReturnAssign::NAME, NoReturnAssign::PLUGIN, pass, fail).test_and_snapshot();
}
