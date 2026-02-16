use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn max_dependencies_diagnostic<S: Into<Cow<'static, str>>>(
    message: S,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(message)
        .with_help("Reduce the number of dependencies in this file")
        .with_label(span)
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/max-dependencies.md>
#[derive(Debug, Default, Clone, Deserialize)]
pub struct MaxDependencies(Box<MaxDependenciesConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MaxDependenciesConfig {
    /// Maximum number of dependencies allowed in a file.
    max: usize,
    /// Whether to ignore type imports when counting dependencies.
    ///
    /// ```ts
    /// // Neither of these count as dependencies if `ignoreTypeImports` is true:
    /// import type { Foo } from './foo';
    /// import { type Foo } from './foo';
    /// ```
    ignore_type_imports: bool,
}

impl std::ops::Deref for MaxDependencies {
    type Target = MaxDependenciesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MaxDependenciesConfig {
    fn default() -> Self {
        Self { max: 10, ignore_type_imports: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid modules to have too many dependencies (`import` statements only).
    ///
    /// ### Why is this bad?
    ///
    /// This is a useful rule because a module with too many dependencies is a code smell,
    /// and usually indicates the module is doing too much and/or should be broken up into
    /// smaller modules.
    ///
    /// **NOTE**: This rule only counts `import` statements, and does not count dependencies from
    /// CommonJS `require()` statements. This is a difference from the original
    /// eslint-import-plugin rule.
    ///
    /// ### Examples
    ///
    /// Given `{ "max": 2 }`
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import a from './a';
    /// import b from './b';
    /// import c from './c'; // Too many dependencies: 3 (max: 2)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import a from './a';
    /// import b from './b'; // Allowed: 2 dependencies (max: 2)
    /// ```
    MaxDependencies,
    import,
    pedantic,
    config = MaxDependenciesConfig,
);

impl Rule for MaxDependencies {
    // TODO(breaking change): Remove support for the `["error", 3]`-style configuration
    // option, as it is not actually supported in the original rule.
    // Only the object configuration option should be supported in the future.
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        if let Some(max) = value
            .get(0)
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Ok(Self(Box::new(MaxDependenciesConfig { max, ignore_type_imports: false })))
        } else {
            serde_json::from_value::<DefaultRuleConfig<Self>>(value)
                .map(DefaultRuleConfig::into_inner)
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let mut dependency_sources = FxHashSet::default();
        let mut module_count = 0;
        let mut first_exceeding_span = None;

        for entry in &module_record.import_entries {
            if self.ignore_type_imports && entry.is_type {
                continue;
            }

            if dependency_sources.insert(entry.module_request.name()) {
                module_count += 1;
                if module_count > self.max && first_exceeding_span.is_none() {
                    first_exceeding_span = Some(entry.module_request.span);
                }
            }
        }

        if module_count <= self.max {
            return;
        }

        let Some(span) = first_exceeding_span else {
            return;
        };

        let error = format!(
            "File has too many dependencies ({}). Maximum allowed is {}.",
            module_count, self.max,
        );
        ctx.diagnostic(max_dependencies_diagnostic(error, span));
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"
            import './foo.js';
            ",
            None,
        ),
        (
            r"
             import './foo.js';
             import './bar.js';",
            None,
        ),
        (
            r"
             import './foo.js';
             import './bar.js';",
            Some(json!([])),
        ),
        // (
        //     r"
        //      import './foo.js';
        //      import './bar.js';
        //      const a = require('./foo.js');
        //      const b = require('./bar.js');
        //     ",
        //     Some(json!([{ "max": 2 }])),
        // ),
        (
            r"
                import {x, y, z} from './foo';
            ",
            Some(json!([{ "max": 1 }])),
        ),
        (
            r"
            import type { x } from './foo';
            import type { y } from './foo';
            ",
            Some(json!([{ "max": 1, "ignoreTypeImports": true }])),
        ),
        (
            r"
            import type { x } from './foo';
            import { type z } from './foo';
            import { y } from './foo';
            ",
            Some(json!([{ "max": 1, "ignoreTypeImports": true }])),
        ),
        (
            r"
            import { type x } from './foo';
            import { type y } from './foo';
            ",
            Some(json!([{ "max": 1, "ignoreTypeImports": true }])),
        ),
        (
            r"
            import type { x } from './foo';
            import type { y } from './foo';
            ",
            Some(json!([2])),
        ),
        (
            r"
            import { w } from './foo';
            // This should only count as one dependency, because it's 3 named imports from the same place.
            import { x, y, z } from './bar';
            ",
            Some(json!([{ "max": 2 }])),
        ),
        (
            r"
            import { w } from './foo';
            // This should not count, because both are type imports
            import { type x, type y } from './bar';
            import { z } from './baz';
            ",
            Some(json!([{ "max": 2, "ignoreTypeImports": true }])),
        ),
    ];

    let fail = vec![
        (
            r"
            import { x } from './foo';
            import { y } from './bar';
            import { z } from './baz';
            ",
            Some(json!([1])),
        ),
        (
            r"
            import { x } from './foo';
            import { y } from './bar';
            import { z } from './baz';
            ",
            Some(json!([{ "max": 1 }])),
        ),
        (
            r"
            import { x } from './foo';
            import { y } from './bar';
            import { z } from './baz';
            ",
            Some(json!([{ "max": 2 }])),
        ),
        // (
        //     r"
        //     import { x } from './foo';
        //     require('./bar'); const path = require('path');
        //     import { z } from './baz';
        //     ",
        //     Some(json!([{"max": 2}])),
        // ),
        (
            r"
            import type { x } from './foo';
            import type { y } from './bar';
            ",
            Some(json!([{ "max": 1 }])),
        ),
        (
            r"
            import type { x } from './foo';
            import type { y } from './bar';
            import type { z } from './baz';
            ",
            Some(json!([{ "max": 2, "ignoreTypeImports": false }])),
        ),
        (
            r"
            import type { x } from './foo';
            import { type y } from './bar';
            import { type z } from './baz';
            ",
            Some(json!([{ "max": 2, "ignoreTypeImports": false }])),
        ),
        (
            r"
            import { x } from './foo';
            import { y } from './bar';
            import { z } from './baz.json' with { type: 'json' };
            ",
            Some(json!([{ "max": 2 }])),
        ),
        (
            r"
            import { w } from './foo';
            // This should still count, because only one is a type import
            import { type x, y } from './bar';
            import { z } from './baz';
            ",
            Some(json!([{ "max": 2, "ignoreTypeImports": true }])),
        ),
        (
            r"
            import { definePlugin } from '@oxlint/plugins';
            import { fsSync } from 'node:fs';
            import { path } from 'node:path';
            ",
            Some(json!([{ "max": 2 }])),
        ),
    ];

    Tester::new(MaxDependencies::NAME, MaxDependencies::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
