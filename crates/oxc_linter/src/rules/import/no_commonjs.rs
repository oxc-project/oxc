use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_commonjs_diagnostic(span: Span, name: &str, actual: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Expected {name} instead of {actual}"))
        .with_help("Do not use CommonJS `require` calls and `module.exports` or `exports.*`")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct NoCommonjs {
    allow_primitive_modules: bool,
    allow_require: bool,
    allow_conditional_require: bool,
}

impl Default for NoCommonjs {
    fn default() -> Self {
        Self {
            allow_primitive_modules: false,
            allow_require: false,
            allow_conditional_require: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids the use of CommonJS `require` calls. Also forbids `module.exports` and `exports.*`.
    ///
    /// ### Why is this bad?
    ///
    /// ESM modules or Typescript uses `import` and `export` syntax instead of CommonJS syntax.
    /// This rule enforces the use of more modern module systems to improve maintainability and consistency across the codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var mod = require("fs");
    ///
    /// var exports = (module.exports = {});
    ///
    /// exports.sayHello = function () {
    ///   return "Hello";
    /// };
    ///
    /// module.exports = "Hola";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var a = b && require("c");
    ///
    /// if (typeof window !== "undefined") {
    ///   require("somelib");
    /// }
    ///
    /// var fs = null;
    /// try {
    ///   fs = require("fs");
    /// } catch (error) {}
    /// ```
    ///
    /// ### Allow require
    ///
    /// If `allowRequire` option is set to `true`, `require` calls are valid:
    ///
    /// ```js
    /// var mod = require("./mod");
    /// ```
    ///
    /// but `module.exports` is reported as usual.
    ///
    /// ### Allow conditional require
    ///
    /// By default, conditional requires are allowed, If the `allowConditionalRequire` option is set to `false`, they will be reported.
    ///
    /// ### Allow primitive modules
    ///
    /// If `allowPrimitiveModules` option is set to true, the following is valid:
    ///
    /// ```js
    /// module.exports = "foo";
    /// module.exports = function rule(context) {
    ///   return { /* ... */ };
    /// };
    /// ```
    ///
    /// but this is still reported:
    ///
    /// ```js
    /// module.exports = { x: "y" };
    /// exports.z = function bark() { /* ... */ };
    /// ```
    NoCommonjs,
    import,
    restriction
);

fn is_conditional(parent_node: &AstNode, ctx: &LintContext) -> bool {
    let is_cond = matches!(
        parent_node.kind(),
        AstKind::IfStatement(_)
            | AstKind::TryStatement(_)
            | AstKind::LogicalExpression(_)
            | AstKind::ConditionalExpression(_)
    );

    if is_cond {
        true
    } else {
        let parent = ctx.nodes().parent_node(parent_node.id());
        if matches!(parent.kind(), AstKind::Program(_)) {
            false
        } else {
            is_conditional(parent, ctx)
        }
    }
}
/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-commonjs.md>
impl Rule for NoCommonjs {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self {
            allow_primitive_modules: obj
                .and_then(|v| v.get("allowPrimitiveModules"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            allow_require: obj
                .and_then(|v| v.get("allowRequire"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            allow_conditional_require: obj
                .and_then(|v| v.get("allowConditionalRequire"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            member_expr if member_expr.is_member_expression_kind() => {
                // module.exports
                let Some(member_expr_kind) = member_expr.as_member_expression_kind() else {
                    return;
                };
                let Some(property_name) =
                    member_expr_kind.static_property_name().map(|s| s.as_str())
                else {
                    return;
                };

                let object = member_expr_kind.object();

                if object.is_specific_id("module") && property_name == "exports" {
                    let Some(parent_node) = ctx.nodes().ancestors(node.id()).nth(1) else {
                        return;
                    };

                    if !self.allow_primitive_modules {
                        ctx.diagnostic(no_commonjs_diagnostic(
                            member_expr_kind.span(),
                            "export",
                            property_name,
                        ));
                    }

                    match parent_node.kind() {
                        AstKind::ExpressionStatement(expr_statement)
                            if matches!(
                                expr_statement.expression,
                                Expression::AssignmentExpression(_)
                            ) =>
                        {
                            match &expr_statement.expression {
                                Expression::AssignmentExpression(assign_expr) => {
                                    if !assign_expr.right.is_function()
                                        && !assign_expr.right.is_string_literal()
                                    {
                                        ctx.diagnostic(no_commonjs_diagnostic(
                                            member_expr_kind.span(),
                                            "export",
                                            property_name,
                                        ));
                                    }

                                    return;
                                }
                                _ => return,
                            }
                        }
                        AstKind::AssignmentExpression(assignment_expr) => {
                            if let Expression::ObjectExpression(_object_expr) =
                                &assignment_expr.right.without_parentheses()
                            {
                                ctx.diagnostic(no_commonjs_diagnostic(
                                    member_expr_kind.span(),
                                    "export",
                                    property_name,
                                ));
                            } else {
                                return;
                            }
                        }
                        _ => {
                            ctx.diagnostic(no_commonjs_diagnostic(
                                member_expr_kind.span(),
                                "export",
                                property_name,
                            ));
                            return;
                        }
                    }
                }

                // exports.
                if object.is_specific_id("exports") {
                    if node.scope_id() != ctx.scoping().root_scope_id() {
                        return;
                    }

                    ctx.diagnostic(no_commonjs_diagnostic(
                        member_expr_kind.span(),
                        "export",
                        property_name,
                    ));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if self.allow_conditional_require
                    && node.scope_id() != ctx.scoping().root_scope_id()
                {
                    return;
                }

                if !call_expr.is_require_call() {
                    return;
                }

                if ctx.scoping().find_binding(ctx.scoping().root_scope_id(), "require").is_some() {
                    return;
                }

                if let Argument::TemplateLiteral(template_literal) = &call_expr.arguments[0]
                    && !template_literal.expressions.is_empty()
                {
                    return;
                }

                if self.allow_require {
                    return;
                }

                let parent_node = ctx.nodes().parent_node(node.id());

                if self.allow_conditional_require && is_conditional(parent_node, ctx) {
                    return;
                }

                let Some(callee_name) = call_expr.callee_name() else {
                    return;
                };

                ctx.diagnostic(no_commonjs_diagnostic(call_expr.span, "import", callee_name));
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"import "x";"#, None),
        (r#"import x from "x""#, None),
        (r#"import { x } from "x""#, None),
        (r#"export default "x""#, None),
        ("export function house() {}", None),
        (
            "
                    function someFunc() {
                      const exports = someComputation();
                      expect(exports.someProp).toEqual({ a: 'value' });
                    }
                  ",
            None,
        ),
        (r#"function a() { var x = require("y"); }"#, None),
        (r#"var a = c && require("b")"#, None),
        (r#"require.resolve("help")"#, None),
        ("require.ensure([])", None),
        ("require([], function(a, b, c) {})", None),
        ("var bar = require('./bar', true);", None),
        ("var bar = proxyquire('./bar');", None),
        ("var bar = require('./bar' + 'code');", None),
        ("var bar = require(`x${1}`);", None),
        ("var zero = require(0);", None),
        (r#"require("x")"#, Some(json!([{ "allowRequire": true }]))),
        (r#"require(rootRequire("x"))"#, Some(json!([{ "allowRequire": true }]))),
        (r#"require(String("x"))"#, Some(json!([{ "allowRequire": true }]))),
        (r#"require(["x", "y", "z"].join("/"))"#, Some(json!([{ "allowRequire": true }]))),
        (r#"rootRequire("x")"#, Some(json!([{ "allowRequire": true }]))),
        (r#"rootRequire("x")"#, Some(json!([{ "allowRequire": false }]))),
        ("module.exports = function () {}", Some(json!([{ "allowPrimitiveModules": true }]))),
        (r#"module.exports = "foo""#, Some(json!([{ "allowPrimitiveModules": true }]))),
        (
            r#"if (typeof window !== "undefined") require("x")"#,
            Some(json!([{ "allowRequire": true }])),
        ),
        (
            r#"if (typeof window !== "undefined") require("x")"#,
            Some(json!([{ "allowRequire": false }])),
        ),
        (
            r#"if (typeof window !== "undefined") { require("x") }"#,
            Some(json!([{ "allowRequire": true }])),
        ),
        (
            r#"if (typeof window !== "undefined") { require("x") }"#,
            Some(json!([{ "allowRequire": false }])),
        ),
        (r#"try { require("x") } catch (error) {}"#, None),
        // covers user variables
        (
            "
            import { createRequire } from 'module';
            const require = createRequire();
            require('remark-preset-prettier');
            ",
            None,
        ),
    ];

    let fail = vec![
        (r"module.exports = {}", None),
        (r#"var x = require("x")"#, None),
        (r#"require("x")"#, None),
        (r"require(`x`)", None),
        (
            r#"if (typeof window !== "undefined") require("x")"#,
            Some(json!([{ "allowConditionalRequire": false }])),
        ),
        (
            r#"if (typeof window !== "undefined") { require("x") }"#,
            Some(json!([{ "allowConditionalRequire": false }])),
        ),
        (
            r#"try { require("x") } catch (error) {}"#,
            Some(json!([{ "allowConditionalRequire": false }])),
        ),
        // exports
        (r#"exports.face = "palm""#, None),
        (r#"module.exports.face = "palm""#, None),
        (r"module.exports = face", None),
        (r"exports = module.exports = {}", None),
        (r"var x = module.exports = {}", None),
        (r"module.exports = {}", Some(json!([{ "allowPrimitiveModules": true }]))),
        (r"var x = module.exports", Some(json!([{ "allowPrimitiveModules": true }]))),
    ];

    Tester::new(NoCommonjs::NAME, NoCommonjs::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
