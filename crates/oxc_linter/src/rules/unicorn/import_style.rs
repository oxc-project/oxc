use std::collections::BTreeMap;

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, ExportAllDeclaration, ExportNamedDeclaration, Expression,
        ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
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
    allowed_styles: &StyleSet,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Use {} import for module `{module_name}`.",
        allowed_styles.format_for_diagnostic()
    ))
    .with_label(span)
}

#[derive(Debug, Clone)]
pub struct ImportStyle(Box<ImportStyleConfig>);

impl Default for ImportStyle {
    fn default() -> Self {
        Self(Box::new(ImportStyleConfig::default()))
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
    config = RawImportStyleConfig,
    version = "next",
);

impl Rule for ImportStyle {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let raw =
            serde_json::from_value::<DefaultRuleConfig<RawImportStyleConfig>>(value)?.into_inner();
        Ok(Self(Box::new(ImportStyleConfig::from_raw(raw))))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) if self.0.check_import => {
                self.check_import_declaration(import_decl, ctx);
            }
            AstKind::ImportExpression(import_expr) if self.0.check_dynamic_import => {
                if is_assigned_dynamic_import(node, ctx) {
                    return;
                }
                let Expression::StringLiteral(source) = &import_expr.source else { return };
                self.report_if_needed(
                    import_expr.span,
                    source.value.as_str(),
                    StyleSet::unassigned(),
                    false,
                    ctx,
                );
            }
            AstKind::ExportAllDeclaration(export_decl) if self.0.check_export_from => {
                self.check_export_all_declaration(export_decl, ctx);
            }
            AstKind::ExportNamedDeclaration(export_decl) if self.0.check_export_from => {
                self.check_export_named_declaration(export_decl, ctx);
            }
            AstKind::ExpressionStatement(statement) if self.0.check_require => {
                let Expression::CallExpression(call_expr) = &statement.expression else { return };
                let Some(source) = call_expr.common_js_require() else { return };
                self.report_if_needed(
                    call_expr.span,
                    source.value.as_str(),
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
        if self.0.check_dynamic_import {
            if let Some(Expression::AwaitExpression(await_expr)) = &declarator.init {
                if let Expression::ImportExpression(import_expr) = &await_expr.argument {
                    if let Expression::StringLiteral(source) = &import_expr.source {
                        self.report_if_needed(
                            declarator.span,
                            source.value.as_str(),
                            get_actual_assignment_target_styles(&declarator.id),
                            false,
                            ctx,
                        );
                        return;
                    }
                }
            }
        }

        if self.0.check_require {
            if let Some(Expression::CallExpression(call_expr)) = &declarator.init {
                if let Some(source) = call_expr.common_js_require() {
                    self.report_if_needed(
                        declarator.span,
                        source.value.as_str(),
                        get_actual_assignment_target_styles(&declarator.id),
                        true,
                        ctx,
                    );
                }
            }
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
        let Some(allowed_styles) = self.0.styles.get(module_name) else { return };
        if allowed_styles.is_empty() {
            return;
        }

        let allowed_styles =
            if is_require && allowed_styles.default_style && !allowed_styles.namespace {
                allowed_styles.with_namespace()
            } else {
                *allowed_styles
            };

        if actual_styles.is_subset_of(&allowed_styles) {
            return;
        }

        ctx.diagnostic(import_style_diagnostic(span, module_name, &allowed_styles));
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

#[derive(Debug, Clone)]
struct ImportStyleConfig {
    styles: BTreeMap<String, StyleSet>,
    check_import: bool,
    check_dynamic_import: bool,
    check_export_from: bool,
    check_require: bool,
}

impl Default for ImportStyleConfig {
    fn default() -> Self {
        Self::from_raw(RawImportStyleConfig::default())
    }
}

impl ImportStyleConfig {
    fn from_raw(raw: RawImportStyleConfig) -> Self {
        let mut styles = if raw.extend_default_styles { default_styles() } else { BTreeMap::new() };

        for (module_name, override_styles) in raw.styles {
            match override_styles {
                ModuleStylesOverride::Disabled(_) => {
                    styles.insert(module_name, StyleSet::default());
                }
                ModuleStylesOverride::Styles(override_styles) => {
                    let next = if raw.extend_default_styles {
                        override_styles
                            .apply_to(styles.get(&module_name).copied().unwrap_or_default())
                    } else {
                        override_styles.apply_to(StyleSet::default())
                    };
                    styles.insert(module_name, next);
                }
            }
        }

        Self {
            styles,
            check_import: raw.check_import,
            check_dynamic_import: raw.check_dynamic_import,
            check_export_from: raw.check_export_from,
            check_require: raw.check_require,
        }
    }
}

impl Default for RawImportStyleConfig {
    fn default() -> Self {
        Self {
            styles: BTreeMap::new(),
            extend_default_styles: true,
            check_import: true,
            check_dynamic_import: true,
            check_export_from: false,
            check_require: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RawImportStyleConfig {
    styles: BTreeMap<String, ModuleStylesOverride>,
    #[serde(default = "default_true")]
    extend_default_styles: bool,
    #[serde(default = "default_true")]
    check_import: bool,
    #[serde(default = "default_true")]
    check_dynamic_import: bool,
    check_export_from: bool,
    #[serde(default = "default_true")]
    check_require: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RawStyleSet {
    named: Option<bool>,
    namespace: Option<bool>,
    #[serde(rename = "default")]
    default_style: Option<bool>,
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

    fn is_subset_of(self, other: &Self) -> bool {
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

fn default_styles() -> BTreeMap<String, StyleSet> {
    BTreeMap::from([
        ("chalk".to_string(), StyleSet::default_style()),
        ("node:path".to_string(), StyleSet::default_style()),
        ("node:util".to_string(), StyleSet::named()),
        ("path".to_string(), StyleSet::default_style()),
        ("util".to_string(), StyleSet::named()),
    ])
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
        ("import util, {inspect} from 'default'", Some(options.clone())),
        ("import util from 'util'", None),
        ("import util from 'node:util'", None),
        ("import * as util from 'util'", None),
        ("import * as util from 'node:util'", None),
        ("const util = require('util')", None),
        ("const util = require('node:util')", None),
        ("require('util')", None),
        ("require('node:util')", None),
        ("import {red} from 'chalk'", None),
        ("import {red as green} from 'chalk'", None),
        (
            "async () => {
                const {red} = await import('chalk');
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
