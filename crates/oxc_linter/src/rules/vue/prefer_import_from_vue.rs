use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn prefer_import_from_vue_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("enforce import from 'vue' instead of import from '@vue/*'")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferImportFromVue;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce import from 'vue' instead of import from '@vue/*'.
    ///
    /// ### Why is this bad?
    ///
    /// Imports from the following modules are almost always wrong. You should import from vue instead.
    /// - `@vue/runtime-dom`
    /// - `@vue/runtime-core`
    /// - `@vue/reactivity`
    /// - `@vue/shared`
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { createApp } from '@vue/runtime-dom'
    /// import { Component } from '@vue/runtime-core'
    /// import { ref } from '@vue/reactivity'
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { createApp, ref, Component } from 'vue'
    /// ```
    PreferImportFromVue,
    vue,
    correctness,
    fix
);

const VUE_MODULES: &[&str; 4] =
    &["@vue/reactivity", "@vue/runtime-core", "@vue/runtime-dom", "@vue/shared"];
impl Rule for PreferImportFromVue {
    fn run_once(&self, ctx: &LintContext) {
        let records = ctx.module_record();

        for entry in &records.import_entries {
            if VUE_MODULES.contains(&entry.module_request.name.as_str()) {
                ctx.diagnostic_with_fix(
                    prefer_import_from_vue_diagnostic(entry.module_request.span),
                    |fixer| fixer.replace(entry.module_request.span, "'vue'".to_string()),
                );
            }
        }

        for entry in &records.indirect_export_entries {
            let Some(name) = &entry.module_request else {
                continue;
            };
            if VUE_MODULES.contains(&name.name.as_str()) {
                ctx.diagnostic_with_fix(prefer_import_from_vue_diagnostic(name.span), |fixer| {
                    fixer.replace(name.span, "'vue'".to_string())
                });
            }
        }

        for entry in &records.star_export_entries {
            let Some(name) = &entry.module_request else {
                continue;
            };
            if VUE_MODULES.contains(&name.name.as_str()) {
                ctx.diagnostic_with_fix(prefer_import_from_vue_diagnostic(name.span), |fixer| {
                    fixer.replace(name.span, "'vue'".to_string())
                });
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        !ctx.source_type().is_typescript_definition()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("import { createApp } from 'vue'", None, None, None),
        ("import { ref, reactive } from '@vue/composition-api'", None, None, None),
        ("export { createApp } from 'vue'", None, None, None),
        ("export * from 'vue'", None, None, None),
        ("import Foo from 'foo'", None, None, None),
        (
            "import { createApp } from 'vue'
			    export { createApp }",
            None,
            None,
            None,
        ),
        (
            "import { unknown } from '@vue/runtime-dom'",
            None,
            None,
            Some(PathBuf::from("test.d.ts")),
        ),
    ];

    let fail = vec![
        ("import { createApp } from '@vue/runtime-dom'", None, None, None),
        ("import { computed } from '@vue/runtime-core'", None, None, None),
        ("import { computed } from '@vue/reactivity'", None, None, None),
        ("import { normalizeClass } from '@vue/shared'", None, None, None),
        ("import { unknown } from '@vue/reactivity'", None, None, None),
        ("import { unknown } from '@vue/runtime-dom'", None, None, None),
        ("import * as Foo from '@vue/reactivity'", None, None, None),
        ("import * as Foo from '@vue/runtime-dom'", None, None, None),
        ("export * from '@vue/reactivity'", None, None, None),
        ("export * from '@vue/runtime-dom'", None, None, None),
        ("export { computed } from '@vue/reactivity'", None, None, None),
        ("export { computed } from '@vue/runtime-dom'", None, None, None),
        ("export { unknown } from '@vue/reactivity'", None, None, None),
        ("export { unknown } from '@vue/runtime-dom'", None, None, None),
        ("import unknown from '@vue/reactivity'", None, None, None),
        ("import unknown from '@vue/runtime-dom'", None, None, None),
    ];

    let fix = vec![
        ("import { createApp } from '@vue/runtime-dom'", "import { createApp } from 'vue'", None),
        ("import { computed } from '@vue/runtime-core'", "import { computed } from 'vue'", None),
        ("import { computed } from '@vue/reactivity'", "import { computed } from 'vue'", None),
        (
            "import { normalizeClass } from '@vue/shared'",
            "import { normalizeClass } from 'vue'",
            None,
        ),
        ("import { unknown } from '@vue/runtime-dom'", "import { unknown } from 'vue'", None),
        ("import * as Foo from '@vue/runtime-dom'", "import * as Foo from 'vue'", None),
        ("export { computed } from '@vue/reactivity'", "export { computed } from 'vue'", None),
        ("export { computed } from '@vue/runtime-dom'", "export { computed } from 'vue'", None),
        ("import unknown from '@vue/runtime-dom'", "import unknown from 'vue'", None),
    ];
    Tester::new(PreferImportFromVue::NAME, PreferImportFromVue::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
