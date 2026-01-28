use nodejs_built_in_modules::is_nodejs_builtin_module;
use oxc_ast::{
    AstKind,
    ast::{Expression, TSModuleReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_nodejs_modules_diagnostic(span: Span, module_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not import Node.js builtin module `{module_name}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
pub struct NoNodejsModulesConfig {
    /// Array of names of allowed modules. Defaults to an empty array.
    allow: FxHashSet<CompactStr>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoNodejsModules(Box<NoNodejsModulesConfig>);

impl std::ops::Deref for NoNodejsModules {
    type Target = NoNodejsModulesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid the use of Node.js builtin modules. Can be useful for client-side web projects that do not have access to those modules.
    ///
    /// ### Why is this bad?
    ///
    /// Node.js builtins (e.g. `fs`, `path`, `crypto`) are not available in browsers, so importing them in client bundles causes runtime failures or forces bundlers to inject heavy polyfills/shims.
    /// This increases bundle size, can leak server-only logic to the client, and may hide environment mismatches until production.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import fs from 'fs';
    /// import path from 'path';
    ///
    /// var fs = require('fs');
    /// var path = require('path');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import _ from 'lodash';
    /// import foo from 'foo';
    /// import foo from './foo';
    ///
    /// var _ = require('lodash');
    /// var foo = require('foo');
    /// var foo = require('./foo');
    ///
    /// /* eslint import/no-nodejs-modules: ["error", {"allow": ["path"]}] */
    /// import path from 'path';
    /// ```
    NoNodejsModules,
    import,
    style,
    config = NoNodejsModulesConfig,
);

impl Rule for NoNodejsModules {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let module_name = match node.kind() {
            AstKind::ImportExpression(import) => match &import.source {
                Expression::StringLiteral(str_lit) => Some(str_lit.value),
                Expression::TemplateLiteral(temp_lit) if temp_lit.is_no_substitution_template() => {
                    temp_lit.single_quasi()
                }
                _ => None,
            },
            AstKind::TSImportEqualsDeclaration(import) => match &import.module_reference {
                TSModuleReference::ExternalModuleReference(external) => {
                    Some(external.expression.value)
                }
                _ => None,
            },
            AstKind::CallExpression(call) if !call.optional => {
                call.common_js_require().map(|s| s.value)
            }
            AstKind::ImportDeclaration(import) => Some(import.source.value),
            AstKind::ExportNamedDeclaration(export) => {
                export.source.as_ref().map(|item| item.value)
            }
            AstKind::ExportAllDeclaration(export_all) => Some(export_all.source.value),
            _ => return,
        };

        let Some(module_name) = module_name else {
            return;
        };

        if self.allow.contains(module_name.as_str()) {
            return;
        }

        if module_name.starts_with("node:") || is_nodejs_builtin_module(&module_name) {
            ctx.diagnostic(no_nodejs_modules_diagnostic(node.span(), &module_name));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Non Node.js modules
        (r#"import _ from "lodash""#, None),
        (r#"import find from "lodash.find""#, None),
        (r#"import foo from "./foo""#, None),
        (r#"import foo from "../foo""#, None),
        (r#"import foo from "foo""#, None),
        (r#"import foo from "./""#, None),
        (r#"import foo from "@scope/foo""#, None),
        (r#"var _ = require("lodash")"#, None),
        (r#"var find = require("lodash.find")"#, None),
        (r#"var foo = require("./foo")"#, None),
        (r#"var foo = require("../foo")"#, None),
        (r#"var foo = require("foo")"#, None),
        (r#"var foo = require("./")"#, None),
        (r#"var foo = require("@scope/foo")"#, None),
        // With allow option
        (r#"import events from "events""#, Some(serde_json::json!([{ "allow": ["events"] }]))),
        (r#"import path from "path""#, Some(serde_json::json!([{ "allow": ["path"] }]))),
        (r#"var events = require("events")"#, Some(serde_json::json!([{ "allow": ["events"] }]))),
        (r#"var path = require("path")"#, Some(serde_json::json!([{ "allow": ["path"] }]))),
        (
            r#"import path from "path";import events from "events""#,
            Some(serde_json::json!([{ "allow": ["path", "events"] }])),
        ),
        // Node.js protocol with allow option
        (
            r#"import events from "node:events""#,
            Some(serde_json::json!([{ "allow": ["node:events"] }])),
        ),
        (
            r#"var events = require("node:events")"#,
            Some(serde_json::json!([{ "allow": ["node:events"] }])),
        ),
        (r#"import path from "node:path""#, Some(serde_json::json!([{ "allow": ["node:path"] }]))),
        (
            r#"var path = require("node:path")"#,
            Some(serde_json::json!([{ "allow": ["node:path"] }])),
        ),
        (
            r#"import path from "node:path"; import events from "node:events""#,
            Some(serde_json::json!([{ "allow": ["node:path", "node:events"] }])),
        ),
        // TypeScript import equals
        (r#"import foo = require("./foo")"#, None),
        (r#"import foo = require("lodash")"#, None),
        // Dynamic imports
        (r#"import("lodash")"#, None),
        (r"import(`lodash`)", None),
        (r#"import("./foo")"#, None),
        (r#"import("@scope/foo")"#, None),
    ];

    let fail = vec![
        // Node.js builtin modules
        (r#"import path from "path""#, None),
        (r#"import fs from "fs""#, None),
        (r#"var path = require("path")"#, None),
        (r#"var fs = require("fs")"#, None),
        (r"import(`fs`)", None),
        // With allow option
        (r#"import fs from "fs""#, Some(serde_json::json!([{ "allow": ["path"] }]))),
        (r#"import crypto from "crypto""#, Some(serde_json::json!([{ "allow": ["fs"] }]))),
        (r#"import("util")"#, Some(serde_json::json!([{ "allow": ["path"] }]))),
        (r#"export * from "fs""#, Some(serde_json::json!([{ "allow": ["path"] }]))),
        // Node.js protocol
        (r#"import path from "node:path""#, None),
        (r#"var path = require("node:path")"#, None),
        (r#"import fs from "node:fs""#, None),
        (r#"var fs = require("node:fs")"#, None),
        (r#"import crypto from "node:crypto""#, None),
        (r#"import("node:fs")"#, None),
        (r#"export { foo } from "node:path""#, None),
        (r#"import util = require("node:util")"#, None),
        (r#"import fs from "node:fs""#, Some(serde_json::json!([{ "allow": ["node:path"] }]))),
        (r#"import("node:crypto")"#, Some(serde_json::json!([{ "allow": ["node:fs"] }]))),
        // TypeScript import equals
        (r#"import fs = require("fs")"#, None),
        (r#"import path = require("path")"#, None),
        // Export declarations
        (r#"export { foo } from "fs""#, None),
        (r#"export { default as foo } from "crypto""#, None),
        (r#"export * from "path""#, None),
    ];

    Tester::new(NoNodejsModules::NAME, NoNodejsModules::PLUGIN, pass, fail).test_and_snapshot();
}
