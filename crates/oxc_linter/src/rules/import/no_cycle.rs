use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use cow_utils::CowUtils;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AllowWarnDeny, ModuleRecord,
    context::{ProjectLintContext, RuleLabel},
    module_graph_visitor::{ModuleGraphVisitorBuilder, ModuleGraphVisitorEvent, VisitFoldWhile},
    rule::{DefaultRuleConfig, ProjectRule, Rule},
    rules::RuleEnum,
};

fn no_cycle_diagnostic(span: Span, stack: &[(CompactStr, PathBuf)], cwd: &Path) -> OxcDiagnostic {
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

fn format_cycle(stack: &[(CompactStr, PathBuf)], cwd: &Path) -> String {
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
#[derive(Debug, Clone, Copy, JsonSchema, Deserialize)]
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
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }
}

impl ProjectRule for NoCycle {
    fn run_on_project(
        &self,
        ctx: &ProjectLintContext<'_>,
    ) -> FxHashMap<PathBuf, Vec<OxcDiagnostic>> {
        let target_paths = ctx.target_paths(|rule| match rule {
            RuleEnum::ImportNoCycle(config) => Some(config),
            _ => None,
        });
        if target_paths.is_empty() {
            return FxHashMap::default();
        }

        collect_cycle_diagnostics(ctx, &target_paths)
    }
}

impl NoCycle {
    /// builds diagnostics for `module_record`'s direct imports that are
    /// `in_same_cycle`. cycles have already been detected here—this just builds
    /// a nice error display via BFS
    fn module_diagnostics(
        self,
        module_record: &ModuleRecord,
        in_same_cycle: impl Fn(&Path) -> bool,
    ) -> Vec<OxcDiagnostic> {
        let needle = &module_record.resolved_absolute_path;
        let mut direct_imports = module_record
            .loaded_modules()
            .iter()
            .map(|(key, weak_module_record)| (key.clone(), weak_module_record.upgrade().unwrap()))
            .collect::<Vec<_>>();
        direct_imports.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut diagnostics = Vec::new();

        for (key, loaded_module_record) in direct_imports {
            if !should_traverse_module(
                self.ignore_types,
                &key,
                &loaded_module_record,
                module_record,
            ) {
                continue;
            }

            let requested_module = module_record.requested_modules[&key][0];
            let span = requested_module.span;
            let mut stack =
                vec![(key.clone(), loaded_module_record.resolved_absolute_path.clone())];

            if loaded_module_record.resolved_absolute_path == *needle {
                diagnostics
                    .push(self_referencing_cycle_diagnostic(span, requested_module.is_import));
                continue;
            }

            if !in_same_cycle(&loaded_module_record.resolved_absolute_path) {
                continue;
            }

            let visitor_result = ModuleGraphVisitorBuilder::default()
                .max_depth(self.max_depth.saturating_sub(1))
                .filter(|(key, val), parent| {
                    should_traverse_module(self.ignore_types, key, val, parent)
                })
                .event(|event, (key, val), _| match event {
                    ModuleGraphVisitorEvent::Enter => {
                        stack.push((key.clone(), val.resolved_absolute_path.clone()));
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
                diagnostics.push(no_cycle_diagnostic(
                    span,
                    &stack,
                    &std::env::current_dir().unwrap(),
                ));
            }
        }

        diagnostics
    }
}

fn should_traverse_module(
    ignore_types: bool,
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

    if ignore_types {
        // Equivalent to collecting both entry lists and testing `!is_empty() && all(is_type)`,
        // without materializing either `Vec`. This runs once per graph edge considered.
        let mut types = parent
            .import_entries
            .iter()
            .filter(|entry| entry.module_request.name() == key)
            .map(|entry| entry.is_type)
            .chain(
                parent
                    .indirect_export_entries
                    .iter()
                    .filter(|entry| {
                        entry
                            .module_request
                            .as_ref()
                            .is_some_and(|module_request| module_request.name() == key)
                    })
                    .map(|entry| entry.is_type),
            )
            .peekable();

        if types.peek().is_some() && types.all(|is_type| is_type) {
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

mod graph_cycles {
    use std::path::Path;

    use petgraph::graphmap::DiGraphMap;
    use rayon::prelude::*;
    use rustc_hash::FxHashMap;

    use crate::context::ProjectModules;

    use super::should_traverse_module;

    type ImportGraph<'a> = DiGraphMap<&'a Path, ()>;

    /// Builds a directed graph in parallel so cycles can be detected
    fn build_import_graph<'a>(ignore_types: bool, modules: &ProjectModules<'a>) -> ImportGraph<'a> {
        let edges: Vec<(&'a Path, &'a Path)> = modules
            .par_iter()
            .flat_map_iter(|(&path, &records)| {
                records.iter().flat_map(move |module_record| {
                    let loaded_modules = module_record.loaded_modules();
                    loaded_modules
                        .iter()
                        .map(|(specifier, weak_module_record)| {
                            (specifier, weak_module_record.upgrade().unwrap())
                        })
                        .filter(|(specifier, imported)| {
                            should_traverse_module(ignore_types, specifier, imported, module_record)
                        })
                        // grabs module path from modules since `Weak` upgrade
                        // doesn't live long enough
                        .filter_map(|(_, imported)| {
                            modules.get_key_value(imported.resolved_absolute_path.as_path())
                        })
                        .map(move |(&to, _)| (path, to))
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        let mut graph = ImportGraph::new();
        for &path in modules.keys() {
            graph.add_node(path);
        }
        for (from, to) in edges {
            graph.add_edge(from, to, ());
        }

        graph
    }

    pub type CycleId = usize;

    /// Turns a directed module graph into a map of `Path` to `CycleId`. This is done linearly via Tarjan's algorithm
    fn cycles_by_path<'a>(graph: &ImportGraph<'a>) -> FxHashMap<&'a Path, CycleId> {
        let is_cycle = |component: &[&Path]| -> bool {
            component.len() > 1 || graph.contains_edge(component[0], component[0])
        };
        petgraph::algo::tarjan_scc(graph)
            .into_iter()
            .enumerate()
            .filter(|(_, component)| is_cycle(component))
            // strongly connected components are unique in a directed graph
            .flat_map(|(id, component)| component.into_iter().map(move |module| (module, id)))
            .collect()
    }

    /// Linearly detects cycles in the module graph. Returns which cycle a path
    /// is in, if any
    pub fn cycles_by_module<'a>(
        ignore_types: bool,
        modules: &ProjectModules<'a>,
    ) -> FxHashMap<&'a Path, CycleId> {
        cycles_by_path(&build_import_graph(ignore_types, modules))
    }
}

/// Converts the cycles found in `cycles_by_module` into diagnostics.
/// Abides by disable directives like other rules
fn collect_cycle_diagnostics(
    project: &ProjectLintContext<'_>,
    target_paths: &FxHashMap<PathBuf, (AllowWarnDeny, NoCycle)>,
) -> FxHashMap<PathBuf, Vec<OxcDiagnostic>> {
    let modules = project.modules();
    // types must be checked for the whole graph if any config enables them
    let include_type_edges = target_paths.values().any(|(_, config)| !config.ignore_types);
    let cycles = graph_cycles::cycles_by_module(!include_type_edges, modules);

    let rule = RuleLabel::new(NoCycle::PLUGIN, NoCycle::NAME);

    modules
        .par_iter()
        .filter_map(|(&path, records)| {
            let cycle = cycles.get(path)?;
            let (path, &(severity, config)) = target_paths.get_key_value(path)?;

            let diagnostics: Vec<OxcDiagnostic> = records
                .iter()
                .flat_map(|module_record| {
                    config
                        .module_diagnostics(module_record, |imported: &Path| {
                            cycles.get(imported) == Some(cycle)
                        })
                        .into_iter()
                        .map(|diagnostic| (diagnostic, module_record.source_text_offset))
                })
                .filter_map(|(diagnostic, section_offset)| {
                    project.finalize_diagnostic(path, diagnostic, rule, severity, section_offset)
                })
                .collect();

            (!diagnostics.is_empty()).then(|| (path.clone(), diagnostics))
        })
        .collect()
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
