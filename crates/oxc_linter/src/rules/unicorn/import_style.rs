use oxc_ast::{
    AstKind,
    ast::{
        BinaryOperator, BindingPattern, ExportAllDeclaration, ExportNamedDeclaration, Expression,
        ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{ToJsString, WithoutGlobalReferenceInformation};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::{
    JsonSchema, SchemaGenerator,
    schema::{Schema, SchemaObject},
};
use serde::{Deserialize, Serialize, de};
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::default_true,
};

fn import_style_diagnostic(
    span: Span,
    module_name: &str,
    allowed_styles: StyleSet,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Use {} import for module `{module_name}`.",
        allowed_styles.format_for_diagnostic()
    ))
    .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ImportStyle {
    /// Per-module import style preferences.
    ///
    /// Each key is a module specifier. Set the value to `false` to disable checking for the
    /// module, or to an object that allows one or more import styles. The available styles are
    /// `unassigned`, `default`, `namespace`, and `named`. When `extendDefaultStyles` is `true`,
    /// these entries extend the built-in defaults instead of replacing them.
    ///
    /// The default module preferences are default imports for `chalk`, `path`, and `node:path`,
    /// and named imports for `util` and `node:util`.
    ///
    /// With `{ "styles": { "node:util": { "named": true, "default": false } } }`,
    /// examples of **incorrect** code:
    /// ```js
    /// import util from "node:util";
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```js
    /// import {promisify} from "node:util";
    /// ```
    styles: FxHashMap<String, ModuleStylesOverride>,
    /// Whether `styles` extends or replaces the built-in module preferences.
    ///
    /// When this is `true`, entries in `styles` are merged with the default preferences. For
    /// example, `{ "styles": { "path": { "named": true } } }` allows named imports from
    /// `path` while leaving its default import style allowed. When this is `false`, only modules
    /// configured in `styles` are checked.
    ///
    /// With `{ "extendDefaultStyles": false, "styles": {} }`, examples of **correct** code:
    /// ```js
    /// import {red} from "chalk";
    /// ```
    #[serde(default = "default_true")]
    extend_default_styles: bool,
    /// Whether static import declarations are checked.
    ///
    /// Set this to `false` to skip `import ... from "module"` and side-effect imports like
    /// `import "module"`.
    ///
    /// With the default configuration, examples of **incorrect** code:
    /// ```js
    /// import {red} from "chalk";
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```js
    /// import chalk from "chalk";
    /// ```
    #[serde(default = "default_true")]
    check_import: bool,
    /// Whether dynamic import expressions are checked.
    ///
    /// Set this to `false` to skip calls such as `await import("module")`.
    ///
    /// With the default configuration, examples of **incorrect** code:
    /// ```js
    /// async () => {
    ///   const {red} = await import("chalk");
    /// };
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```js
    /// async () => {
    ///   const {default: chalk} = await import("chalk");
    /// };
    /// ```
    #[serde(default = "default_true")]
    check_dynamic_import: bool,
    /// Whether export-from declarations are checked.
    ///
    /// This is disabled by default. Set this to `true` to check declarations like
    /// `export ... from "module"`.
    ///
    /// With `{ "checkExportFrom": true }`, examples of **incorrect** code:
    /// ```js
    /// export * from "node:util";
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```js
    /// export {promisify} from "node:util";
    /// ```
    check_export_from: bool,
    /// Whether CommonJS `require()` calls are checked.
    ///
    /// Set this to `false` to skip `require("module")` calls completely.
    ///
    /// With the default configuration, examples of **incorrect** code:
    /// ```js
    /// const util = require("node:util");
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```js
    /// const {promisify} = require("node:util");
    /// ```
    #[serde(default = "default_true")]
    check_require: bool,
}

impl Default for ImportStyle {
    fn default() -> Self {
        Self {
            styles: FxHashMap::default(),
            extend_default_styles: true,
            check_import: true,
            check_dynamic_import: true,
            check_export_from: false,
            check_require: true,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce specific import styles per module.
    ///
    /// ### Why is this bad?
    ///
    /// Some modules are easier to read when imported in a consistent way.
    /// For example, utility modules often work better with named imports,
    /// while modules that expose one primary interface are clearer as default imports.
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
    /// import {promisify} from "node:util";
    /// ```
    ImportStyle,
    unicorn,
    restriction,
    none,
    config = ImportStyle,
    version = "next",
);

impl Rule for ImportStyle {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) if self.check_import => {
                self.check_import_declaration(import_decl, ctx);
            }
            AstKind::ImportExpression(import_expr) if self.check_dynamic_import => {
                if is_assigned_dynamic_import(node, ctx) {
                    return;
                }
                let Some(source) = get_module_name(&import_expr.source) else { return };
                self.report_if_needed(
                    import_expr.span,
                    source.as_ref(),
                    StyleSet::unassigned(),
                    false,
                    ctx,
                );
            }
            AstKind::ExportAllDeclaration(export_decl) if self.check_export_from => {
                self.check_export_all_declaration(export_decl, ctx);
            }
            AstKind::ExportNamedDeclaration(export_decl) if self.check_export_from => {
                self.check_export_named_declaration(export_decl, ctx);
            }
            AstKind::ExpressionStatement(statement) if self.check_require => {
                let Expression::CallExpression(call_expr) = &statement.expression else { return };
                let Some(source) = get_require_module_name(call_expr) else { return };
                self.report_if_needed(
                    call_expr.span,
                    source.as_ref(),
                    StyleSet::unassigned(),
                    true,
                    ctx,
                );
            }
            AstKind::VariableDeclarator(declarator) => {
                self.check_variable_declarator(declarator, ctx);
            }
            _ => {}
        }
    }
}

impl ImportStyle {
    fn check_import_declaration(&self, import_decl: &ImportDeclaration<'_>, ctx: &LintContext<'_>) {
        let actual_styles = get_actual_import_declaration_styles(import_decl);
        self.report_if_needed(
            import_decl.span,
            import_decl.source.value.as_str(),
            actual_styles,
            false,
            ctx,
        );
    }

    fn check_export_all_declaration(
        &self,
        export_decl: &ExportAllDeclaration<'_>,
        ctx: &LintContext<'_>,
    ) {
        self.report_if_needed(
            export_decl.span,
            export_decl.source.value.as_str(),
            StyleSet::namespace(),
            false,
            ctx,
        );
    }

    fn check_export_named_declaration(
        &self,
        export_decl: &ExportNamedDeclaration<'_>,
        ctx: &LintContext<'_>,
    ) {
        let Some(source) = &export_decl.source else { return };
        let actual_styles = get_actual_export_declaration_styles(export_decl);
        self.report_if_needed(export_decl.span, source.value.as_str(), actual_styles, false, ctx);
    }

    fn check_variable_declarator(
        &self,
        declarator: &VariableDeclarator<'_>,
        ctx: &LintContext<'_>,
    ) {
        if self.check_dynamic_import
            && let Some(Expression::AwaitExpression(await_expr)) = &declarator.init
            && let Expression::ImportExpression(import_expr) = &await_expr.argument
            && let Some(source) = get_module_name(&import_expr.source)
        {
            self.report_if_needed(
                declarator.span,
                source.as_ref(),
                get_actual_assignment_target_styles(&declarator.id),
                false,
                ctx,
            );
            return;
        }

        if self.check_require
            && let Some(Expression::CallExpression(call_expr)) = &declarator.init
            && let Some(source) = get_require_module_name(call_expr)
        {
            self.report_if_needed(
                declarator.span,
                source.as_ref(),
                get_actual_assignment_target_styles(&declarator.id),
                true,
                ctx,
            );
        }
    }

    fn report_if_needed(
        &self,
        span: Span,
        module_name: &str,
        actual_styles: StyleSet,
        is_require: bool,
        ctx: &LintContext<'_>,
    ) {
        let Some(allowed_styles) = self.allowed_styles(module_name) else { return };

        let allowed_styles =
            if is_require && allowed_styles.default_style && !allowed_styles.namespace {
                allowed_styles.with_namespace()
            } else {
                allowed_styles
            };

        if actual_styles.is_subset_of(allowed_styles) {
            return;
        }

        ctx.diagnostic(import_style_diagnostic(span, module_name, allowed_styles));
    }

    fn allowed_styles(&self, module_name: &str) -> Option<StyleSet> {
        let override_styles = self.styles.get(module_name);
        if matches!(override_styles, Some(ModuleStylesOverride::Disabled(_))) {
            return None;
        }

        let base = self.extend_default_styles.then(|| default_style_for(module_name)).flatten();
        let allowed_styles =
            if let Some(ModuleStylesOverride::Styles(override_styles)) = override_styles {
                override_styles.apply_to(base.unwrap_or_default())
            } else {
                base?
            };

        (!allowed_styles.is_empty()).then_some(allowed_styles)
    }
}

fn get_actual_import_declaration_styles(import_decl: &ImportDeclaration<'_>) -> StyleSet {
    let Some(specifiers) = &import_decl.specifiers else {
        return StyleSet::unassigned();
    };
    if specifiers.is_empty() {
        return StyleSet::unassigned();
    }

    let mut styles = StyleSet::default();
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => styles.default_style = true,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => styles.namespace = true,
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                if is_default_module_export_name(&specifier.imported) {
                    styles.default_style = true;
                } else {
                    styles.named = true;
                }
            }
        }
    }
    styles
}

fn get_actual_export_declaration_styles(export_decl: &ExportNamedDeclaration<'_>) -> StyleSet {
    if export_decl.specifiers.is_empty() {
        return StyleSet::unassigned();
    }

    let mut styles = StyleSet::default();
    for specifier in &export_decl.specifiers {
        if is_default_module_export_name(&specifier.exported) {
            styles.default_style = true;
        } else {
            styles.named = true;
        }
    }
    styles
}

fn get_actual_assignment_target_styles(pattern: &BindingPattern<'_>) -> StyleSet {
    match pattern {
        BindingPattern::BindingIdentifier(_) | BindingPattern::ArrayPattern(_) => {
            StyleSet::namespace()
        }
        BindingPattern::AssignmentPattern(pattern) => {
            get_actual_assignment_target_styles(&pattern.left)
        }
        BindingPattern::ObjectPattern(pattern) => {
            let mut styles = StyleSet::default();
            if pattern.properties.is_empty() && pattern.rest.is_none() {
                styles.unassigned = true;
                return styles;
            }

            for property in &pattern.properties {
                if property.key.is_specific_static_name("default") {
                    styles.default_style = true;
                } else {
                    styles.named = true;
                }
            }
            if pattern.rest.is_some() {
                styles.named = true;
            }
            styles
        }
    }
}

fn is_default_module_export_name(name: &ModuleExportName<'_>) -> bool {
    name.identifier_name().is_some_and(|name| name == "default")
}

fn get_module_name<'a>(expr: &'a Expression<'a>) -> Option<std::borrow::Cow<'a, str>> {
    let expr = expr.get_inner_expression();
    if let Some(value) = expr.to_js_string(&WithoutGlobalReferenceInformation) {
        return Some(value);
    }

    if let Expression::BinaryExpression(binary_expr) = expr
        && binary_expr.operator == BinaryOperator::Addition
    {
        let left = get_module_name(&binary_expr.left)?;
        let right = get_module_name(&binary_expr.right)?;
        return Some(std::borrow::Cow::Owned(format!("{left}{right}")));
    }

    None
}

fn get_require_module_name<'a>(
    call_expr: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<std::borrow::Cow<'a, str>> {
    if call_expr.arguments.len() != 1 || !call_expr.callee.is_specific_id("require") {
        return None;
    }
    call_expr
        .arguments
        .first()
        .and_then(oxc_ast::ast::Argument::as_expression)
        .and_then(get_module_name)
}

fn is_assigned_dynamic_import(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::AwaitExpression(await_expr) = parent.kind() else {
        return false;
    };
    if await_expr.argument.span() != node.kind().span() {
        return false;
    }

    let grandparent = ctx.nodes().parent_node(parent.id());
    let AstKind::VariableDeclarator(declarator) = grandparent.kind() else {
        return false;
    };
    declarator.init.as_ref().is_some_and(|init| init.span() == await_expr.span)
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ModuleStylesOverride {
    Disabled(bool),
    Styles(RawStyleSet),
}

impl<'de> Deserialize<'de> for ModuleStylesOverride {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawModuleStylesOverride {
            Disabled(bool),
            Styles(RawStyleSet),
        }

        match RawModuleStylesOverride::deserialize(deserializer)? {
            RawModuleStylesOverride::Disabled(false) => Ok(Self::Disabled(false)),
            RawModuleStylesOverride::Disabled(true) => {
                Err(de::Error::custom("module style override boolean must be `false`"))
            }
            RawModuleStylesOverride::Styles(styles) => Ok(Self::Styles(styles)),
        }
    }
}

impl JsonSchema for ModuleStylesOverride {
    fn schema_name() -> String {
        "ModuleStylesOverride".to_string()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "ModuleStylesOverride".into()
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        let mut false_schema = <bool as JsonSchema>::json_schema(r#gen).into_object();
        false_schema.enum_values = Some(vec![false.into()]);

        let mut schema = SchemaObject::default();
        schema.subschemas().one_of =
            Some(vec![false_schema.into(), <RawStyleSet as JsonSchema>::json_schema(r#gen)]);
        schema.into()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RawStyleSet {
    /// Whether named imports or destructured `require()` calls are allowed for this module.
    ///
    /// With `{ "styles": { "node:util": { "named": true } } }`, this is valid:
    /// ```js
    /// import {promisify} from "node:util";
    /// ```
    named: Option<bool>,
    /// Whether namespace imports or whole-module `require()` assignments are allowed for this module.
    ///
    /// With `{ "styles": { "node:fs": { "namespace": true } } }`, this is valid:
    /// ```js
    /// import * as fs from "node:fs";
    /// ```
    namespace: Option<bool>,
    /// Whether default imports or whole-module `require()` assignments are allowed for this module.
    ///
    /// With `{ "styles": { "chalk": { "default": true } } }`, this is valid:
    /// ```js
    /// import chalk from "chalk";
    /// ```
    #[serde(rename = "default")]
    default_style: Option<bool>,
    /// Whether side-effect imports or unassigned dynamic imports/requires are allowed for this module.
    ///
    /// With `{ "styles": { "polyfill": { "unassigned": true } } }`, this is valid:
    /// ```js
    /// import "polyfill";
    /// ```
    unassigned: Option<bool>,
}

impl RawStyleSet {
    fn apply_to(self, base: StyleSet) -> StyleSet {
        StyleSet {
            named: self.named.unwrap_or(base.named),
            namespace: self.namespace.unwrap_or(base.namespace),
            default_style: self.default_style.unwrap_or(base.default_style),
            unassigned: self.unassigned.unwrap_or(base.unassigned),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct StyleSet {
    named: bool,
    namespace: bool,
    default_style: bool,
    unassigned: bool,
}

impl StyleSet {
    const fn named() -> Self {
        Self { named: true, namespace: false, default_style: false, unassigned: false }
    }

    const fn namespace() -> Self {
        Self { named: false, namespace: true, default_style: false, unassigned: false }
    }

    const fn default_style() -> Self {
        Self { named: false, namespace: false, default_style: true, unassigned: false }
    }

    const fn unassigned() -> Self {
        Self { named: false, namespace: false, default_style: false, unassigned: true }
    }

    fn is_empty(self) -> bool {
        !self.named && !self.namespace && !self.default_style && !self.unassigned
    }

    fn is_subset_of(self, other: Self) -> bool {
        (!self.named || other.named)
            && (!self.namespace || other.namespace)
            && (!self.default_style || other.default_style)
            && (!self.unassigned || other.unassigned)
    }

    fn with_namespace(self) -> Self {
        Self { namespace: true, ..self }
    }

    fn format_for_diagnostic(self) -> String {
        let mut parts = Vec::with_capacity(4);
        if self.named {
            parts.push("named");
        }
        if self.namespace {
            parts.push("namespace");
        }
        if self.default_style {
            parts.push("default");
        }
        if self.unassigned {
            parts.push("unassigned");
        }

        match parts.as_slice() {
            [] => String::new(),
            [one] => (*one).to_string(),
            [left, right] => format!("{left} or {right}"),
            _ => {
                let last = parts.pop().unwrap_or_default();
                format!("{}, or {}", parts.join(", "), last)
            }
        }
    }
}

fn default_style_for(module_name: &str) -> Option<StyleSet> {
    match module_name {
        "chalk" | "node:path" | "path" => Some(StyleSet::default_style()),
        "node:util" | "util" => Some(StyleSet::named()),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let options = json!([{
        "checkExportFrom": true,
        "styles": {
            "unassigned": { "unassigned": true, "named": false },
            "default": { "default": true, "named": false },
            "namespace": { "namespace": true, "named": false },
            "named": { "named": true }
        }
    }]);

    let pass = vec![
        ("require('unassigned')", Some(options.clone())),
        ("const {} = require('unassigned')", Some(options.clone())),
        ("import 'unassigned'", Some(options.clone())),
        ("import {} from 'unassigned'", Some(options.clone())),
        ("import('unassigned')", Some(options.clone())),
        ("export {} from 'unassigned'", Some(options.clone())),
        ("const x = require('default')", Some(options.clone())),
        ("const {default: x} = require('default')", Some(options.clone())),
        ("const [] = require('default')", Some(options.clone())),
        ("import x from 'default'", Some(options.clone())),
        (
            "async () => {
                const {default: x} = await import('default');
            }",
            Some(options.clone()),
        ),
        ("export {default} from 'default'", Some(options.clone())),
        ("const x = require('namespace')", Some(options.clone())),
        ("const [] = require('namespace')", Some(options.clone())),
        ("import * as x from 'namespace'", Some(options.clone())),
        (
            "async () => {
                const x = await import('namespace');
            }",
            Some(options.clone()),
        ),
        ("export * from 'namespace'", Some(options.clone())),
        ("const {x} = require('named')", Some(options.clone())),
        ("const {...rest} = require('named')", Some(options.clone())),
        ("const {x: y} = require('named')", Some(options.clone())),
        ("import {x} from 'named'", Some(options.clone())),
        ("import {x as y} from 'named'", Some(options.clone())),
        (
            "async () => {
                const {x} = await import('named');
            }",
            Some(options.clone()),
        ),
        (
            "async () => {
                const {x: y} = await import('named');
            }",
            Some(options.clone()),
        ),
        ("export {x} from 'named'", Some(options.clone())),
        ("export {x as y} from 'named'", Some(options.clone())),
        ("const foo = 1; export {foo}", Some(options.clone())),
        ("export const foo = 1;", Some(options.clone())),
        ("export function foo() {}", Some(options.clone())),
        ("import {inspect} from 'util'", None),
        ("import {inspect} from 'node:util'", None),
        ("const {inspect} = require('util')", None),
        ("const {inspect} = require('node:util')", None),
        ("import chalk from 'chalk'", None),
        ("import {default as chalk} from 'chalk'", None),
        ("export {promisify, callbackify} from 'util'", Some(json!([{ "checkExportFrom": true }]))),
        (
            "export {promisify, callbackify} from 'node:util'",
            Some(json!([{ "checkExportFrom": true }])),
        ),
        ("require('chalk')", Some(json!([{ "styles": {}, "extendDefaultStyles": false }]))),
        ("import 'chalk'", Some(json!([{ "checkImport": false }]))),
        (
            "async () => {
                const {red} = await import('chalk');
            }",
            Some(json!([{ "checkDynamicImport": false }])),
        ),
        ("import('chalk')", Some(json!([{ "checkDynamicImport": false }]))),
        ("require('chalk')", Some(json!([{ "checkRequire": false }]))),
        ("const {red} = require('chalk')", Some(json!([{ "checkRequire": false }]))),
        (
            "import util, {inspect} from 'named-or-default'",
            Some(json!([{ "styles": { "named-or-default": { "named": true, "default": true } } }])),
        ),
        ("require(variable)", None),
        ("const x = require(variable)", None),
        ("const x = require('unassigned').x", Some(options.clone())),
        (
            "async () => {
                const {red} = await import(variable);
            }",
            None,
        ),
        (
            "import util from 'node:util'; import * as util2 from 'node:util'; import {foo} from 'node:util';",
            Some(json!([{ "styles": { "node:util": false } }])),
        ),
        ("import type chalk from 'chalk'", None),
        ("import type {x} from 'named'", Some(options.clone())),
        ("let a", None),
    ];

    let fail = vec![
        ("const {x} = require('unassigned')", Some(options.clone())),
        ("const {default: x} = require('unassigned')", Some(options.clone())),
        ("import x from 'unassigned'", Some(options.clone())),
        (
            "async () => {
                const {default: x} = await import('unassigned');
            }",
            Some(options.clone()),
        ),
        ("const x = require('unassigned')", Some(options.clone())),
        ("import * as x from 'unassigned'", Some(options.clone())),
        (
            "async () => {
                const x = await import('unassigned');
            }",
            Some(options.clone()),
        ),
        ("const {x: y} = require('unassigned')", Some(options.clone())),
        ("import {x} from 'unassigned'", Some(options.clone())),
        ("import {x as y} from 'unassigned'", Some(options.clone())),
        (
            "async () => {
                const {x} = await import('unassigned');
            }",
            Some(options.clone()),
        ),
        (
            "async () => {
                const {x: y} = await import('unassigned');
            }",
            Some(options.clone()),
        ),
        ("const {...rest} = require('unassigned')", Some(options.clone())),
        ("const [] = require('unassigned')", Some(options.clone())),
        ("export * from 'unassigned'", Some(options.clone())),
        ("export {x} from 'unassigned'", Some(options.clone())),
        ("export {x as y} from 'unassigned'", Some(options.clone())),
        ("export {default} from 'unassigned'", Some(options.clone())),
        ("require('default')", Some(options.clone())),
        ("const {} = require('default')", Some(options.clone())),
        ("const {...rest} = require('default')", Some(options.clone())),
        ("import 'default'", Some(options.clone())),
        ("import {} from 'default'", Some(options.clone())),
        ("import('default')", Some(options.clone())),
        ("import * as x from 'default'", Some(options.clone())),
        (
            "async () => {
                const {x} = await import('default');
            }",
            Some(options.clone()),
        ),
        ("const {x} = require('default')", Some(options.clone())),
        ("const {x: y} = require('default')", Some(options.clone())),
        ("import {x} from 'default'", Some(options.clone())),
        ("import {x as y} from 'default'", Some(options.clone())),
        ("export * from 'default'", Some(options.clone())),
        ("export {x} from 'default'", Some(options.clone())),
        ("export {x as y} from 'default'", Some(options.clone())),
        ("require('namespace')", Some(options.clone())),
        ("const {} = require('namespace')", Some(options.clone())),
        ("import 'namespace'", Some(options.clone())),
        ("import {} from 'namespace'", Some(options.clone())),
        ("import('namespace')", Some(options.clone())),
        ("const {default: x} = require('namespace')", Some(options.clone())),
        ("const {...rest} = require('namespace')", Some(options.clone())),
        ("import x from 'namespace'", Some(options.clone())),
        ("const {x} = require('namespace')", Some(options.clone())),
        ("const {x: y} = require('namespace')", Some(options.clone())),
        ("import {x} from 'namespace'", Some(options.clone())),
        ("import {x as y} from 'namespace'", Some(options.clone())),
        ("export {x} from 'namespace'", Some(options.clone())),
        ("export {x as y} from 'namespace'", Some(options.clone())),
        ("export {default} from 'namespace'", Some(options.clone())),
        ("require('named')", Some(options.clone())),
        ("const {} = require('named')", Some(options.clone())),
        ("const [] = require('named')", Some(options.clone())),
        ("import 'named'", Some(options.clone())),
        ("import {} from 'named'", Some(options.clone())),
        ("import('named')", Some(options.clone())),
        ("const x = require('named')", Some(options.clone())),
        ("const {default: x} = require('named')", Some(options.clone())),
        ("import x from 'named'", Some(options.clone())),
        (
            "async () => {
                const {default: x} = await import('named');
            }",
            Some(options.clone()),
        ),
        ("import * as x from 'named'", Some(options.clone())),
        (
            "async () => {
                const x = await import('named');
            }",
            Some(options.clone()),
        ),
        ("export * from 'named'", Some(options.clone())),
        ("export {default} from 'named'", Some(options.clone())),
        ("import util, {inspect} from 'named'", Some(options.clone())),
        ("import util, {inspect} from 'default'", Some(options)),
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
        (
            "async () => {
                const {red} = await import('chalk');
            }",
            None,
        ),
        (
            "async () => {
                const {red} = await import('ch' + 'alk');
            }",
            None,
        ),
        (
            "require('no-unassigned')",
            Some(
                json!([{ "styles": { "no-unassigned": { "named": true, "namespace": true, "default": true } } }]),
            ),
        ),
        (
            "import * as util from 'node:util';",
            Some(json!([{ "styles": { "node:util": { "default": true } } }])),
        ),
        (
            "import * as util from 'node:util';",
            Some(json!([{ "styles": { "node:util": { "default": true, "named": false } } }])),
        ),
        ("import {type ChalkInstance} from 'chalk'", None),
        ("import type {ChalkInstance} from 'chalk'", None),
    ];

    Tester::new(ImportStyle::NAME, ImportStyle::PLUGIN, pass, fail).test_and_snapshot();
}

#[cfg(test)]
mod internal_tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn rejects_true_module_style_override() {
        let result = ImportStyle::from_configuration(json!([{
            "styles": {
                "util": true
            }
        }]));

        assert!(result.is_err());
    }

    #[test]
    fn accepts_false_module_style_override() {
        let result = ImportStyle::from_configuration(json!([{
            "styles": {
                "util": false
            }
        }]));

        assert!(result.is_ok());
    }
}
