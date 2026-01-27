use nodejs_built_in_modules::is_nodejs_builtin_module;
use oxc_ast::{
    AstKind,
    ast::{Expression, TSModuleReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_nodejs_modules_diagnostic(span: Span, module_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not import Node.js builtin module \"{module_name}\""))
        .with_help("This module is not available in browser environments.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoNodejsModules(Box<NoNodejsModulesConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoNodejsModulesConfig {
    /// List of Node.js builtin modules that are allowed to be imported.
    allow: Vec<CompactStr>,
}

impl std::ops::Deref for NoNodejsModules {
    type Target = NoNodejsModulesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids importing Node.js built-in modules.
    ///
    /// ### Why is this bad?
    ///
    /// Client-side web projects do not have access to Node.js built-in modules.
    /// Using them in browser code will cause runtime errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import fs from 'fs';
    /// import path from 'path';
    /// var events = require('events');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import _ from 'lodash';
    /// import foo from './foo';
    /// var bar = require('./bar');
    /// ```
    NoNodejsModules,
    import,
    restriction,
    config = NoNodejsModulesConfig,
);

impl Rule for NoNodejsModules {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // ESM import declarations
            AstKind::ImportDeclaration(import_decl) => {
                let module_name = import_decl.source.value.as_str();
                if is_nodejs_module_not_allowed(module_name, &self.allow) {
                    ctx.diagnostic(no_nodejs_modules_diagnostic(
                        import_decl.source.span,
                        module_name,
                    ));
                }
            }
            // ESM export { } from '...'
            AstKind::ExportNamedDeclaration(export_decl) => {
                if let Some(source) = &export_decl.source {
                    let module_name = source.value.as_str();
                    if is_nodejs_module_not_allowed(module_name, &self.allow) {
                        ctx.diagnostic(no_nodejs_modules_diagnostic(source.span, module_name));
                    }
                }
            }
            // ESM export * from '...'
            AstKind::ExportAllDeclaration(export_decl) => {
                let module_name = export_decl.source.value.as_str();
                if is_nodejs_module_not_allowed(module_name, &self.allow) {
                    ctx.diagnostic(no_nodejs_modules_diagnostic(
                        export_decl.source.span,
                        module_name,
                    ));
                }
            }
            // Dynamic import expressions: import('fs') or import(`fs`)
            AstKind::ImportExpression(import_expr) => {
                let (module_name, span) = match &import_expr.source {
                    Expression::StringLiteral(str_literal) => {
                        (str_literal.value.as_str(), str_literal.span)
                    }
                    Expression::TemplateLiteral(template_literal)
                        if template_literal.is_no_substitution_template() =>
                    {
                        let quasi = &template_literal.quasis[0];
                        (quasi.value.raw.as_str(), quasi.span)
                    }
                    _ => return,
                };
                if is_nodejs_module_not_allowed(module_name, &self.allow) {
                    ctx.diagnostic(no_nodejs_modules_diagnostic(span, module_name));
                }
            }
            // CommonJS require() calls
            AstKind::CallExpression(call_expr) if !call_expr.optional => {
                if let Some(str_literal) = call_expr.common_js_require() {
                    let module_name = str_literal.value.as_str();
                    if is_nodejs_module_not_allowed(module_name, &self.allow) {
                        ctx.diagnostic(no_nodejs_modules_diagnostic(str_literal.span, module_name));
                    }
                }
            }
            // TypeScript import = require('...')
            AstKind::TSImportEqualsDeclaration(import_decl) => {
                if let TSModuleReference::ExternalModuleReference(external) =
                    &import_decl.module_reference
                {
                    let module_name = external.expression.value.as_str();
                    if is_nodejs_module_not_allowed(module_name, &self.allow) {
                        ctx.diagnostic(no_nodejs_modules_diagnostic(
                            external.expression.span,
                            module_name,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check if the module is a Node.js built-in and not in the allow list.
fn is_nodejs_module_not_allowed(module_name: &str, allow: &[CompactStr]) -> bool {
    let module_name = module_name.strip_prefix("node:").unwrap_or(module_name);

    // Get base module name (handle subpaths like fs/promises)
    let base_module = module_name.split('/').next().unwrap_or(module_name);

    is_nodejs_builtin_module(base_module)
        && !allow.iter().any(|allowed| allowed.as_str() == base_module)
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        // ESLint test cases - regular packages
        r#"import _ from "lodash""#,
        r#"import find from "lodash.find""#,
        r#"import foo from "./foo""#,
        r#"import foo from "../foo""#,
        r#"import foo from "foo""#,
        r#"import foo from "./""#,
        r#"import foo from "@scope/foo""#,
        // CommonJS equivalents
        r#"var _ = require("lodash")"#,
        r#"var find = require("lodash.find")"#,
        r#"var foo = require("./foo")"#,
        r#"var foo = require("../foo")"#,
        r#"var foo = require("foo")"#,
        r#"var foo = require("./")"#,
        r#"var foo = require("@scope/foo")"#,
        // TypeScript import = require (non-nodejs)
        r#"import foo = require("foo")"#,
        r#"import foo = require("./foo")"#,
    ];

    let pass_with_options = vec![
        // With allow option
        (r#"import events from "events""#, Some(json!([{ "allow": ["events"] }]))),
        (r#"import path from "path""#, Some(json!([{ "allow": ["path"] }]))),
        (r#"var events = require("events")"#, Some(json!([{ "allow": ["events"] }]))),
        (r#"var path = require("path")"#, Some(json!([{ "allow": ["path"] }]))),
        (
            r#"import path from "path";import events from "events""#,
            Some(json!([{ "allow": ["path", "events"] }])),
        ),
    ];

    let fail = vec![
        // ESLint test cases
        r#"import path from "path""#,
        r#"import fs from "fs""#,
        r#"var path = require("path")"#,
        r#"var fs = require("fs")"#,
        // node: protocol
        r#"import path from "node:path""#,
        r#"import fs from "node:fs""#,
        r#"var path = require("node:path")"#,
        r#"var fs = require("node:fs")"#,
        // subpaths
        r#"import fs from "fs/promises""#,
        r#"var fs = require("fs/promises")"#,
        // dynamic imports
        r#"import("fs")"#,
        r#"import("node:fs")"#,
        r"import(`fs`)",
        r"import(`node:path`)",
        // exports
        r#"export { readFile } from "fs""#,
        r#"export * from "fs""#,
        // TypeScript import = require
        r#"import fs = require("fs")"#,
        r#"import path = require("node:path")"#,
    ];

    let fail_with_options = vec![
        // With allow option that doesn't match
        (r#"import fs from "fs""#, Some(json!([{ "allow": ["path"] }]))),
    ];

    let pass_all: Vec<(&str, Option<serde_json::Value>)> =
        pass.into_iter().map(|s| (s, None)).chain(pass_with_options).collect();

    let fail_all: Vec<(&str, Option<serde_json::Value>)> =
        fail.into_iter().map(|s| (s, None)).chain(fail_with_options).collect();

    Tester::new(NoNodejsModules::NAME, NoNodejsModules::PLUGIN, pass_all, fail_all)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
