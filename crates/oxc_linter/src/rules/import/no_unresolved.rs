use std::path::{Component, Path};

use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::SourceType;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-unresolved): Ensure imports point to a file/module that can be resolved")]
#[diagnostic(severity(warning))]
struct NoUnresolvedDiagnostic(#[label] pub Span);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-unresolved.md>
#[derive(Debug, Default, Clone)]
pub struct NoUnresolved;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures an imported module can be resolved to a module on the local filesystem.
    NoUnresolved,
    nursery
);

impl Rule for NoUnresolved {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();

        for (specifier, requested_modules) in &module_record.requested_modules {
            if module_record.loaded_modules.contains_key(specifier) {
                continue;
            }
            let specifier_path = Path::new(specifier.as_str());
            // skip if the extension is not supported
            if specifier_path.extension().is_some()
                && SourceType::from_path(specifier_path).is_err()
            {
                continue;
            }
            // skip node.js builtin modules
            if specifier.starts_with("node:")
                || (specifier_path
                    .components()
                    .next()
                    .is_some_and(|c| matches!(c, Component::Normal(_)))
                    && NODEJS_BUILTINS.binary_search(&specifier.as_str()).is_ok())
            {
                continue;
            }

            for requested_module in requested_modules {
                // ignore type-only imports and exports
                if requested_module.is_type() {
                    continue;
                }
                ctx.diagnostic(NoUnresolvedDiagnostic(requested_module.span()));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // TODO: handle malformed file?
        // r#"import "./malformed.js""#,
        r#"import foo from "./bar";"#,
        r"import bar from './bar.js';",
        r"import {someThing} from './test-module';",
        r"import fs from 'fs';",
        r"import fs from 'node:fs';",
        r"import('fs');",
        r"import('fs');",
        r#"import * as foo from "a""#,
        r#"export { foo } from "./bar""#,
        r#"export * from "./bar""#,
        r"let foo; export { foo }",
        r#"export * as bar from "./bar""#,
        // parser: parsers.BABEL_OLD
        // r#"export bar from "./bar""#,
        r#"import foo from "./jsx/MyUnCoolComponent.jsx""#,
        r#"var foo = require("./bar")"#,
        r#"require("./bar")"#,
        // TODO: commonjs: false
        // r#"require("./does-not-exist")"#,
        // r#"require("./does-not-exist")"#,
        r#"require(["./bar"], function (bar) {})"#,
        r#"define(["./bar"], function (bar) {})"#,
        r#"require(["./does-not-exist"], function (bar) {})"#,
        r#"define(["require", "exports", "module"], function (r, e, m) { })"#,
        r#"require(["./does-not-exist"])"#,
        r#"define(["./does-not-exist"], function (bar) {})"#,
        // r#"require("./does-not-exist", "another arg")"#,
        r#"proxyquire("./does-not-exist")"#,
        r#"(function() {})("./does-not-exist")"#,
        r"define([0, foo], function (bar) {})",
        r"require(0)",
        r"require(foo)",
        // Unsupported extensions
        r#"import "./test.png""#,
        // ignore type-only imports and exports
        r"import type { m } from 'mod'",
        r"export type * from 'mod'",
    ];

    let fail = vec![
        r#"import reallyfake from "./reallyfake/module""#,
        r"import bar from './baz';",
        r"import bar from './baz';",
        r"import bar from './empty-folder';",
        r"import { DEEP } from 'in-alternate-root';",
        // TODO: dynamic import
        // r#"import('in-alternate-root').then(function({DEEP}) {});"#,
        r#"export { foo } from "./does-not-exist""#,
        r#"export * from "./does-not-exist""#,
        // TODO: dynamic import
        // r#"import('in-alternate-root').then(function({DEEP}) {});"#,
        r#"export * as bar from "./does-not-exist""#,
        r#"export bar from "./does-not-exist""#,
        // r#"var bar = require("./baz")"#,
        // TODO: require expression
        // r#"require("./baz")"#,
        // TODO: amd
        // r#"require(["./baz"], function (bar) {})"#,
        // r#"define(["./baz"], function (bar) {})"#,
        // r#"define(["./baz", "./bar", "./does-not-exist"], function (bar) {})"#,
    ];

    Tester::new(NoUnresolved::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
