use std::borrow::Cow;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn max_dependencies_diagnostic<S: Into<Cow<'static, str>>>(
    message: S,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(message)
        .with_help("Reduce the number of dependencies in this file")
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/max-dependencies.md>
#[derive(Debug, Default, Clone)]
pub struct MaxDependencies(Box<MaxDependenciesConfig>);

#[derive(Debug, Clone)]
pub struct MaxDependenciesConfig {
    max: usize,
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
    /// Forbid modules to have too many dependencies (import or require statements).
    ///
    /// ### Why is this bad?
    ///
    /// This is a useful rule because a module with too many dependencies is a code smell,
    /// and usually indicates the module is doing too much and/or should be broken up into
    /// smaller modules.
    ///
    /// ### Examples
    ///
    /// Given `{"max": 2}`
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
);

impl Rule for MaxDependencies {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Self(Box::new(MaxDependenciesConfig { max, ignore_type_imports: false }))
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(10, |v| usize::try_from(v).unwrap_or(10));
            let ignore_type_imports = config
                .and_then(|config| config.get("ignoreTypeImports"))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            Self(Box::new(MaxDependenciesConfig { max, ignore_type_imports }))
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let mut module_count = module_record.import_entries.len();

        let Some(entry) = module_record.import_entries.get(self.max) else {
            return;
        };

        if self.ignore_type_imports {
            let type_imports =
                module_record.import_entries.iter().filter(|entry| entry.is_type).count();

            module_count -= type_imports;
        }

        if module_count <= self.max {
            return;
        }

        let error = format!(
            "File has too many dependencies ({}). Maximum allowed is {}.",
            module_count, self.max,
        );
        ctx.diagnostic(max_dependencies_diagnostic(error, entry.module_request.span()));
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
        // (
        //     r"
        //      import './foo.js';
        //      import './bar.js';
        //      const a = require('./foo.js');
        //      const b = require('./bar.js');
        //     ",
        //     Some(json!([{"max": 2}])),
        // ),
        (
            r"
                import {x, y, z} from './foo';
            ",
            None,
        ),
        (
            r"
            import type { x } from './foo';
            import type { y } from './foo';
            ",
            Some(json!([{"max": 1, "ignoreTypeImports": true}])),
        ),
    ];

    let fail = vec![
        (
            r"
            import { x } from './foo';
            import { y } from './foo';
            import { z } from './bar';
            ",
            Some(json!([{"max": 1}])),
        ),
        (
            r"
            import { x } from './foo';
            import { y } from './foo';
            import { z } from './baz';
            ",
            Some(json!([{"max": 2}])),
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
            import type { y } from './foo';
            ",
            Some(json!([{"max": 1, }])),
        ),
        (
            r"
            import type { x } from './foo';
            import type { y } from './foo';
            import type { z } from './baz';
            ",
            Some(json!([{"max": 2, "ignoreTypeImports": false}])),
        ),
    ];

    Tester::new(MaxDependencies::NAME, MaxDependencies::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
