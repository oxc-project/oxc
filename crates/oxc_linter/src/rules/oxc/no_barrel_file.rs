use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    module_graph_visitor::{ModuleGraphVisitorBuilder, VisitFoldWhile},
    rule::Rule,
    ModuleRecord,
};

fn no_barrel_file(total: usize, threshold: usize, labels: Vec<LabeledSpan>) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Barrel file detected, {total} modules are loaded."
    ))
    .with_help(format!("Loading {total} modules is slow for runtimes and bundlers.\nThe configured threshold is {threshold}.\nSee also: <https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-7>."))
    .with_labels(labels)
}
#[derive(Debug, Clone)]
pub struct NoBarrelFile {
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
    /// The default threshold is 100;
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
    restriction
);

impl Rule for NoBarrelFile {
    #[allow(clippy::cast_possible_truncation)]
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            threshold: value
                .get(0)
                .and_then(|config| config.get("threshold"))
                .and_then(serde_json::Value::as_u64)
                .map_or(NoBarrelFile::default().threshold, |n| n as usize),
        }
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
            if let Some(remote_module) =
                module_record.loaded_modules.read().unwrap().get(module_request.name())
            {
                if let Some(count) = count_loaded_modules(remote_module) {
                    total += count;
                    labels.push(module_request.span().label(format!("{count} modules")));
                }
            };
        }

        let threshold = self.threshold;
        if total >= threshold {
            ctx.diagnostic(no_barrel_file(total, threshold, labels));
        }
    }
}

fn count_loaded_modules(module_record: &ModuleRecord) -> Option<usize> {
    if module_record.loaded_modules.read().unwrap().is_empty() {
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
