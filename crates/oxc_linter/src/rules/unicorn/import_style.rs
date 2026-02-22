use indexmap::IndexMap;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, Expression, ImportDeclaration, ImportDeclarationSpecifier,
        PropertyKey, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const STYLE_UNASSIGNED: &str = "unassigned";
const STYLE_DEFAULT: &str = "default";
const STYLE_NAMESPACE: &str = "namespace";
const STYLE_NAMED: &str = "named";

fn import_style_diagnostic(span: Span, allowed_styles: &str, module_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use {allowed_styles} import for module `{module_name}`."))
        .with_label(span)
}

/// Configuration for the `unicorn/import-style` rule.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportStyleOptions {
    /// Check static `import ... from "module"` declarations.
    check_import: bool,
    /// Check `import("module")` and `await import("module")`.
    check_dynamic_import: bool,
    /// Check `export ... from "module"` declarations.
    check_export_from: bool,
    /// Check CommonJS `require("module")` usage.
    check_require: bool,
    /// Merge user `styles` entries with the built-in defaults.
    extend_default_styles: bool,
    /// Per-module style configuration.
    ///
    /// Each key is a module specifier and each value is either:
    /// - `false` to disable checks for that module
    /// - an object containing style flags (`unassigned`, `default`, `namespace`, `named`)
    styles: IndexMap<String, ModuleStyles>,
}

impl Default for ImportStyleOptions {
    fn default() -> Self {
        Self {
            check_import: true,
            check_dynamic_import: true,
            check_export_from: false,
            check_require: true,
            extend_default_styles: true,
            styles: IndexMap::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
enum ModuleStyles {
    Disabled(FalseOnly),
    Styles(IndexMap<String, bool>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
struct FalseOnly;

impl<'de> Deserialize<'de> for FalseOnly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        if value {
            Err(serde::de::Error::custom("expected `false`, got `true`"))
        } else {
            Ok(FalseOnly)
        }
    }
}

#[derive(Debug, Clone)]
struct ImportStyleState {
    options: ImportStyleOptions,
    resolved_styles: FxHashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ImportStyle(Box<ImportStyleState>);

impl Default for ImportStyle {
    fn default() -> Self {
        Self::new(ImportStyleOptions::default())
    }
}

impl ImportStyle {
    fn new(options: ImportStyleOptions) -> Self {
        let resolved_styles = resolve_styles(&options);
        Self(Box::new(ImportStyleState { options, resolved_styles }))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce specific import styles per module.
    ///
    /// ### Why is this bad?
    ///
    /// Some modules are clearer when imported with a specific style.
    /// For example, modules with many unrelated APIs are often easier to read with named imports.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import util from "node:util";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { promisify } from "node:util";
    /// ```
    ImportStyle,
    unicorn,
    style,
    config = ImportStyleOptions,
);

impl Rule for ImportStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<ImportStyleOptions>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(Self::new)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) if self.0.options.check_import => {
                let module_name = import_decl.source.value.as_str();
                let Some(allowed_styles) = self.0.resolved_styles.get(module_name) else {
                    return;
                };
                let actual_styles = get_actual_import_declaration_styles(import_decl);
                Self::report(
                    ctx,
                    import_decl.span,
                    module_name,
                    &actual_styles,
                    allowed_styles,
                    false,
                );
            }
            AstKind::ImportExpression(import_expr) if self.0.options.check_dynamic_import => {
                if is_assigned_dynamic_import(node.id(), ctx) {
                    return;
                }
                let Some(module_name) = get_module_name_from_expression(&import_expr.source) else {
                    return;
                };
                let Some(allowed_styles) = self.0.resolved_styles.get(module_name.as_str()) else {
                    return;
                };
                Self::report(
                    ctx,
                    import_expr.span,
                    &module_name,
                    &[STYLE_UNASSIGNED],
                    allowed_styles,
                    false,
                );
            }
            AstKind::ExportAllDeclaration(export_all) if self.0.options.check_export_from => {
                let module_name = export_all.source.value.as_str();
                let Some(allowed_styles) = self.0.resolved_styles.get(module_name) else {
                    return;
                };
                Self::report(
                    ctx,
                    export_all.span,
                    module_name,
                    &[STYLE_NAMESPACE],
                    allowed_styles,
                    false,
                );
            }
            AstKind::ExportNamedDeclaration(export_named) if self.0.options.check_export_from => {
                let Some(source) = &export_named.source else {
                    return;
                };
                let module_name = source.value.as_str();
                let Some(allowed_styles) = self.0.resolved_styles.get(module_name) else {
                    return;
                };
                let actual_styles = get_actual_export_declaration_styles(export_named);
                Self::report(
                    ctx,
                    export_named.span,
                    module_name,
                    &actual_styles,
                    allowed_styles,
                    false,
                );
            }
            AstKind::CallExpression(call_expr) if self.0.options.check_require => {
                if call_expr.optional
                    || !call_expr.callee.is_specific_id("require")
                    || call_expr.arguments.len() != 1
                    || !matches!(
                        ctx.nodes().parent_kind(node.id()),
                        AstKind::ExpressionStatement(_)
                    )
                {
                    return;
                }

                let Some(module_name) = call_expr
                    .arguments
                    .first()
                    .and_then(Argument::as_expression)
                    .and_then(get_module_name_from_expression)
                else {
                    return;
                };
                let Some(allowed_styles) = self.0.resolved_styles.get(module_name.as_str()) else {
                    return;
                };
                Self::report(
                    ctx,
                    call_expr.span,
                    &module_name,
                    &[STYLE_UNASSIGNED],
                    allowed_styles,
                    true,
                );
            }
            AstKind::VariableDeclarator(var_decl) => {
                if self.0.options.check_dynamic_import {
                    self.check_dynamic_import_assignment(ctx, var_decl);
                }
                if self.0.options.check_require {
                    self.check_require_assignment(ctx, var_decl);
                }
            }
            _ => {}
        }
    }
}

impl ImportStyle {
    fn report(
        ctx: &LintContext<'_>,
        span: Span,
        module_name: &str,
        actual_styles: &[&str],
        allowed_styles: &[String],
        is_require: bool,
    ) {
        if allowed_styles.is_empty() {
            return;
        }

        let mut effective_allowed: FxHashSet<&str> =
            allowed_styles.iter().map(String::as_str).collect();
        if is_require
            && effective_allowed.contains(STYLE_DEFAULT)
            && !effective_allowed.contains(STYLE_NAMESPACE)
        {
            effective_allowed.insert(STYLE_NAMESPACE);
        }

        if actual_styles.iter().all(|style| effective_allowed.contains(style)) {
            return;
        }

        let allowed_styles = allowed_styles.iter().map(String::as_str).collect::<Vec<_>>();
        let allowed_styles = format_disjunction(&allowed_styles);
        ctx.diagnostic(import_style_diagnostic(span, &allowed_styles, module_name));
    }

    fn check_dynamic_import_assignment(
        &self,
        ctx: &LintContext<'_>,
        var_decl: &VariableDeclarator<'_>,
    ) {
        let Some(init) = &var_decl.init else { return };
        let Expression::AwaitExpression(await_expr) = init.get_inner_expression() else {
            return;
        };
        let Expression::ImportExpression(import_expr) = await_expr.argument.get_inner_expression()
        else {
            return;
        };
        let Some(module_name) = get_module_name_from_expression(&import_expr.source) else {
            return;
        };
        let Some(allowed_styles) = self.0.resolved_styles.get(module_name.as_str()) else {
            return;
        };
        let actual_styles = get_actual_assignment_target_import_styles(&var_decl.id);
        Self::report(ctx, var_decl.span, &module_name, &actual_styles, allowed_styles, false);
    }

    fn check_require_assignment(&self, ctx: &LintContext<'_>, var_decl: &VariableDeclarator<'_>) {
        let Some(init) = &var_decl.init else { return };
        let Expression::CallExpression(call_expr) = init.get_inner_expression() else {
            return;
        };
        if call_expr.optional
            || !call_expr.callee.is_specific_id("require")
            || call_expr.arguments.len() != 1
        {
            return;
        }

        let Some(module_name) = call_expr
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .and_then(get_module_name_from_expression)
        else {
            return;
        };
        let Some(allowed_styles) = self.0.resolved_styles.get(module_name.as_str()) else {
            return;
        };
        let actual_styles = get_actual_assignment_target_import_styles(&var_decl.id);
        Self::report(ctx, var_decl.span, &module_name, &actual_styles, allowed_styles, true);
    }
}

fn is_assigned_dynamic_import(node_id: oxc_semantic::NodeId, ctx: &LintContext<'_>) -> bool {
    let parent_node = ctx.nodes().parent_node(node_id);
    if !matches!(parent_node.kind(), AstKind::AwaitExpression(_)) {
        return false;
    }
    matches!(ctx.nodes().parent_kind(parent_node.id()), AstKind::VariableDeclarator(_))
}

fn get_actual_import_declaration_styles(import_decl: &ImportDeclaration<'_>) -> Vec<&'static str> {
    let Some(specifiers) = &import_decl.specifiers else {
        return vec![STYLE_UNASSIGNED];
    };
    if specifiers.is_empty() {
        return vec![STYLE_UNASSIGNED];
    }

    let mut styles = Vec::new();
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                push_unique(&mut styles, STYLE_DEFAULT);
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                push_unique(&mut styles, STYLE_NAMESPACE);
            }
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                if specifier.imported.identifier_name().is_some_and(|name| name == "default") {
                    push_unique(&mut styles, STYLE_DEFAULT);
                } else {
                    push_unique(&mut styles, STYLE_NAMED);
                }
            }
        }
    }
    styles
}

fn get_actual_export_declaration_styles(
    export_decl: &oxc_ast::ast::ExportNamedDeclaration<'_>,
) -> Vec<&'static str> {
    if export_decl.specifiers.is_empty() {
        return vec![STYLE_UNASSIGNED];
    }

    let mut styles = Vec::new();
    for specifier in &export_decl.specifiers {
        if specifier.exported.identifier_name().is_some_and(|name| name == "default") {
            push_unique(&mut styles, STYLE_DEFAULT);
        } else {
            push_unique(&mut styles, STYLE_NAMED);
        }
    }
    styles
}

fn get_actual_assignment_target_import_styles(pattern: &BindingPattern<'_>) -> Vec<&'static str> {
    match pattern {
        BindingPattern::BindingIdentifier(_) | BindingPattern::ArrayPattern(_) => {
            vec![STYLE_NAMESPACE]
        }
        BindingPattern::AssignmentPattern(assignment_pattern) => {
            get_actual_assignment_target_import_styles(&assignment_pattern.left)
        }
        BindingPattern::ObjectPattern(object_pattern) => {
            if object_pattern.properties.is_empty() && object_pattern.rest.is_none() {
                return vec![STYLE_UNASSIGNED];
            }

            let mut styles = Vec::new();
            for property in &object_pattern.properties {
                if let PropertyKey::StaticIdentifier(identifier) = &property.key {
                    if identifier.name == "default" {
                        push_unique(&mut styles, STYLE_DEFAULT);
                    } else {
                        push_unique(&mut styles, STYLE_NAMED);
                    }
                }
            }

            if object_pattern.rest.is_some() {
                push_unique(&mut styles, STYLE_NAMED);
            }
            styles
        }
    }
}

fn push_unique<T: PartialEq>(vec: &mut Vec<T>, value: T) {
    if !vec.contains(&value) {
        vec.push(value);
    }
}

fn get_module_name_from_expression(expr: &Expression<'_>) -> Option<String> {
    match expr.get_inner_expression() {
        Expression::StringLiteral(string_literal) => Some(string_literal.value.to_string()),
        Expression::TemplateLiteral(template_literal)
            if template_literal.is_no_substitution_template() =>
        {
            template_literal.single_quasi().map(|quasi| quasi.to_string())
        }
        Expression::BinaryExpression(binary_expr)
            if binary_expr.operator == BinaryOperator::Addition =>
        {
            Some(format!(
                "{}{}",
                get_module_name_from_expression(&binary_expr.left)?,
                get_module_name_from_expression(&binary_expr.right)?
            ))
        }
        _ => None,
    }
}

fn format_disjunction(styles: &[&str]) -> String {
    match styles.len() {
        0 => String::new(),
        1 => styles[0].to_string(),
        2 => format!("{} or {}", styles[0], styles[1]),
        n => {
            let prefix = styles[..n - 1].join(", ");
            format!("{prefix}, or {}", styles[n - 1])
        }
    }
}

fn resolve_styles(options: &ImportStyleOptions) -> FxHashMap<String, Vec<String>> {
    if !options.extend_default_styles {
        return options
            .styles
            .iter()
            .map(|(module_name, module_styles)| {
                (module_name.clone(), module_styles_to_allowed(module_styles, None))
            })
            .collect();
    }

    let mut module_names =
        default_styles().iter().map(|(name, _)| (*name).to_string()).collect::<Vec<_>>();
    let extra_module_names = options
        .styles
        .keys()
        .filter(|name| !module_names.contains(*name))
        .cloned()
        .collect::<Vec<_>>();
    module_names.extend(extra_module_names);

    module_names
        .into_iter()
        .map(|module_name| {
            let module_style = options.styles.get(&module_name);
            let default = default_styles()
                .iter()
                .find(|(name, _)| *name == module_name)
                .map(|(_, styles)| *styles);
            (module_name, module_styles_to_allowed_opt(module_style, default))
        })
        .collect()
}

fn module_styles_to_allowed_opt(
    module_style: Option<&ModuleStyles>,
    default_styles: Option<&[(&str, bool)]>,
) -> Vec<String> {
    if matches!(module_style, Some(ModuleStyles::Disabled(_))) {
        return Vec::new();
    }

    let mut merged = IndexMap::<String, bool>::new();
    if let Some(default_styles) = default_styles {
        for (style, enabled) in default_styles {
            merged.insert((*style).to_string(), *enabled);
        }
    }

    if let Some(ModuleStyles::Styles(module_styles)) = module_style {
        for (style, enabled) in module_styles {
            merged.insert(style.clone(), *enabled);
        }
    }

    merged.into_iter().filter_map(|(style, enabled)| enabled.then_some(style)).collect()
}

fn module_styles_to_allowed(
    module_style: &ModuleStyles,
    default_styles: Option<&[(&str, bool)]>,
) -> Vec<String> {
    match module_style {
        ModuleStyles::Disabled(_) => Vec::new(),
        ModuleStyles::Styles(_) => module_styles_to_allowed_opt(Some(module_style), default_styles),
    }
}

fn default_styles() -> &'static [(&'static str, &'static [(&'static str, bool)])] {
    &[
        ("chalk", &[(STYLE_DEFAULT, true)]),
        ("path", &[(STYLE_DEFAULT, true)]),
        ("node:path", &[(STYLE_DEFAULT, true)]),
        ("util", &[(STYLE_NAMED, true)]),
        ("node:util", &[(STYLE_NAMED, true)]),
    ]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let styles_option = json!([{
        "checkExportFrom": true,
        "styles": {
            "unassigned": { "unassigned": true, "named": false },
            "default": { "default": true, "named": false },
            "namespace": { "namespace": true, "named": false },
            "named": { "named": true }
        }
    }]);

    let pass = vec![
        ("let a", None),
        ("import {inspect} from 'util'", None),
        ("import {inspect} from 'node:util'", None),
        ("const {inspect} = require('util')", None),
        ("const {inspect} = require('node:util')", None),
        ("import chalk from 'chalk'", None),
        ("import {default as chalk} from 'chalk'", None),
        ("const path = require('path')", None),
        ("export {promisify, callbackify} from 'util'", None),
        ("export {promisify, callbackify} from 'node:util'", None),
        ("require('chalk')", Some(json!([{ "styles": {}, "extendDefaultStyles": false }]))),
        ("import {red} from 'chalk'", Some(json!([{ "checkImport": false }]))),
        (
            "async () => { const {red} = await import('chalk'); }",
            Some(json!([{ "checkDynamicImport": false }])),
        ),
        ("import('chalk')", Some(json!([{ "checkDynamicImport": false }]))),
        ("require('util')", Some(json!([{ "checkRequire": false }]))),
        (
            "import util, {inspect} from 'named-or-default'",
            Some(json!([{ "styles": { "named-or-default": { "named": true, "default": true } } }])),
        ),
        (
            "import util from 'node:util'; import * as util2 from 'node:util'; import {foo} from 'node:util';",
            Some(json!([{ "styles": { "node:util": false } }])),
        ),
        ("require('unassigned')", Some(styles_option.clone())),
        ("import x from 'default'", Some(styles_option.clone())),
        ("import * as x from 'namespace'", Some(styles_option.clone())),
        ("import {x} from 'named'", Some(styles_option.clone())),
        ("export * from 'namespace'", Some(styles_option.clone())),
        ("export {default} from 'default'", Some(styles_option.clone())),
        ("const x = require('default')", Some(styles_option.clone())),
        ("const {default: x} = require('default')", Some(styles_option.clone())),
        ("const [] = require('default')", Some(styles_option.clone())),
        ("async () => { const {x} = await import('named'); }", Some(styles_option.clone())),
        (
            "import {foo} from 'node:util'",
            Some(json!([{ "styles": { "node:util": { "default": true } } }])),
        ),
        (
            "import foo from 'node:util'",
            Some(json!([{ "styles": { "node:util": { "default": true } } }])),
        ),
    ];

    let fail = vec![
        ("import util from 'util'", None),
        ("import util from 'node:util'", None),
        ("import * as util from 'util'", None),
        ("import * as util from 'node:util'", None),
        ("const util = require('util')", None),
        ("const util = require('node:util')", None),
        ("require('util')", None),
        ("require('node:util')", None),
        ("require('ut' + 'il')", None),
        ("require('node:' + 'util')", None),
        ("import {red} from 'chalk'", None),
        ("import {red as green} from 'chalk'", None),
        ("async () => { const {red} = await import('chalk'); }", None),
        ("const {x} = require('unassigned')", Some(styles_option.clone())),
        ("import x from 'unassigned'", Some(styles_option.clone())),
        ("require('default')", Some(styles_option.clone())),
        ("import {x} from 'default'", Some(styles_option.clone())),
        ("export {x} from 'default'", Some(styles_option.clone())),
        ("import * as x from 'named'", Some(styles_option.clone())),
        ("async () => { const x = await import('named'); }", Some(styles_option)),
        (
            "import util, {inspect} from 'named'",
            Some(json!([{ "styles": { "named": { "named": true } } }])),
        ),
        (
            "import * as util from 'node:util'",
            Some(json!([{ "styles": { "node:util": { "default": true } } }])),
        ),
        (
            "import {foo} from 'node:util'",
            Some(json!([{ "styles": { "node:util": { "default": true, "named": false } } }])),
        ),
    ];

    Tester::new(ImportStyle::NAME, ImportStyle::PLUGIN, pass, fail).test_and_snapshot();
}
