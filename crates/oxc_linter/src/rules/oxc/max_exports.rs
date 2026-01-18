use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn max_exports_diagnostic<S: Into<Cow<'static, str>>>(message: S, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message)
        .with_help("Reduce the number of exports in this module or split it into smaller modules")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MaxExports(Box<MaxExportsConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MaxExportsConfig {
    /// Maximum number of exports allowed in a module.
    max: usize,
    /// Whether to ignore type exports when counting.
    ignore_type_exports: bool,
}

impl std::ops::Deref for MaxExports {
    type Target = MaxExportsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxExportsConfig {
    fn default() -> Self {
        Self { max: 10, ignore_type_exports: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Limits the number of exports from a single module.
    ///
    /// ### Why is this bad?
    ///
    /// Modules with many exports are harder to understand and maintain, and often indicate that
    /// the module is doing too much. Splitting such modules improves cohesion and makes imports
    /// more intentional.
    ///
    /// ### Examples
    ///
    /// Given `{ "max": 2 }`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// export const a = 1;
    /// export const b = 2;
    /// export const c = 3; // Too many exports: 3 (max: 2)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// export const a = 1;
    /// export const b = 2; // Allowed: 2 exports (max: 2)
    /// ```
    ///
    /// ### Options
    ///
    /// #### max
    ///
    /// The maximum number of exports allowed. Default is `10`.
    ///
    /// ```json
    /// { "oxc/max-exports": ["error", { "max": 5 }] }
    /// ```
    ///
    /// #### ignoreTypeExports
    ///
    /// When `true`, TypeScript type exports are not counted toward the limit.
    ///
    /// ```json
    /// { "oxc/max-exports": ["error", { "max": 5, "ignoreTypeExports": true }] }
    /// ```
    MaxExports,
    oxc,
    pedantic,
    config = MaxExportsConfig,
);

impl Rule for MaxExports {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        if let Some(max) = value
            .get(0)
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Ok(Self(Box::new(MaxExportsConfig { max, ignore_type_exports: false })))
        } else {
            Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
                .unwrap_or_default()
                .into_inner())
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        if !module_record.has_module_syntax {
            return;
        }

        let entries = module_record
            .local_export_entries
            .iter()
            .chain(module_record.indirect_export_entries.iter())
            .chain(module_record.star_export_entries.iter());

        let mut export_count = 0usize;
        let mut first_exceeding_span: Option<Span> = None;

        for entry in entries {
            if self.ignore_type_exports && entry.is_type {
                continue;
            }

            export_count += 1;
            if export_count == self.max.saturating_add(1) {
                first_exceeding_span = Some(entry.span);
            }
        }

        if export_count <= self.max {
            return;
        }

        let span = first_exceeding_span.unwrap_or_else(|| Span::new(0, 0));
        let error = format!(
            "File has too many exports ({}). Maximum allowed is {}.",
            export_count, self.max,
        );
        ctx.diagnostic(max_exports_diagnostic(error, span));
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r"export const a = 1;", None),
        (
            r"
            export const a = 1;
            export const b = 2;
            ",
            Some(json!([{ "max": 2 }])),
        ),
        (
            r"
            export type Foo = { a: string };
            export type Bar = { b: number };
            ",
            Some(json!([{ "max": 0, "ignoreTypeExports": true }])),
        ),
        (
            r"
            export type Foo = { a: string };
            export const a = 1;
            ",
            Some(json!([{ "max": 1, "ignoreTypeExports": true }])),
        ),
    ];

    let fail = vec![
        (
            r"
            export const a = 1;
            export const b = 2;
            ",
            Some(json!([1])),
        ),
        (
            r"
            export const a = 1;
            export const b = 2;
            export const c = 3;
            ",
            Some(json!([{ "max": 2 }])),
        ),
        (
            r"
            export type Foo = { a: string };
            export const a = 1;
            export const b = 2;
            ",
            Some(json!([{ "max": 1, "ignoreTypeExports": true }])),
        ),
        (r"export * from './mod';", Some(json!([{ "max": 0 }]))),
        (r"export { foo } from './mod';", Some(json!([{ "max": 0 }]))),
    ];

    Tester::new(MaxExports::NAME, MaxExports::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .test_and_snapshot();
}
