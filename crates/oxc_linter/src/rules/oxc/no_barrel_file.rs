use crate::{
    ModuleRecord,
    context::LintContext,
    module_graph_visitor::{ModuleGraphVisitorBuilder, VisitFoldWhile},
    rule::{DefaultRuleConfig, Rule},
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

fn no_barrel_file(total: usize, threshold: usize, labels: Vec<LabeledSpan>) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Barrel file detected, {total} modules are loaded."
    ))
    .with_help(format!("Loading {total} modules is slow for runtimes and bundlers.\nThe configured threshold is {threshold}.\nSee also: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7>."))
    .with_labels(labels)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoBarrelFile {
    /// The maximum number of modules that can be re-exported via `export *`
    /// before the rule is triggered.
    threshold: usize,
}

impl Default for NoBarrelFile {
    fn default() -> Self {
        Self { threshold: 100 }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of barrel files where the file contains `export *` statements,
    /// and the total number of modules exceed a threshold.
    ///
    /// The default threshold is 100.
    ///
    /// ### Why is this bad?
    ///
    /// Barrel files that re-export many modules can significantly slow down
    /// applications and bundlers. When a barrel file exports a large number of
    /// modules, importing from it forces the runtime or bundler to process all
    /// the exported modules, even if only a few are actually used. This leads
    /// to slower startup times and larger bundle sizes.
    ///
    /// References:
    ///
    /// * <https://github.com/thepassle/eslint-plugin-barrel-files>
    /// * <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7>
    ///
    /// ### Example
    ///
    /// Invalid:
    ///
    /// ```javascript
    /// export * from 'foo'; // where `foo` loads a subtree of 100 modules
    /// import * as ns from 'foo'; // where `foo` loads a subtree of 100 modules
    /// ```
    ///
    /// Valid:
    ///
    /// ```javascript
    /// export { foo } from 'foo';
    /// ```
    NoBarrelFile,
    oxc,
    restriction,
    config = NoBarrelFile,
);

impl Rule for NoBarrelFile {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoBarrelFile>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        if !module_record.has_module_syntax {
            return;
        }

        let module_requests = module_record
            .indirect_export_entries
            .iter()
            .chain(module_record.star_export_entries.iter())
            .filter_map(|export_entry| {
                if let Some(module_request) = &export_entry.module_request {
                    let import_name = &export_entry.import_name;
                    if import_name.is_all() || import_name.is_all_but_default() {
                        return Some(module_request);
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        let mut labels = vec![];
        let mut total: usize = 0;

        for module_request in module_requests {
            // the own module is counted as well
            total += 1;

            if let Some(remote_module) = module_record.get_loaded_module(module_request.name())
                && let Some(count) = count_loaded_modules(&remote_module)
            {
                total += count;
                labels.push(module_request.span.label(format!("{count} modules")));
            }
        }

        let threshold = self.threshold;
        if total > threshold {
            if labels.is_empty() {
                labels.push(Span::new(0, 0).label("File defined here."));
            }

            ctx.diagnostic(no_barrel_file(total, threshold, labels));
        }
    }
}

fn count_loaded_modules(module_record: &ModuleRecord) -> Option<usize> {
    if module_record.loaded_modules().is_empty() {
        return None;
    }
    Some(
        ModuleGraphVisitorBuilder::default()
            .visit_fold(0, module_record, |acc, _, _| VisitFoldWhile::Next(acc + 1))
            .result,
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"export type * from "foo";"#, None),
        (r#"export type { foo } from "foo";"#, None),
        (r#"export type * from "foo"; export type { bar } from "bar";"#, None),
        (r#"import { foo, bar, baz } from "../import/export-star/models";"#, None),
        (
            r#"import boo from "foo";
                    const test = 0;"#,
            Some(serde_json::json!([{"threshold": 0}])),
        ),
        (r"export const test = 0;", Some(serde_json::json!([{"threshold": 0}]))),
        (
            r"export const test = 0;
            export const other = 1;",
            Some(serde_json::json!([{"threshold": 0}])),
        ),
    ];

    let settings = Some(serde_json::json!([{"threshold": 1}]));

    let fail = vec![(
        r#"export * from "./deep/a.js";
           export * from "./deep/b.js";
           export * from "./deep/c.js";
           export * from "./deep/d.js";"#,
        settings,
    )];

    Tester::new(NoBarrelFile::NAME, NoBarrelFile::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
