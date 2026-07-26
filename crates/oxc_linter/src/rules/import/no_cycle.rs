use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
    sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    ModuleRecord,
    context::LintContext,
    module_graph_visitor::{ModuleGraphVisitorBuilder, ModuleGraphVisitorEvent, VisitFoldWhile},
    rule::{DefaultRuleConfig, Rule},
};

/// The working directory, used to relativize the paths in a cycle description.
///
/// `std::env::current_dir` is a `getcwd` syscall, which is not cheap (on macOS it reconstructs the
/// path by walking up the vnode chain). The working directory does not change during a lint run, so
/// resolve it once for the process, and only on the first `no-cycle` diagnostic — a run that reports
/// no cycles never pays for it, and never panics if the directory is unreadable.
static CWD: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());

fn no_cycle_diagnostic(
    span: Span,
    stack: &[(CompactStr, Arc<ModuleRecord>)],
    cwd: &Path,
) -> OxcDiagnostic {
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

fn format_cycle(stack: &[(CompactStr, Arc<ModuleRecord>)], cwd: &Path) -> String {
    let mut lines = Vec::with_capacity(stack.len() * 2 + 1);

    for (i, (specifier, module)) in stack.iter().enumerate() {
        let relative_path = module
            .resolved_absolute_path
            .strip_prefix(cwd)
            .unwrap_or(&module.resolved_absolute_path)
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
    /// Disallow cyclic dependencies. The rule ensures that there is no resolvable path back
    /// to this module via its dependencies.
    ///
    /// This includes cycles of depth 1 (imported module imports me), up to an effectively
    /// infinite value when the `maxDepth` option is not set.
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
    version = "0.0.13",
    short_description = "Disallow cyclic dependencies that import the current module in its own dependency graph.",
);

impl Rule for NoCycle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        // Almost every module in a real codebase is in no cycle at all, and answering that for the
        // whole module costs one traversal instead of one per direct import. See `can_be_in_cycle`.
        //
        // Only worth it when `max_depth` is unbounded, which is the default: the per-import walks
        // then explore the whole reachable subgraph anyway, so the prefilter replaces work rather
        // than adding to it. With a small `max_depth` the walks are shallow and an unbounded
        // prefilter could cost more than it saves.
        if self.max_depth == u32::MAX && !self.can_be_in_cycle(module_record) {
            return;
        }

        let needle = &module_record.resolved_absolute_path;
        let mut direct_imports = module_record
            .loaded_modules()
            .iter()
            .map(|(key, weak_module_record)| (key.clone(), weak_module_record.upgrade().unwrap()))
            .collect::<Vec<_>>();
        direct_imports.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        for (key, loaded_module_record) in direct_imports {
            if !self.should_traverse_module(&key, &loaded_module_record, module_record) {
                continue;
            }

            let requested_module = module_record.requested_modules[&key][0];
            let span = requested_module.span;
            let mut stack = vec![(key.clone(), Arc::clone(&loaded_module_record))];

            if loaded_module_record.resolved_absolute_path == *needle {
                ctx.diagnostic(self_referencing_cycle_diagnostic(span, requested_module.is_import));
                continue;
            }

            let visitor_result = ModuleGraphVisitorBuilder::default()
                .max_depth(self.max_depth.saturating_sub(1))
                .filter(|(key, val), parent| self.should_traverse_module(key, val, parent))
                .event(|event, (key, val), _| match event {
                    ModuleGraphVisitorEvent::Enter => {
                        stack.push((key.clone(), Arc::clone(val)));
                    }
                    ModuleGraphVisitorEvent::Leave => {
                        stack.pop();
                    }
                })
                .visit_fold(false, &loaded_module_record, |_, (_, val), _| {
                    if val.resolved_absolute_path == *needle {
                        VisitFoldWhile::Stop(true)
                    } else {
                        VisitFoldWhile::Next(false)
                    }
                });

            if visitor_result.result {
                ctx.diagnostic(no_cycle_diagnostic(span, &stack, &CWD));
            }
        }
    }
}

impl NoCycle {
    /// Can `root` be part of a dependency cycle at all?
    ///
    /// `run_once` runs one full graph walk per direct import, and on real codebases nearly every
    /// one of those walks explores `root`'s entire reachable subgraph only to conclude "no cycle".
    /// But `root` is in a cycle if and only if *some* module reachable from it imports it back, so
    /// a single traversal of that subgraph answers the question for every direct import at once.
    /// Where the walks visit the union of their subgraphs once per import, this visits it once.
    ///
    /// Returning `true` only gates the existing walks, which still decide what is reported and
    /// reconstruct the path for the diagnostic. So this must never return `false` when a walk would
    /// have found a cycle, and it doesn't: it traverses the same edges under the same filter, and
    /// is unbounded where the walks are bounded by `max_depth`.
    fn can_be_in_cycle(&self, root: &ModuleRecord) -> bool {
        // Keyed on `Arc` pointer identity, like `ModuleGraphVisitor::traversed`, to avoid cloning a
        // `PathBuf` per node. `root` is deliberately absent: an edge back to it is the answer, not
        // a node to expand.
        let mut visited = FxHashSet::<usize>::default();
        let mut stack = Vec::<Arc<ModuleRecord>>::new();

        if self.visit_dependencies(root, root, &mut visited, &mut stack) {
            return true;
        }
        while let Some(module_record) = stack.pop() {
            if self.visit_dependencies(&module_record, root, &mut visited, &mut stack) {
                return true;
            }
        }
        false
    }

    /// Push `module_record`'s not-yet-visited traversable dependencies onto `stack`, returning
    /// `true` if any of them is `root` itself — i.e. if this module closes a cycle back to `root`.
    fn visit_dependencies(
        &self,
        module_record: &ModuleRecord,
        root: &ModuleRecord,
        visited: &mut FxHashSet<usize>,
        stack: &mut Vec<Arc<ModuleRecord>>,
    ) -> bool {
        for (specifier, weak_module_record) in module_record.loaded_modules().iter() {
            let loaded_module_record = weak_module_record.upgrade().unwrap();
            if !self.should_traverse_module(specifier, &loaded_module_record, module_record) {
                continue;
            }
            // Compared by path, not pointer, because that is what the walks report a cycle on:
            // a file split into multiple sections has one record per section but one path.
            if loaded_module_record.resolved_absolute_path == root.resolved_absolute_path {
                return true;
            }
            if visited.insert(Arc::as_ptr(&loaded_module_record) as usize) {
                stack.push(loaded_module_record);
            }
        }
        false
    }

    fn should_traverse_module(
        &self,
        key: &CompactStr,
        module: &Arc<ModuleRecord>,
        parent: &ModuleRecord,
    ) -> bool {
        let path = &module.resolved_absolute_path;

        let is_node_module = path
            .components()
            .any(|c| matches!(c, Component::Normal(p) if p == OsStr::new("node_modules")));

        if is_node_module {
            return false;
        }

        if self.ignore_types {
            // Equivalent to collecting both entry lists and testing `!is_empty() && all(is_type)`,
            // without materializing either `Vec`. This runs once per graph edge considered.
            let mut has_entry = false;
            let mut all_types = true;

            for entry in
                parent.import_entries.iter().filter(|entry| entry.module_request.name() == key)
            {
                has_entry = true;
                if !entry.is_type {
                    all_types = false;
                    break;
                }
            }

            if all_types {
                for entry in parent.indirect_export_entries.iter().filter(|entry| {
                    entry
                        .module_request
                        .as_ref()
                        .is_some_and(|module_request| module_request.name() == key)
                }) {
                    has_entry = true;
                    if !entry.is_type {
                        all_types = false;
                        break;
                    }
                }
            }

            if has_entry && all_types {
                return false;
            }
        }

        // Allow self referencing named export.
        // In test.js:
        // ```
        // export function example1() { }
        // export * as Example from './test.js';
        // ```
        if path == &parent.resolved_absolute_path
            && let Some(e) = module
                .indirect_export_entries
                .iter()
                .find(|e| e.module_request.as_ref().is_some_and(|r| r.name.as_str() == key))
            && e.export_name.is_name()
        {
            return false;
        }

        true
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

#[test]
fn test_issue_21252_reports_each_cyclic_import() {
    use crate::tester::Tester;

    let pass: Vec<&str> = vec![];
    let fail = vec![
        r"import './b.js';
import './c.js';

export const name = 'a';",
        r"// oxlint-disable-next-line import/no-cycle
import './b.js';
import './c.js';

export const name = 'a';",
    ];

    Tester::new(NoCycle::NAME, NoCycle::PLUGIN, pass, fail)
        .change_rule_path("cycles/issue_21252/a.js")
        .with_import_plugin(true)
        .with_snapshot_suffix("issue_21252")
        .test_and_snapshot();
}
