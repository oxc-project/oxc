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

fn no_return_assign_diagnostic(span: Span, help: &'static str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Returned expression contains an assignment.")
        .with_label(span)
        .with_help(help)
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
    none,
    config = NoReturnAssignMode,
    version = "0.9.10",
    short_description = "Disallows assignment operators in return statements.",
);

fn is_sentinel_node(ast_kind: AstKind) -> bool {
    (ast_kind.is_statement() && !matches!(&ast_kind, AstKind::ExpressionStatement(_)))
        || matches!(
            &ast_kind,
            AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) | AstKind::Class(_)
        )
}

impl Rule for NoReturnAssign {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(_) = node.kind() else {
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

        let return_span = match parent_node.kind() {
            AstKind::ReturnStatement(stmt) => stmt.span(),
            AstKind::ArrowFunctionExpression(arrow) if arrow.is_expression() => arrow.span(),
            _ => return,
        };
        ctx.diagnostic(no_return_assign_diagnostic(return_span, self.help_message()));
    }
}

impl NoReturnAssign {
    fn help_message(&self) -> &'static str {
        match self.0 {
            NoReturnAssignMode::Always => {
                "Compute the value in a separate statement before returning it."
            }
            NoReturnAssignMode::ExceptParens => {
                "Compute the value before returning it, or wrap the assignment in parentheses to make the intent explicit."
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
        (
            r"const cache = {};
const o = {
    get x() {
        // eslint-disable-next-line no-return-assign
        return (
            cache.x ??
            (cache.x = build())
        );
    },
};",
            Some(serde_json::json!(["always"])),
        ),
        (
            r"// eslint-disable-next-line no-return-assign
const get = () => (
    cache.x ??
    (cache.x = build())
);",
            Some(serde_json::json!(["always"])),
        ),
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
        (
            r"const cache = {};
const o = {
    get x() {
        return (
            cache.x ??
            (cache.x = build())
        );
    },
};",
            Some(serde_json::json!(["always"])),
        ),
        (
            r"const get = () => (
    cache.x ??
    (cache.x = build())
);",
            Some(serde_json::json!(["always"])),
        ),
    ];

    Tester::new(NoReturnAssign::NAME, NoReturnAssign::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn invalid_configs_error_in_from_configuration() {
    // An array with an object should produce an error, since the rule only accepts a string.
    let invalid = serde_json::json!([{ "foo": "bar" }]);
    assert!(NoReturnAssign::from_configuration(invalid).is_err());

    // String that isn't one of the allowed options should produce an error
    let invalid = serde_json::json!(["foobar"]);
    assert!(NoReturnAssign::from_configuration(invalid).is_err());
    let invalid = serde_json::json!(["ExceptParens"]);
    assert!(NoReturnAssign::from_configuration(invalid).is_err());
    let invalid = serde_json::json!(["Always"]);
    assert!(NoReturnAssign::from_configuration(invalid).is_err());

    // Valid configs should not produce an error
    let valid = serde_json::json!(["except-parens"]);
    assert!(NoReturnAssign::from_configuration(valid).is_ok());
}
