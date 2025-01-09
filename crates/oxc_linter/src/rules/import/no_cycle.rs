#![allow(clippy::cast_possible_truncation)]
use std::{ffi::OsStr, path::Component, sync::Arc};

use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    module_graph_visitor::{ModuleGraphVisitorBuilder, ModuleGraphVisitorEvent, VisitFoldWhile},
    rule::Rule,
    ModuleRecord,
};

fn no_cycle_diagnostic(span: Span, paths: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Dependency cycle detected")
        .with_help(format!("These paths form a cycle: \n{paths}"))
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-cycle.md>
#[derive(Debug, Clone)]
pub struct NoCycle {
    /// maximum dependency depth to traverse
    max_depth: u32,
    /// ignore type only imports
    ignore_types: bool,
    /// ignore external modules
    #[allow(unused)]
    ignore_external: bool,
    /// Allow cyclic dependency if there is at least one dynamic import in the chain
    #[allow(unused)]
    allow_unsafe_dynamic_cyclic_dependency: bool,
}

impl Default for NoCycle {
    fn default() -> Self {
        Self {
            max_depth: u32::MAX,
            ignore_types: true,
            ignore_external: false,
            allow_unsafe_dynamic_cyclic_dependency: false,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures that there is no resolvable path back to this module via its dependencies.
    ///
    /// This includes cycles of depth 1 (imported module imports me) to "∞" (or Infinity),
    /// if the maxDepth option is not set.
    ///
    /// ### Why is this bad?
    ///
    /// Dependency cycles lead to confusing architectures where bugs become hard to find.
    /// It is common to import an `undefined` value that is caused by a cyclic dependency.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // dep-b.js
    /// import './dep-a.js'
    /// export function b() { /* ... */ }
    /// ```
    /// ```javascript
    /// // dep-a.js
    /// import { b } from './dep-b.js' // reported: Dependency cycle detected.
    /// export function a() { /* ... */ }
    /// ```
    ///
    /// In this example, `dep-a.js` and `dep-b.js` import each other, creating a circular
    /// dependency, which is problematic.
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // dep-b.js
    /// export function b() { /* ... */ }
    /// ```
    /// ```javascript
    /// // dep-a.js
    /// import { b } from './dep-b.js' // no circular dependency
    /// export function a() { /* ... */ }
    /// ```
    ///
    /// In this corrected version, `dep-b.js` no longer imports `dep-a.js`, breaking the cycle.
    NoCycle,
    import,
    restriction
);

impl Rule for NoCycle {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        let default = NoCycle::default();
        Self {
            max_depth: obj
                .and_then(|v| v.get("maxDepth"))
                .and_then(serde_json::Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(default.max_depth, |n| n as u32),
            ignore_types: obj
                .and_then(|v| v.get("ignoreTypes"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(default.ignore_types),
            ignore_external: obj
                .and_then(|v| v.get("ignoreExternal"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(default.ignore_external),
            allow_unsafe_dynamic_cyclic_dependency: obj
                .and_then(|v| v.get("allowUnsafeDynamicCyclicDependency"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(default.allow_unsafe_dynamic_cyclic_dependency),
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        let needle = &module_record.resolved_absolute_path;
        let cwd = std::env::current_dir().unwrap();

        let mut stack = Vec::new();
        let ignore_types = self.ignore_types;
        let visitor_result = ModuleGraphVisitorBuilder::default()
            .max_depth(self.max_depth)
            .filter(move |(key, val): (&CompactStr, &Arc<ModuleRecord>), parent: &ModuleRecord| {
                let path = &val.resolved_absolute_path;

                let is_node_module = path
                    .components()
                    .any(|c| matches!(c, Component::Normal(p) if p == OsStr::new("node_modules")));

                if is_node_module {
                    return false;
                }

                if ignore_types {
                    let import_entries = parent
                        .import_entries
                        .iter()
                        .filter(|entry| entry.module_request.name() == key)
                        .collect::<Vec<_>>();
                    if !import_entries.is_empty()
                        && import_entries.iter().all(|entry| entry.is_type)
                    {
                        return false;
                    }
                }

                true
            })
            .event(|event, (key, val), _| match event {
                ModuleGraphVisitorEvent::Enter => {
                    stack.push((key.clone(), val.resolved_absolute_path.clone()));
                }
                ModuleGraphVisitorEvent::Leave => {
                    stack.pop();
                }
            })
            .visit_fold(false, module_record, |_, (_, val), _| {
                let path = &val.resolved_absolute_path;
                if path == needle {
                    VisitFoldWhile::Stop(true)
                } else {
                    VisitFoldWhile::Next(false)
                }
            });

        if visitor_result.result {
            let span = module_record.requested_modules[&stack[0].0][0].span;
            let help = stack
                .iter()
                .map(|(specifier, path)| {
                    format!(
                        "-> {specifier} - {}",
                        path.strip_prefix(&cwd)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .cow_replace('\\', "/")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            ctx.diagnostic(no_cycle_diagnostic(span, &help));
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"import foo from "./foo.js""#, None),
        (r#"import _ from "lodash""#, None),
        (r#"import foo from "@scope/foo""#, None),
        (r#"var _ = require("lodash")"#, None),
        (r#"var find = require("lodash.find")"#, None),
        (r#"var foo = require("./foo")"#, None),
        (r#"var foo = require("../foo")"#, None),
        (r#"var foo = require("foo")"#, None),
        (r#"var foo = require("./")"#, None),
        (r#"var foo = require("@scope/foo")"#, None),
        (r#"var bar = require("./bar/index")"#, None),
        (r#"var bar = require("./bar")"#, None),
        (r#"var bar = require("./bar")"#, None),
        // TODO: settings 'import/external-module-folders': ['cycles/external'],
        // (r#"import { foo } from "./external-depth-two""#, Some(json!([[{"ignoreExternal":true}]))),
        // (
        // r#"import { foo } from "cycles/external/depth-one""#,
        // Some(json!([[{"ignoreExternal":true}])),
        // ),
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":1}]))),
        (r#"import { foo, bar } from "./es6/depth-two""#, Some(json!([{"maxDepth":1}]))),
        (r#"import("./es6/depth-two").then(function({ foo }) {})"#, Some(json!([{"maxDepth":1}]))),
        // parser: parsers.BABEL_OLD
        // (r#"import type { FooType } from "./es6/depth-one""#, None),
        // (r#"import type { FooType, BarType } from "./es6/depth-one""#, None),
        (
            r#"function bar(){ return import("./es6/depth-one"); } // #2265 1"#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./es6/depth-one-dynamic"; // #2265 2"#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"function bar(){ return import("./es6/depth-one"); } // #2265 3"#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./es6/depth-one-dynamic"; // #2265 4"#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./typescript/ts-types-only-importing-type";"#,
            Some(json!([{"ignoreTypes":true}])),
        ),
        (
            r#"import { foo } from "./typescript/ts-types-only-importing-multiple-types";"#,
            Some(json!([{"ignoreTypes":true}])),
        ),
        (
            r#"import { foo } from "./typescript/ts-types-depth-two";"#,
            Some(json!([{"ignoreTypes":true}])),
        ),
        (
            r#"import { foo } from "./typescript/ts-depth-type-and-value-imports";"#,
            Some(json!([{"ignoreTypes":true}])),
        ),
        // Flow not supported
        // (r#"import { bar } from "./flow-types""#, None),
        // (r#"import { bar } from "./flow-types-only-importing-type""#, None),
        // (r#"import { bar } from "./flow-types-only-importing-multiple-types""#, None),
        // (r#"import { bar } from "./flow-typeof""#, None),
    ];

    let fail = vec![
        // (r#"import { bar } from "./flow-types-some-type-imports""#, None),
        // TODO: settings  'import/resolver': 'webpack', 'import/external-module-folders': ['cycles/external'],
        // (r#"import { foo } from "cycles/external/depth-one""#, None),
        // TODO: settings 'import/external-module-folders': ['cycles/external'],
        // (r#"import { foo } from "./external-depth-two""#, None),
        // (r#"import { foo } from "./es6/depth-one""#, None),
        (r#"import { foo } from "./es6/depth-one""#, Some(json!([{"maxDepth":1}]))),
        // (r#"const { foo } = require("./es6/depth-one")"#, Some(json!([{"commonjs":true}]))),
        // TODO: amd
        // (r#"require(["./es6/depth-one"], d1 => {})"#, Some(json!([{"amd":true}]))),
        // (r#"define(["./es6/depth-one"], d1 => {})"#, Some(json!([{"amd":true}]))),
        (r#"import { foo } from "./es6/depth-one-reexport""#, None),
        (r#"import { foo } from "./es6/depth-one-reexport""#, Some(json!([{"ignoreTypes":true}]))),
        (r#"import { foo } from "./es6/depth-two""#, None),
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":2}]))),
        // (r#"const { foo } = require("./es6/depth-two")"#, Some(json!([{"commonjs":true}]))),
        (r#"import { two } from "./es6/depth-three-star""#, None),
        (r#"import one, { two, three } from "./es6/depth-three-star""#, None),
        (r#"import { bar } from "./es6/depth-three-indirect""#, None),
        (r#"import { bar } from "./es6/depth-three-indirect""#, None),
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":null}]))),
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":"∞"}]))),
        (
            r#"import { foo } from "./es6/depth-one""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./es6/depth-one""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"maxDepth":1}])),
        ),
        // (
        // r#"const { foo } = require("./es6/depth-one")"#,
        // Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"commonjs":true}])),
        // ),
        // TODO: amd
        // (
        // r#"require(["./es6/depth-one"], d1 => {})"#,
        // Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"amd":true}])),
        // ),
        // (
        // r#"define(["./es6/depth-one"], d1 => {})"#,
        // Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"amd":true}])),
        // ),
        (
            r#"import { foo } from "./es6/depth-one-reexport""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./es6/depth-two""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./es6/depth-two""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"maxDepth":2}])),
        ),
        // (
        // r#"const { foo } = require("./es6/depth-two")"#,
        // Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"commonjs":true}])),
        // ),
        (
            r#"import { two } from "./es6/depth-three-star""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import one, { two, three } from "./es6/depth-three-star""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { bar } from "./es6/depth-three-indirect""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { bar } from "./es6/depth-three-indirect""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        (
            r#"import { foo } from "./es6/depth-two""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"maxDepth":null}])),
        ),
        (
            r#"import { foo } from "./es6/depth-two""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"maxDepth":"∞"}])),
        ),
        // TODO: dynamic import
        // (r#"import("./es6/depth-three-star")"#, None),
        // (r#"import("./es6/depth-three-indirect")"#, None),
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":null}]))),
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":"∞"}]))),
        // TODO: dynamic import
        // (r#"function bar(){ return import("./es6/depth-one"); } // #2265 5"#, None),
        // (r#"import { foo } from "./es6/depth-one-dynamic"; // #2265 6"#, None),
        // (r#"function bar(){ return import("./es6/depth-one"); } // #2265 7"#, None),
        // (r#"import { foo } from "./es6/depth-one-dynamic"; // #2265 8"#, None),
        // // Flow not supported
        // (r#"import { bar } from "./flow-types-depth-one""#, None),
        (r#"import { foo } from "./intermediate-ignore""#, None),
        (r#"import { foo } from "./ignore""#, None),
        (
            r#"import { foo } from "./typescript/ts-types-some-type-imports";"#,
            Some(json!([{"ignoreTypes":true}])),
        ),
    ];

    Tester::new(NoCycle::NAME, NoCycle::PLUGIN, pass, fail)
        .change_rule_path("cycles/depth-zero.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
