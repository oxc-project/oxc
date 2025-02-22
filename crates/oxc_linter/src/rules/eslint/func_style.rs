use oxc_ast::{AstKind, ast::FunctionType};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::Span;
use rustc_hash::FxHashSet;
use serde_json::Value;

use crate::{ast_util::nth_outermost_paren_parent, context::LintContext, rule::Rule};

fn func_style_diagnostic(span: Span, style: Style) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Expected a function {}.", style.as_str()))
        .with_help("Enforce the consistent use of either `function` declarations or expressions assigned to variables")
        .with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum Style {
    #[default]
    Expression,
    Declaration,
}

impl Style {
    pub fn from(raw: &str) -> Self {
        if raw == "declaration" { Self::Declaration } else { Self::Expression }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Style::Expression => "expression",
            Style::Declaration => "declaration",
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
enum NamedExports {
    #[default]
    Ignore,
    Expression,
    Declaration,
}

impl NamedExports {
    pub fn from(raw: &str) -> Self {
        match raw {
            "expression" => Self::Expression,
            "declaration" => Self::Declaration,
            _ => Self::Ignore,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct FuncStyle {
    style: Style,
    allow_arrow_functions: bool,
    named_exports: Option<NamedExports>,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce the consistent use of either function declarations or expressions assigned to variables
    ///
    /// ### Why is this bad?
    /// This rule enforces a particular type of function style, either function declarations or expressions assigned to variables.
    /// You can specify which you prefer in the configuration.
    ///
    /// ### Examples
    /// // function declaration
    /// function doSomething() {
    ///     // ...
    /// }
    ///
    /// // arrow function expression assigned to a variable
    /// const doSomethingElse = () => {
    ///     // ...
    /// };
    ///
    /// // function expression assigned to a variable
    /// const doSomethingAgain = function() {
    ///     // ...
    /// };
    ///
    /// Examples of incorrect code for this rule with the default "expression" option:
    /// ```js
    /// /*eslint func-style: ["error", "expression"]*/
    ///
    /// function foo() {
    ///     // ...
    /// }
    /// ```
    ///
    /// Examples of incorrect code for this rule with the "declaration" option:
    /// ```js
    /// /*eslint func-style: ["error", "declaration"]*/
    /// var foo = function() {
    ///     // ...
    /// };
    ///
    /// var foo = () => {};
    /// ```
    ///
    /// Examples of incorrect code for this rule with the "declaration" and {"overrides": { "namedExports": "expression" }} option:
    /// ```js
    /// /*eslint func-style: ["error", "declaration", { "overrides": { "namedExports": "expression" } }]*/
    /// export function foo() {
    ///     // ...
    /// }
    /// ```
    ///
    /// Examples of incorrect code for this rule with the "expression" and {"overrides": { "namedExports": "declaration" }} option:
    /// ```js
    /// /*eslint func-style: ["error", "expression", { "overrides": { "namedExports": "declaration" } }]*/
    /// export var foo = function() {
    ///     // ...
    /// };
    ///
    /// export var bar = () => {};
    /// ```
    ///
    /// Examples of correct code for this rule with the default "expression" option:
    /// ```js
    /// /*eslint func-style: ["error", "expression"]*/
    /// var foo = function() {
    ///     // ...
    /// };
    ///
    /// Examples of correct code for this rule with the "declaration" option:
    /// ```js
    /// /*eslint func-style: ["error", "declaration"]*/
    /// function foo() {
    ///     // ...
    /// }
    ///  // Methods (functions assigned to objects) are not checked by this rule
    /// SomeObject.foo = function() {
    ///     // ...
    /// };
    /// ```
    ///
    /// Examples of additional correct code for this rule with the "declaration", { "allowArrowFunctions": true } options:
    /// ```js
    /// /*eslint func-style: ["error", "declaration", { "allowArrowFunctions": true }]*/
    /// var foo = () => {};
    /// ```
    ///
    /// Examples of correct code for this rule with the "declaration" and {"overrides": { "namedExports": "expression" }} option:
    /// ```js
    /// /*eslint func-style: ["error", "declaration", { "overrides": { "namedExports": "expression" } }]*/
    /// export var foo = function() {
    ///     // ...
    /// };
    /// export var bar = () => {};
    /// ```
    ///
    /// Examples of correct code for this rule with the "expression" and {"overrides": { "namedExports": "declaration" }} option:
    /// ```js
    /// /*eslint func-style: ["error", "expression", { "overrides": { "namedExports": "declaration" } }]*/
    /// export function foo() {
    ///     // ...
    /// }
    /// ```
    ///
    /// Examples of correct code for this rule with the {"overrides": { "namedExports": "ignore" }} option:
    /// ```js
    /// /*eslint func-style: ["error", "expression", { "overrides": { "namedExports": "ignore" } }]*/
    /// export var foo = function() {
    ///     // ...
    /// };
    ///
    /// export var bar = () => {};
    /// export function baz() {
    ///     // ...
    /// }
    /// ```
    FuncStyle,
    eslint,
    style,
    pending
);

fn is_ancestor_export_name_decl<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(export_decl_ancestor) = nth_outermost_paren_parent(node, ctx, 2) {
        if let AstKind::ExportNamedDeclaration(_) = export_decl_ancestor.kind() {
            return true;
        }
    }
    false
}

impl Rule for FuncStyle {
    fn from_configuration(value: Value) -> Self {
        let obj1 = value.get(0);
        let obj2 = value.get(1);

        Self {
            style: obj1.and_then(Value::as_str).map(Style::from).unwrap_or_default(),
            allow_arrow_functions: obj2
                .and_then(|v| v.get("allowArrowFunctions"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
            named_exports: obj2
                .and_then(|v| v.get("overrides"))
                .and_then(|v| v.get("namedExports"))
                .and_then(Value::as_str)
                .map(NamedExports::from),
        }
    }
    fn run_once<'a>(&self, ctx: &LintContext) {
        let semantic = ctx.semantic();
        let is_decl_style = self.style == Style::Declaration;

        // step 1
        // We can iterate over ctx.nodes() and process FunctionDeclaration and FunctionExpression,
        // whereas for ArrowFunctionExpression we need to record this and super inside it

        let mut arrow_func_nodes = Vec::new();
        let mut arrow_func_ancestor_records = FxHashSet::<NodeId>::default();

        for node in semantic.nodes() {
            match node.kind() {
                AstKind::Function(func) => {
                    let Some(parent) = semantic.nodes().parent_node(node.id()) else {
                        return;
                    };
                    match func.r#type {
                        FunctionType::FunctionDeclaration => {
                            // There are two situations to diagnostic
                            // 1) if style not equal to "declaration"
                            // we need to consider whether the parent node is ExportDefaultDeclaration or ExportNamedDeclaration
                            // "function foo() {}" should diagnostic
                            // "export function foo() {}" with option ["expression"] should diagnostic
                            //
                            // 2) For cases where the parent node is ExportNamedDeclaration,
                            // we just need to check if the self.named_exports value is expression
                            if !is_decl_style {
                                let should_diagnostic = match parent.kind() {
                                    AstKind::ExportDefaultDeclaration(_) => false,
                                    AstKind::ExportNamedDeclaration(_) => {
                                        self.named_exports.is_none()
                                    }
                                    _ => true,
                                };
                                if should_diagnostic {
                                    ctx.diagnostic(func_style_diagnostic(func.span, self.style));
                                }
                            }

                            if self.named_exports == Some(NamedExports::Expression)
                                && matches!(parent.kind(), AstKind::ExportNamedDeclaration(_))
                            {
                                ctx.diagnostic(func_style_diagnostic(func.span, self.style));
                            }
                        }
                        FunctionType::FunctionExpression => {
                            let is_ancestor_export = is_ancestor_export_name_decl(node, ctx);
                            if let AstKind::VariableDeclarator(decl) = parent.kind() {
                                if is_decl_style
                                    && (self.named_exports.is_none() || !is_ancestor_export)
                                {
                                    ctx.diagnostic(func_style_diagnostic(decl.span, self.style));
                                }

                                if self.named_exports == Some(NamedExports::Declaration)
                                    && is_ancestor_export
                                {
                                    ctx.diagnostic(func_style_diagnostic(decl.span, self.style));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                AstKind::ThisExpression(_) | AstKind::Super(_) if !self.allow_arrow_functions => {
                    // We need to determine if the recent FunctionBody is an arrow function
                    let arrow_func_ancestor = semantic
                        .nodes()
                        .ancestors(node.id())
                        .skip(1)
                        .find(|v| matches!(v.kind(), AstKind::FunctionBody(_)))
                        .map(|el| semantic.nodes().parent_node(el.id()).unwrap());
                    if let Some(ret) = arrow_func_ancestor {
                        arrow_func_ancestor_records.insert(ret.id());
                    }
                }
                AstKind::ArrowFunctionExpression(_) if !self.allow_arrow_functions => {
                    arrow_func_nodes.push(node);
                }
                _ => continue,
            }
        }

        // step 2
        // We deal with arrow functions that do not contain this and super
        for node in arrow_func_nodes {
            if !arrow_func_ancestor_records.contains(&node.id()) {
                let Some(parent) = semantic.nodes().parent_node(node.id()) else {
                    return;
                };
                if let AstKind::VariableDeclarator(decl) = parent.kind() {
                    let is_ancestor_export = is_ancestor_export_name_decl(node, ctx);
                    if is_decl_style && (self.named_exports.is_none() || !is_ancestor_export) {
                        ctx.diagnostic(func_style_diagnostic(decl.span, self.style));
                    }

                    if self.named_exports == Some(NamedExports::Declaration) && is_ancestor_export {
                        ctx.diagnostic(func_style_diagnostic(decl.span, self.style));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        (
            "function foo(){}
			 function bar(){}",
            Some(serde_json::json!(["declaration"])),
        ),
        ("foo.bar = function(){};", Some(serde_json::json!(["declaration"]))),
        ("(function() { /* code */ }());", Some(serde_json::json!(["declaration"]))),
        ("var module = (function() { return {}; }());", Some(serde_json::json!(["declaration"]))),
        ("var object = { foo: function(){} };", Some(serde_json::json!(["declaration"]))),
        ("Array.prototype.foo = function(){};", Some(serde_json::json!(["declaration"]))),
        ("foo.bar = function(){};", Some(serde_json::json!(["expression"]))),
        (
            "var foo = function(){};
			 var bar = function(){};",
            Some(serde_json::json!(["expression"])),
        ),
        (
            "var foo = () => {};
			 var bar = () => {}",
            Some(serde_json::json!(["expression"])),
        ), // { "ecmaVersion": 6 },
        ("var foo = function() { this; }.bind(this);", Some(serde_json::json!(["declaration"]))),
        ("var foo = () => { this; };", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        (
            "class C extends D { foo() { var bar = () => { super.baz(); }; } }",
            Some(serde_json::json!(["declaration"])),
        ), // { "ecmaVersion": 6 },
        (
            "var obj = { foo() { var bar = () => super.baz; } }",
            Some(serde_json::json!(["declaration"])),
        ), // { "ecmaVersion": 6 },
        ("export default function () {};", None), // { "ecmaVersion": 6, "sourceType": "module" },
        (
            "var foo = () => {};",
            Some(serde_json::json!(["declaration", { "allowArrowFunctions": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = () => { function foo() { this; } };",
            Some(serde_json::json!(["declaration", { "allowArrowFunctions": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = () => ({ bar() { super.baz(); } });",
            Some(serde_json::json!(["declaration", { "allowArrowFunctions": true }])),
        ), // { "ecmaVersion": 6 },
        ("export function foo() {};", Some(serde_json::json!(["declaration"]))),
        (
            "export function foo() {};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "export function foo() {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "export function foo() {};",
            Some(serde_json::json!(["expression", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export function foo() {};",
            Some(serde_json::json!(["declaration", { "overrides": { "namedExports": "ignore" } }])),
        ),
        ("export var foo = function(){};", Some(serde_json::json!(["expression"]))),
        (
            "export var foo = function(){};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export var foo = function(){};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export var foo = function(){};",
            Some(serde_json::json!(["declaration", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export var foo = function(){};",
            Some(serde_json::json!(["expression", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export var foo = () => {};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export var foo = () => {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export var foo = () => {};",
            Some(serde_json::json!(["declaration", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export var foo = () => {};",
            Some(serde_json::json!(["expression", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export var foo = () => {};",
            Some(
                serde_json::json!(["declaration", { "allowArrowFunctions": true, "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export var foo = () => {};",
            Some(
                serde_json::json!(["expression", { "allowArrowFunctions": true, "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export var foo = () => {};",
            Some(
                serde_json::json!(["declaration", { "allowArrowFunctions": true, "overrides": { "namedExports": "ignore" } }]),
            ),
        ),
    ];

    let fail = vec![
        ("var foo = function(){};", Some(serde_json::json!(["declaration"]))),
        ("var foo = () => {};", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        ("var foo = () => { function foo() { this; } };", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        ("var foo = () => ({ bar() { super.baz(); } });", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        ("function foo(){}", Some(serde_json::json!(["expression"]))),
        ("export function foo(){}", Some(serde_json::json!(["expression"]))),
        (
            "export function foo() {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export function foo() {};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        ("export var foo = function(){};", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        (
            "export var foo = function(){};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "export var foo = function(){};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        ("export var foo = () => {};", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        (
            "export var b = () => {};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "export var c = () => {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "function foo() {};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function() {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "expression" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = () => {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "expression" } }]),
            ),
        ), // { "ecmaVersion": 6 }
    ];

    Tester::new(FuncStyle::NAME, FuncStyle::PLUGIN, pass, fail).test_and_snapshot();
}
