use crate::{
    ast_util::nth_outermost_paren_parent,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};
use oxc_ast::{AstKind, ast::FunctionType};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId};
use oxc_span::Span;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn func_style_diagnostic(span: Span, style: Style) -> OxcDiagnostic {
    let style_str = if style == Style::Declaration { "declaration" } else { "expression" };
    OxcDiagnostic::warn(format!("Expected a function {style_str}."))
        .with_help("Enforce the consistent use of either `function` declarations or expressions assigned to variables")
        .with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum Style {
    #[default]
    Expression,
    Declaration,
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum NamedExports {
    #[default]
    Ignore,
    Expression,
    Declaration,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FuncStyle(Style, FuncStyleConfig);

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct FuncStyleConfig {
    /// When true, arrow functions are allowed regardless of the style setting.
    allow_arrow_functions: bool,
    /// When true, functions with type annotations are allowed regardless of the style setting.
    allow_type_annotation: bool,
    /// Override the style specifically for named exports. Can be "expression", "declaration", or "ignore" (default).
    overrides: Overrides,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct Overrides {
    named_exports: Option<NamedExports>,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the consistent use of either function declarations or expressions assigned to variables
    ///
    /// ### Why is this bad?
    ///
    /// This rule enforces a particular type of function style, either function declarations or expressions assigned to variables.
    /// You can specify which you prefer in the configuration.
    ///
    /// ### Examples
    /// ```js
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
    /// ```
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
    /// ```
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
    fix = pending,
    config = FuncStyle
);

fn is_ancestor_export_name_decl<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(export_decl_ancestor) = nth_outermost_paren_parent(node, ctx, 2)
        && let AstKind::ExportNamedDeclaration(_) = export_decl_ancestor.kind()
    {
        return true;
    }
    false
}

impl Rule for FuncStyle {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<FuncStyle>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once<'a>(&self, ctx: &LintContext) {
        let FuncStyle(style, config) = &self;
        let semantic = ctx.semantic();
        let is_decl_style = style == &Style::Declaration;

        // step 1
        // We can iterate over ctx.nodes() and process FunctionDeclaration and FunctionExpression,
        // whereas for ArrowFunctionExpression we need to record this and super inside it

        let mut arrow_func_nodes = Vec::new();
        let mut arrow_func_ancestor_records = FxHashSet::<NodeId>::default();

        for node in semantic.nodes() {
            match node.kind() {
                AstKind::Function(func) => {
                    let parent = semantic.nodes().parent_node(node.id());
                    match func.r#type {
                        FunctionType::FunctionDeclaration => {
                            if func.body.is_none()
                                || func.id.as_ref().is_some_and(|id| {
                                    !ctx.scoping().symbol_redeclarations(id.symbol_id()).is_empty()
                                })
                            {
                                continue;
                            }

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
                                        config.overrides.named_exports.is_none()
                                    }
                                    _ => true,
                                };
                                if should_diagnostic {
                                    ctx.diagnostic(func_style_diagnostic(func.span, *style));
                                }
                            }

                            if config.overrides.named_exports == Some(NamedExports::Expression)
                                && matches!(parent.kind(), AstKind::ExportNamedDeclaration(_))
                            {
                                ctx.diagnostic(func_style_diagnostic(func.span, Style::Expression));
                            }
                        }
                        FunctionType::FunctionExpression => {
                            let is_ancestor_export = is_ancestor_export_name_decl(node, ctx);
                            if let AstKind::VariableDeclarator(decl) = parent.kind() {
                                let is_type_annotation = config.allow_type_annotation
                                    && decl.id.type_annotation.is_some();
                                if is_type_annotation {
                                    continue;
                                }
                                if is_decl_style
                                    && (config.overrides.named_exports.is_none()
                                        || !is_ancestor_export)
                                {
                                    ctx.diagnostic(func_style_diagnostic(decl.span, *style));
                                }

                                if config.overrides.named_exports == Some(NamedExports::Declaration)
                                    && is_ancestor_export
                                {
                                    ctx.diagnostic(func_style_diagnostic(
                                        decl.span,
                                        Style::Declaration,
                                    ));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                AstKind::ThisExpression(_) | AstKind::Super(_) if !config.allow_arrow_functions => {
                    // We need to determine if the recent FunctionBody is an arrow function
                    let arrow_func_ancestor = semantic
                        .nodes()
                        .ancestors(node.id())
                        .find(|v| matches!(v.kind(), AstKind::FunctionBody(_)))
                        .map(|el| semantic.nodes().parent_node(el.id()));
                    if let Some(ret) = arrow_func_ancestor {
                        arrow_func_ancestor_records.insert(ret.id());
                    }
                }
                AstKind::ArrowFunctionExpression(_) if !config.allow_arrow_functions => {
                    arrow_func_nodes.push(node);
                }
                _ => {}
            }
        }

        // step 2
        // We deal with arrow functions that do not contain this and super
        for node in arrow_func_nodes {
            if !arrow_func_ancestor_records.contains(&node.id()) {
                let parent = semantic.nodes().parent_node(node.id());
                if let AstKind::VariableDeclarator(decl) = parent.kind() {
                    let is_type_annotation =
                        config.allow_type_annotation && decl.id.type_annotation.is_some();
                    if is_type_annotation {
                        continue;
                    }
                    let is_ancestor_export = is_ancestor_export_name_decl(node, ctx);
                    if is_decl_style
                        && (config.overrides.named_exports.is_none() || !is_ancestor_export)
                    {
                        ctx.diagnostic(func_style_diagnostic(decl.span, Style::Declaration));
                    }

                    if config.overrides.named_exports == Some(NamedExports::Declaration)
                        && is_ancestor_export
                    {
                        ctx.diagnostic(func_style_diagnostic(decl.span, Style::Declaration));
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
                serde_json::json!(["declaration",{ "allowArrowFunctions": true, "overrides": { "namedExports": "ignore" }}]),
            ),
        ),
        ("$1: function $2() { }", Some(serde_json::json!(["declaration"]))), // { "sourceType": "script" },
        ("switch ($0) { case $1: function $2() { } }", Some(serde_json::json!(["declaration"]))),
        (
            "function foo(): void {}
             function bar(): void {}",
            Some(serde_json::json!(["declaration"])),
        ),
        ("(function(): void { /* code */ }());", Some(serde_json::json!(["declaration"]))),
        (
            "const module = (function(): { [key: string]: any } { return {}; }());",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "const object: { foo: () => void } = { foo: function(): void {} };",
            Some(serde_json::json!(["declaration"])),
        ),
        ("Array.prototype.foo = function(): void {};", Some(serde_json::json!(["declaration"]))),
        (
            "const foo: () => void = function(): void {};
             const bar: () => void = function(): void {};",
            Some(serde_json::json!(["expression"])),
        ),
        (
            "const foo: () => void = (): void => {};
             const bar: () => void = (): void => {}",
            Some(serde_json::json!(["expression"])),
        ),
        (
            "const foo: () => void = function(): void { this; }.bind(this);",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "const foo: () => void = (): void => { this; };",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "class C extends D { foo(): void { const bar: () => void = (): void => { super.baz(); }; } }",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "const obj: { foo(): void } = { foo(): void { const bar: () => void = (): void => super.baz; } }",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "const foo: () => void = (): void => {};",
            Some(serde_json::json!(["declaration", { "allowArrowFunctions": true }])),
        ),
        (
            "const foo: () => void = (): void => { function foo(): void { this; } };",
            Some(serde_json::json!(["declaration", { "allowArrowFunctions": true }])),
        ),
        (
            "const foo: () => { bar(): void } = (): { bar(): void } => ({ bar(): void { super.baz(); } });",
            Some(serde_json::json!(["declaration", { "allowArrowFunctions": true }])),
        ),
        ("export function foo(): void {};", Some(serde_json::json!(["declaration"]))),
        (
            "export function foo(): void {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "declaration" } },            ]),
            ),
        ),
        (
            "export function foo(): void {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "declaration" } },            ]),
            ),
        ),
        (
            "export function foo(): void {};",
            Some(serde_json::json!(["expression", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export function foo(): void {};",
            Some(serde_json::json!(["declaration", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(serde_json::json!(["expression"])),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(serde_json::json!(["declaration", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(serde_json::json!(["expression", { "overrides": { "namedExports": "ignore" } }])),
        ),
        (
            "const expression: Fn = function () {}",
            Some(serde_json::json!(["declaration", { "allowTypeAnnotation": true }])),
        ),
        (
            "const arrow: Fn = () => {}",
            Some(serde_json::json!(["declaration", { "allowTypeAnnotation": true }])),
        ),
        (
            "export const expression: Fn = function () {}",
            Some(serde_json::json!(["declaration", { "allowTypeAnnotation": true }])),
        ),
        (
            "export const arrow: Fn = () => {}",
            Some(serde_json::json!(["declaration", { "allowTypeAnnotation": true }])),
        ),
        (
            "export const expression: Fn = function () {}",
            Some(
                serde_json::json!(["expression", { "allowTypeAnnotation": true, "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "export const arrow: Fn = () => {}",
            Some(
                serde_json::json!(["expression", { "allowTypeAnnotation": true, "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        ("$1: function $2(): void { }", Some(serde_json::json!(["declaration"]))),
        (
            "switch ($0) { case $1: function $2(): void { } }",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "
                    function test(a: string): string;
                    function test(a: number): number;
                    function test(a: unknown) {
                      return a;
                    }
                    ",
            None,
        ),
        (
            "
                    export function test(a: string): string;
                    export function test(a: number): number;
                    export function test(a: unknown) {
                      return a;
                    }
                    ",
            None,
        ),
        (
            "
                        export function test(a: string): string;
                        export function test(a: number): number;
                        export function test(a: unknown) {
                          return a;
                        }
                        ",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "
                    switch ($0) {
                        case $1:
                        function test(a: string): string;
                        function test(a: number): number;
                        function test(a: unknown) {
                        return a;
                        }
                    }
                    ",
            None,
        ),
        (
            "
                    switch ($0) {
                        case $1:
                        function test(a: string): string;
                        break;
                        case $2:
                        function test(a: unknown) {
                        return a;
                        }
                    }
                    ",
            None,
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
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export function foo() {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        ("export var foo = function(){};", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        (
            "export var foo = function(){};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "export var foo = function(){};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        ("export var foo = () => {};", Some(serde_json::json!(["declaration"]))), // { "ecmaVersion": 6 },
        (
            "export var b = () => {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "export var c = () => {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "function foo() {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function() {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = () => {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "const foo = function() {};",
            Some(serde_json::json!(["declaration", { "allowTypeAnnotation": true }])),
        ),
        ("$1: function $2() { }", None), // { "sourceType": "script" },
        (
            "const foo = () => {};",
            Some(serde_json::json!(["declaration", { "allowTypeAnnotation": true }])),
        ),
        (
            "export const foo = function() {};",
            Some(
                serde_json::json!(["expression", { "allowTypeAnnotation": true, "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "export const foo = () => {};",
            Some(
                serde_json::json!(["expression", { "allowTypeAnnotation": true, "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        ("if (foo) function bar() {}", None), // { "sourceType": "script" },
        ("const foo: () => void = function(): void {};", Some(serde_json::json!(["declaration"]))),
        ("const foo: () => void = (): void => {};", Some(serde_json::json!(["declaration"]))),
        (
            "const foo: () => void = (): void => { function foo(): void { this; } };",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "const foo: () => { bar(): void } = (): { bar(): void } => ({ bar(): void { super.baz(); } });",
            Some(serde_json::json!(["declaration"])),
        ),
        ("function foo(): void {}", Some(serde_json::json!(["expression"]))),
        ("export function foo(): void {}", Some(serde_json::json!(["expression"]))),
        (
            "export function foo(): void {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export function foo(): void {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "export const foo: () => void = function(): void {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "export const foo: () => void = (): void => {};",
            Some(serde_json::json!(["declaration"])),
        ),
        (
            "export const b: () => void = (): void => {};",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "declaration" } } ]),
            ),
        ),
        (
            "export const c: () => void = (): void => {};",
            Some(
                serde_json::json!(["declaration", { "overrides": { "namedExports": "declaration" } } ]),
            ),
        ),
        (
            "function foo(): void {};",
            Some(
                serde_json::json!(["expression",{ "overrides": { "namedExports": "declaration" } }]),
            ),
        ),
        (
            "const foo: () => void = function(): void {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "const foo: () => void = (): void => {};",
            Some(
                serde_json::json!(["declaration",{ "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        ("$1: function $2(): void { }", None),
        ("if (foo) function bar(): string {}", None),
        (
            "
                        function test1(a: string): string;
                        function test2(a: number): number;
                        function test3(a: unknown) {
                          return a;
                        }",
            None,
        ),
        (
            "
                        export function test1(a: string): string;
                        export function test2(a: number): number;
                        export function test3(a: unknown) {
                          return a;
                        }",
            None,
        ),
        (
            "
                        export function test1(a: string): string;
                        export function test2(a: number): number;
                        export function test3(a: unknown) {
                          return a;
                        }
                        ",
            Some(
                serde_json::json!(["expression", { "overrides": { "namedExports": "expression" } }]),
            ),
        ),
        (
            "
                        switch ($0) {
                            case $1:
                            function test1(a: string): string;
                            function test2(a: number): number;
                            function test3(a: unknown) {
                                return a;
                            }
                        }
                        ",
            None,
        ),
        (
            "
                        switch ($0) {
                            case $1:
                            function test1(a: string): string;
                            break;
                            case $2:
                            function test2(a: unknown) {
                            return a;
                            }
                        }
                        ",
            None,
        ),
    ];

    Tester::new(FuncStyle::NAME, FuncStyle::PLUGIN, pass, fail).test_and_snapshot();
}
