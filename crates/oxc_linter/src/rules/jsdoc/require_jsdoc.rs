use oxc_semantic::SymbolId;
use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{ClassType, Expression, FunctionType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    module_record::{ExportEntry, ExportLocalName},
    rule::Rule,
};

fn require_jsdoc_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("Missing JSDoc for public export.")
        .with_help("Add a JSDoc comment for this exported API.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireJsdoc {
    config: RequireJsdocConfig,
}

#[derive(Debug, Default, Clone, Copy)]
struct RequireJsdocConfig {
    require: Require,
}
#[derive(Debug, Clone, Copy)]
struct Require {
    arrow_function_expression: bool,
    class_declaration: bool,
    class_expression: bool,
    function_declaration: bool,
    function_expression: bool,
    method_definition: bool,
}

impl Default for Require {
    fn default() -> Self {
        Self {
            arrow_function_expression: true,
            class_declaration: true,
            class_expression: true,
            function_declaration: true,
            function_expression: true,
            method_definition: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires the presence of JSDoc comments on exported classes, objects, and functions.
    ///
    /// ### Why is this bad?
    ///
    /// Public APIs without documentation reduce maintainability and discoverability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export function quux() {}
    ///
    /// export const bar = () => {};
    ///
    /// function foo() {}
    /// export { foo };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** Docs */
    /// export function quux() {}
    ///
    /// /** Docs */
    /// export const bar = () => {};
    ///
    /// /** Docs */
    /// function foo() {}
    /// export { foo };
    /// ```
    RequireJsdoc,
    jsdoc,
    pedantic
);

impl Rule for RequireJsdoc {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = RequireJsdocConfig::default();

        if let Some(require) = value
            .get(0)
            .and_then(|options| options.get("require"))
            .and_then(serde_json::Value::as_object)
        {
            for (key, value) in require {
                if let Some(flag) = value.as_bool() {
                    match key.as_str() {
                        "ArrowFunctionExpression" => {
                            config.require.arrow_function_expression = flag;
                        }
                        "ClassDeclaration" => config.require.class_declaration = flag,
                        "ClassExpression" => config.require.class_expression = flag,
                        "FunctionDeclaration" => config.require.function_declaration = flag,
                        "FunctionExpression" => config.require.function_expression = flag,
                        "MethodDefinition" => config.require.method_definition = flag,
                        _ => {}
                    }
                }
            }
        }

        Self { config }
    }

    fn run_once(&self, ctx: &LintContext) {
        let module = ctx.module_record();
        let require = self.config.require;

        // Collect locally exported symbol names and their export spans.
        let mut exported_symbols: Vec<(&str, Span)> = Vec::new();

        for ExportEntry { module_request, local_name, span, .. } in &module.local_export_entries {
            // Ignore re-exports from other modules
            if module_request.is_some() {
                continue;
            }

            match local_name {
                ExportLocalName::Name(name_span) | ExportLocalName::Default(name_span) => {
                    exported_symbols.push((name_span.name.as_str(), *span));
                }
                ExportLocalName::Null => {
                    // Cannot resolve anonymous default export or specifier-less cases.
                }
            }
        }

        // Deduplicate by symbol id
        let mut seen: FxHashSet<SymbolId> = FxHashSet::default();

        for (name, export_span) in exported_symbols {
            let Some(symbol_id) = ctx.scoping().get_root_binding(name) else { continue };
            if !seen.insert(symbol_id) {
                continue;
            }

            let decl_id = ctx.scoping().symbol_declaration(symbol_id);
            let decl_node = ctx.nodes().get_node(decl_id);

            let node_should_have_docs = match decl_node.kind() {
                AstKind::ArrowFunctionExpression(_) => require.arrow_function_expression,
                AstKind::Class(class) => match class.r#type {
                    ClassType::ClassDeclaration => require.class_declaration,
                    ClassType::ClassExpression => require.class_expression,
                },
                AstKind::Function(func) => match func.r#type {
                    FunctionType::FunctionDeclaration => require.function_declaration,
                    _ => false,
                },
                AstKind::MethodDefinition(_) => require.method_definition,
                AstKind::VariableDeclarator(decl) => decl
                    .init
                    .as_ref()
                    .is_some_and(|init| match init {
                        Expression::ArrowFunctionExpression(_) => require.arrow_function_expression,
                        Expression::ClassExpression(_) => require.class_expression,
                        Expression::FunctionExpression(_) => require.function_expression,
                        _ => false,
                    }),
                _ => false,
            };

            if node_should_have_docs && !has_any_attached_jsdoc(decl_node, ctx) {
                ctx.diagnostic(require_jsdoc_diagnostic(export_span));
            }
        }
    }
}

fn has_any_attached_jsdoc<'a>(start: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    // Walk up ancestors from the declaration node and check if any node along the way
    // has JSDoc attached. This covers common cases where docs are attached to
    // VariableDeclaration, Export*Declaration, or the Function/Class node itself.
    let mut current = start;
    loop {
        if ctx.jsdoc().get_all_by_node(ctx.nodes(), current).is_some() {
            return true;
        }

        let parent = ctx.nodes().parent_node(current.id());
        match parent.kind() {
            AstKind::Program(_) => return false,
            _ => current = parent,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let no_arrow_function_expression: std::option::Option<serde_json::Value> = serde_json::json!([
      {
        "require": {
          "ArrowFunctionExpression": false,
        },
      },
    ])
    .into();

    let no_class_declaration: std::option::Option<serde_json::Value> = serde_json::json!([
      {
        "require": {
          "ClassDeclaration": false,
        },
      },
    ])
    .into();

    let no_class_expression: std::option::Option<serde_json::Value> = serde_json::json!([
      {
        "require": {
          "ClassExpression": false,
        },
      },
    ])
    .into();

    let no_function_declaration: std::option::Option<serde_json::Value> = serde_json::json!([
      {
        "require": {
          "FunctionDeclaration": false,
        },
      },
    ])
    .into();

    let no_function_expression: std::option::Option<serde_json::Value> = serde_json::json!([
      {
        "require": {
          "FunctionExpression": false,
        },
      },
    ])
    .into();

    let pass = vec![
        ("/** Docs */\nexport const bar = () => {};", None, None),
        ("export const bar = () => {};", no_arrow_function_expression, None),
        ("/** Docs */\nexport class Foo {}", None, None),
        ("export class Foo {}", no_class_declaration, None),
        ("/** Docs */\nexport const Foo = class {}", None, None),
        ("export const Foo = class {}", no_class_expression, None),
        ("/** Docs */\nexport function quux() {}", None, None),
        ("export function quux() {}", no_function_declaration, None),
        ("/** Docs */\nexport const quux = function () {}", None, None),
        ("export const quux = function () {}", no_function_expression, None),
        ("export { foo } from 'mod';", None, None),
    ];

    let fail = vec![
        ("export const bar = () => {};", None, None),
        ("export class Foo {}", None, None),
        ("export const Foo = class {}", None, None),
        ("export function quux() {}", None, None),
        ("export const quux = function () {}", None, None),
    ];

    Tester::new(RequireJsdoc::NAME, RequireJsdoc::PLUGIN, pass, fail).test_and_snapshot();
}
