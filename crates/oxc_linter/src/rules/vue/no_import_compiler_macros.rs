use oxc_ast::{
    AstKind,
    ast::{ImportDeclarationSpecifier, ModuleExportName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn no_import_compiler_macros_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is a compiler macro and doesn't need to be imported."))
        .with_help("Remove the import statement for this macro.")
        .with_label(span)
}

fn invalid_import_compiler_macros_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{name}' is a compiler macro and can't be imported outside of `<script setup>`."
    ))
    .with_help("Remove the import statement for this macro.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportCompilerMacros;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow importing Vue compiler macros.
    ///
    /// ### Why is this bad?
    ///
    /// Compiler Macros like:
    /// - `defineProps`
    /// - `defineEmits`
    /// - `defineExpose`
    /// - `withDefaults`
    /// - `defineModel`
    /// - `defineOptions`
    /// - `defineSlots`
    ///
    /// are globally available in Vue 3's `<script setup>` and do not require explicit imports.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup>
    /// import { defineProps, withDefaults } from 'vue'
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup>
    /// import { ref } from 'vue'
    /// </script>
    /// ```
    NoImportCompilerMacros,
    vue,
    restriction,
    dangerous_fix,
    version = "1.21.0",
    short_description = "Disallow importing Vue compiler macros.",
);

const COMPILER_MACROS: &[&str; 7] = &[
    "defineProps",
    "defineEmits",
    "defineExpose",
    "withDefaults",
    "defineModel",
    "defineOptions",
    "defineSlots",
];

const VUE_MODULES: &[&str; 3] = &["vue", "@vue/runtime-core", "@vue/runtime-dom"];

impl Rule for NoImportCompilerMacros {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        let Some(specifiers) = &import_decl.specifiers else {
            return;
        };

        if !VUE_MODULES.contains(&import_decl.source.value.as_str()) {
            return;
        }

        for (index, specifier) in specifiers.iter().enumerate() {
            let ImportDeclarationSpecifier::ImportSpecifier(import_specifier) = &specifier else {
                continue;
            };

            let ModuleExportName::IdentifierName(imported_name) = &import_specifier.imported else {
                continue;
            };

            if !COMPILER_MACROS.contains(&imported_name.name.as_str()) {
                continue;
            }

            // named specifiers are always a contiguous suffix: `import def, { a, b }`
            let has_prev_named = index > 0
                && matches!(specifiers[index - 1], ImportDeclarationSpecifier::ImportSpecifier(_));
            let has_next_named = index + 1 < specifiers.len();

            let fixer = |fixer: crate::fixer::RuleFixer<'_, 'a>| {
                if specifiers.len() == 1 {
                    return fixer.delete(import_decl);
                }

                if !has_prev_named && has_next_named {
                    // take the specifier and the comma separating it from the next one
                    let next_start = specifiers[index + 1].span().start;
                    let Some(comma) =
                        ctx.find_next_token_within(import_specifier.span.end, next_start, ",")
                    else {
                        return fixer.noop();
                    };
                    return fixer.delete_range(Span::new(
                        import_specifier.span.start,
                        import_specifier.span.end + comma + 1,
                    ));
                }

                // take the comma separating it from the previous specifier, and the braces
                // too if this empties them: `import def, { defineProps }` -> `import def`
                let prev_end = specifiers[index - 1].span().end;
                let Some(comma) =
                    ctx.find_prev_token_within(prev_end, import_specifier.span.start, ",")
                else {
                    return fixer.noop();
                };
                let mut end = import_specifier.span.end;
                if !has_prev_named {
                    let Some(brace) = ctx.find_next_token_within(end, import_decl.span.end, "}")
                    else {
                        return fixer.noop();
                    };
                    end += brace + 1;
                }
                fixer.delete_range(Span::new(prev_end + comma, end))
            };

            if ctx.frameworks_options() == FrameworkOptions::VueSetup {
                // it is safe to removing the import inside `<script setup>`,
                // because the macro can be referenced globally.
                ctx.diagnostic_with_fix(
                    no_import_compiler_macros_diagnostic(
                        import_specifier.span,
                        &imported_name.name,
                    ),
                    fixer,
                );
            } else {
                // it is not safe to suggest removing the import,
                // because it can be referenced in the file.
                ctx.diagnostic_with_dangerous_fix(
                    invalid_import_compiler_macros_diagnostic(
                        import_specifier.span,
                        &imported_name.name,
                    ),
                    fixer,
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                  <script setup>
                  import { ref, computed } from 'vue'
                  import { someFunction } from '@vue/runtime-core'
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import { defineProps } from 'some-other-package'
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
                  import { defineProps } from 'vue'
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import {
                    ref,
                    defineProps
                  } from 'vue'
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import { ref, defineProps } from 'vue'
                  import { defineEmits, computed } from '@vue/runtime-core'
                  import { defineExpose, watch, withDefaults } from '@vue/runtime-dom'
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import { defineModel, defineOptions } from 'vue'
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  import { ref as refFoo, defineSlots as defineSlotsFoo, type computed } from '@vue/runtime-core'
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
        (r"import { defineProps } from 'vue'", None, None, None),
    ];

    let fix = vec![
        // a default import before the only named specifier: the braces go too
        ("import vue, { defineProps } from 'vue'", "import vue from 'vue'", None),
        ("import vue, { defineProps, ref } from 'vue'", "import vue, {  ref } from 'vue'", None),
        ("import vue, { ref, defineProps } from 'vue'", "import vue, { ref } from 'vue'", None),
        // a `,` inside a comment is not the separator between specifiers
        ("import { defineProps /* , */, ref } from 'vue'", "import {  ref } from 'vue'", None),
        ("import { ref, /* , */ defineProps } from 'vue'", "import { ref } from 'vue'", None),
        (
            "import { ref /* keep */, defineProps } from 'vue'",
            "import { ref /* keep */ } from 'vue'",
            None,
        ),
        ("import { defineProps } from 'vue'", "", None),
        (
            "
                  import {
                    ref,
                    defineProps
                  } from 'vue'
                  ",
            "
                  import {
                    ref
                  } from 'vue'
                  ",
            None,
        ),
        (
            "
                  import { ref, defineProps } from 'vue'
                  import { defineEmits, computed } from '@vue/runtime-core'
                  import { defineExpose, watch, withDefaults } from '@vue/runtime-dom'
                  ",
            "
                  import { ref } from 'vue'
                  import {  computed } from '@vue/runtime-core'
                  import {  watch } from '@vue/runtime-dom'
                  ",
            None,
        ),
        (
            "
                  import { defineModel, defineOptions } from 'vue'
                  ",
            "
                  import {  defineOptions } from 'vue'
                  ",
            None,
        ),
        (
            r"
                  import { ref as refFoo, defineSlots as defineSlotsFoo, type computed } from '@vue/runtime-core'
                  ",
            r"
                  import { ref as refFoo, type computed } from '@vue/runtime-core'
                  ",
            None,
        ),
    ];
    Tester::new(NoImportCompilerMacros::NAME, NoImportCompilerMacros::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
