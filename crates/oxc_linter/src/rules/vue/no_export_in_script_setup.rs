use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::Rule,
};

fn no_export_in_script_setup_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("<script setup>` cannot contain ES module exports.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExportInScriptSetup;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `export` in `<script setup>`
    ///
    /// ### Why is this bad?
    ///
    /// The previous version of `<script setup>` RFC used `export` to define variables used in templates,
    /// but the new `<script setup>` RFC has been updated to define without using `export`.
    /// See [Vue RFCs - 0040-script-setup](https://github.com/vuejs/rfcs/blob/master/active-rfcs/0040-script-setup.md) for more details.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup>
    ///   export let msg = 'Hello!'
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup>
    ///   let msg = 'Hello!'
    /// </script>
    /// ```
    NoExportInScriptSetup,
    vue,
    correctness,
);

impl Rule for NoExportInScriptSetup {
    fn run_once(&self, ctx: &LintContext) {
        let modules = ctx.module_record();

        for entry in &modules.local_export_entries {
            if entry.is_type {
                continue;
            }

            ctx.diagnostic(no_export_in_script_setup_diagnostic(entry.span));
        }

        for entry in &modules.indirect_export_entries {
            if entry.is_type {
                continue;
            }

            ctx.diagnostic(no_export_in_script_setup_diagnostic(entry.span));
        }

        for entry in &modules.star_export_entries {
            if entry.is_type {
                continue;
            }
            ctx.diagnostic(no_export_in_script_setup_diagnostic(entry.span));
        }

        if let Some(span) = modules.export_default {
            ctx.diagnostic(no_export_in_script_setup_diagnostic(span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks_options() == FrameworkOptions::VueSetup
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
			      <script>
			      export * from 'foo'
			      export default {}
			      export class A {}
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export * from 'foo'
			      export default {}
			      export class A {}
			      </script>
			      <script setup>
			      let foo;
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
			      <script setup>
			      export * from 'foo'
			      export default {}
			      export class A {}
			      export const test = '123'
			      export function foo() {}
			      const a = 1
			      export { a }
			      export { fao } from 'bar'
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      let foo;
			      </script>
			      <script setup>
			      export * from 'foo'
			      export default {}
			      export class A {}
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      export const Foo = {}
			      export enum Bar {}
			      export { }
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parser": require("vue-eslint-parser"),        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
    ];

    Tester::new(NoExportInScriptSetup::NAME, NoExportInScriptSetup::PLUGIN, pass, fail)
        .test_and_snapshot();
}
