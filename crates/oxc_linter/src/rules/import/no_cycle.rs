use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_module_graph::{self as mg, NormalModule};
use oxc_span::Span;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_cycle_diagnostic(span: Span, stack: &[(String, PathBuf)], cwd: &Path) -> OxcDiagnostic {
    let cycle_description = format_cycle(stack, cwd);
    OxcDiagnostic::warn("Dependency cycle detected")
        .with_help("Refactor to remove the cycle. Consider extracting shared code into a separate module that both files can import.")
        .with_note(format!("These paths form a cycle:\n{cycle_description}"))
        .with_label(span)
}

fn self_referencing_cycle_diagnostic(span: Span, is_import: bool) -> OxcDiagnostic {
    OxcDiagnostic::warn("Dependency cycle detected")
        .with_help(if is_import {
            "Remove the self-referencing import."
        } else {
            "Remove the self-referencing export and consider using a named export instead."
        })
        .with_label(span.primary_label("this module references itself"))
}

fn format_cycle(stack: &[(String, PathBuf)], cwd: &Path) -> String {
    let mut lines = Vec::with_capacity(stack.len() * 2 + 1);

    for (i, (specifier, path)) in stack.iter().enumerate() {
        let relative_path = path
            .strip_prefix(cwd)
            .unwrap_or(path)
            .to_string_lossy()
            .cow_replace('\\', "/")
            .into_owned();

        if i == 0 {
            lines.push(format!("╭──▶ {specifier} ({relative_path})"));
        } else {
            lines.push("│         ⬇ imports".to_string());
            lines.push(format!("│    {specifier} ({relative_path})"));
        }
    }

    // Close the cycle - it imports back to the original file
    lines.push("╰─────────╯ imports the current file".to_string());

    lines.join("\n")
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-cycle.md>
#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoCycle {
    /// Maximum dependency depth to traverse
    max_depth: u32,
    /// Ignore type-only imports
    ignore_types: bool,
    /// Ignore external modules
    ignore_external: bool,
    /// Allow cyclic dependency if there is at least one dynamic import in the chain
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
    /// This includes cycles of depth 1 (imported module imports me) to an effectively
    /// infinite value, if the `maxDepth` option is not set.
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
    restriction,
    config = NoCycle,
);

impl Rule for NoCycle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(module) = ctx.current_module() else {
            return;
        };
        let Some(my_idx) = ctx.current_module_idx() else {
            return;
        };
        let graph = match ctx.module_graph() {
            Some(g) => g,
            None => return,
        };

        let needle = my_idx;
        let cwd = std::env::current_dir().unwrap();
        let ignore_types = self.ignore_types;
        let max_depth = self.max_depth;

        // For each import record in this module, do a DFS to see if we can
        // reach back to `needle`.
        for record in &module.import_records {
            let Some(target_idx) = record.resolved_module else {
                continue;
            };

            // Filter: ignore type-only imports if configured.
            if ignore_types {
                let all_type_only = self.all_type_only_for_specifier(&record.specifier, module);
                if all_type_only {
                    continue;
                }
            }

            // Skip node_modules.
            if let Some(target) = graph.normal_module(target_idx) {
                if target.path.to_string_lossy().contains("node_modules") {
                    continue;
                }
            }

            // Allow self referencing named export.
            if target_idx == needle {
                if let Some(target) = graph.normal_module(target_idx) {
                    if target.indirect_export_entries.iter().any(|e| {
                        e.module_request.as_str() == record.specifier.as_str()
                            && !e.exported_name.is_empty()
                            && e.exported_name.as_str() != "*"
                    }) {
                        continue;
                    }
                }
            }

            // DFS from target_idx looking for needle.
            let mut stack: Vec<(String, PathBuf)> = Vec::new();
            let mut visited = FxHashSet::default();
            visited.insert(my_idx); // don't revisit ourselves

            let found = dfs_find_cycle(
                graph,
                target_idx,
                needle,
                &record.specifier,
                &mut stack,
                &mut visited,
                0,
                max_depth,
                ignore_types,
                ctx,
            );

            if found {
                let span = record.specifier_span;
                if stack.len() == 1 && stack[0].1 == module.path {
                    // Self-referencing cycle.
                    ctx.diagnostic(self_referencing_cycle_diagnostic(span, record.is_import));
                } else {
                    ctx.diagnostic(no_cycle_diagnostic(span, &stack, &cwd));
                }
                // Only report the first cycle found for this module.
                return;
            }
        }
    }
}

impl NoCycle {
    /// Check if ALL import entries and indirect export entries for a given
    /// specifier are type-only.
    fn all_type_only_for_specifier(&self, specifier: &str, module: &NormalModule) -> bool {
        // If any non-type indirect export entry references this specifier,
        // the import is not fully type-only.
        let has_value_indirect_export = module
            .indirect_export_entries
            .iter()
            .any(|entry| entry.module_request.as_str() == specifier && !entry.is_type);
        if has_value_indirect_export {
            return false;
        }

        // Check named imports that reference this specifier (via record_idx).
        // All of them must be type-only, and there must be at least one.
        let matching_imports: Vec<_> = module
            .named_imports
            .values()
            .filter(|import| {
                module
                    .import_records
                    .get(import.record_idx.index())
                    .is_some_and(|rec| rec.specifier.as_str() == specifier)
            })
            .collect();

        !matching_imports.is_empty() && matching_imports.iter().all(|import| import.is_type)
    }
}

/// DFS to find if `needle` is reachable from `current`.
fn dfs_find_cycle(
    graph: &mg::graph::ModuleGraph,
    current: mg::types::ModuleIdx,
    needle: mg::types::ModuleIdx,
    specifier: &str,
    stack: &mut Vec<(String, PathBuf)>,
    visited: &mut FxHashSet<mg::types::ModuleIdx>,
    depth: u32,
    max_depth: u32,
    ignore_types: bool,
    ctx: &LintContext<'_>,
) -> bool {
    if depth > max_depth {
        return false;
    }

    let current_path = match graph.normal_module(current) {
        Some(m) => m.path.clone(),
        None => return false,
    };

    stack.push((specifier.to_string(), current_path));

    // Check if we've reached the needle (cycle found) BEFORE the visited check.
    if current == needle {
        return true;
    }

    if !visited.insert(current) {
        stack.pop();
        return false;
    }

    let Some(current_module) = graph.normal_module(current) else {
        stack.pop();
        return false;
    };

    // Skip node_modules.
    if current_module.path.to_string_lossy().contains("node_modules") {
        stack.pop();
        return false;
    }

    for rec in &current_module.import_records {
        let Some(dep_idx) = rec.resolved_module else {
            continue;
        };

        // Filter type-only.
        if ignore_types {
            let has_value_import = current_module.named_imports.values().any(|imp| {
                if let Some(r) = current_module.import_records.get(imp.record_idx.index()) {
                    r.specifier == rec.specifier && !imp.is_type
                } else {
                    false
                }
            });
            let has_value_indirect = current_module
                .indirect_export_entries
                .iter()
                .any(|e| e.module_request.as_str() == rec.specifier.as_str() && !e.is_type);
            // If there are no value imports and no value indirect exports,
            // it's entirely type-only — skip it. But only if there IS at least
            // one type-only link (to avoid skipping side-effect-only imports).
            if !has_value_import && !has_value_indirect {
                let has_any_type_link = current_module.named_imports.values().any(|imp| {
                    if let Some(r) = current_module.import_records.get(imp.record_idx.index()) {
                        r.specifier == rec.specifier && imp.is_type
                    } else {
                        false
                    }
                }) || current_module
                    .indirect_export_entries
                    .iter()
                    .any(|e| e.module_request.as_str() == rec.specifier.as_str() && e.is_type);
                if has_any_type_link {
                    continue;
                }
            }
        }

        // Skip node_modules.
        if let Some(dep) = graph.normal_module(dep_idx) {
            if dep.path.to_string_lossy().contains("node_modules") {
                continue;
            }
        }

        // Allow self referencing named export.
        if dep_idx == current
            && let Some(dep) = graph.normal_module(dep_idx)
            && dep.indirect_export_entries.iter().any(|e| {
                e.module_request.as_str() == rec.specifier.as_str()
                    && !e.exported_name.is_empty()
                    && e.exported_name.as_str() != "*"
            })
        {
            continue;
        }

        if dfs_find_cycle(
            graph,
            dep_idx,
            needle,
            &rec.specifier,
            stack,
            visited,
            depth + 1,
            max_depth,
            ignore_types,
            ctx,
        ) {
            return true;
        }
    }

    stack.pop();
    false
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
        (r#"import { foo } from "./typescript/ts-types-re-exporting-type";"#, None),
        (r"export function Foo() {}; export * as ns from './depth-zero'", None),
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
        // effectively unlimited:
        (r#"import { foo } from "./es6/depth-two""#, None),
        // Use default value, effectively unlimited:
        (r#"import { foo } from "./es6/depth-two""#, Some(json!([]))),
        // These are not valid config options and just fell back to the default value previously:
        // (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":null}]))),
        // (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":"∞"}]))),
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
        // Equivalent to the commented tests below.
        (
            r#"import { foo } from "./es6/depth-two""#,
            Some(json!([{"allowUnsafeDynamicCyclicDependency":true}])),
        ),
        // These are not valid config options and just fell back to the default value previously:
        // (
        //     r#"import { foo } from "./es6/depth-two""#,
        //     Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"maxDepth":null}])),
        // ),
        // (
        //     r#"import { foo } from "./es6/depth-two""#,
        //     Some(json!([{"allowUnsafeDynamicCyclicDependency":true,"maxDepth":"∞"}])),
        // ),
        // TODO: dynamic import
        // (r#"import("./es6/depth-three-star")"#, None),
        // (r#"import("./es6/depth-three-indirect")"#, None),
        // These are not valid config options and just fell back to the default value previously:
        // (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":null}]))),
        // (r#"import { foo } from "./es6/depth-two""#, Some(json!([{"maxDepth":"∞"}]))),
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
        (
            r#"import { foo } from "./typescript/ts-types-re-exporting-type";"#,
            Some(json!([{"ignoreTypes":false}])),
        ),
        (r"export function Foo() {}; export * from './depth-zero'", None),
        (r"import * as depthZero from './depth-zero'", None),
    ];

    Tester::new(NoCycle::NAME, NoCycle::PLUGIN, pass, fail)
        .change_rule_path("cycles/depth-zero.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}

#[test]
fn test_issue_19245_type_only_branch_does_not_hide_cycle() {
    use crate::tester::Tester;

    let pass: Vec<&str> = vec![];
    let fail = vec![
        r"import { installmentLoanManager } from './installmentLoanManager';
import { aaaInternal } from './aaaInternal';

export const balanceSweepDetailsManager = {
  call(): string {
    return installmentLoanManager.call() + aaaInternal.call();
  },
};",
    ];

    Tester::new(NoCycle::NAME, NoCycle::PLUGIN, pass, fail)
        .change_rule_path("cycles/typescript/issue_19245/balanceSweepDetailsManager.ts")
        .with_import_plugin(true)
        .with_snapshot_suffix("issue_19245")
        .test_and_snapshot();
}
