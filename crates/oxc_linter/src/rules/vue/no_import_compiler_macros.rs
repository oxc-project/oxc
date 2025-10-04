use oxc_ast::{
    AstKind,
    ast::{ImportDeclarationSpecifier, ModuleExportName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_import_compiler_macros_diagnostic(span: Span, name: &Atom) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{name}' is a compiler macro and doesn't need to be imported."))
        .with_help("Remove the import statement for this macro.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImportCompilerMacros;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoImportCompilerMacros,
    vue,
    restriction,
    fix
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
            ctx.diagnostic_with_fix(
                no_import_compiler_macros_diagnostic(import_specifier.span, &imported_name.name),
                |fixer| {
                    if specifiers.len() == 1 {
                        fixer.delete(import_decl)
                    } else if index == 0 {
                        let part_source = ctx.source_range(Span::new(
                            import_specifier.span.end,
                            import_decl.span.end,
                        ));
                        let next_comma_index = part_source.find(',').unwrap_or_default();
                        fixer.delete_range(Span::new(
                            import_specifier.span.start,
                            import_specifier.span.end + next_comma_index as u32 + 1,
                        ))
                    } else {
                        let part_source = ctx.source_range(Span::new(
                            import_decl.span.start,
                            import_specifier.span.start,
                        ));
                        let last_comma_index = part_source.rfind(',').unwrap_or_default();
                        fixer.delete_range(Span::new(
                            import_decl.span.start + last_comma_index as u32,
                            import_specifier.span.end,
                        ))
                    }
                },
            )
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
    ];

    let fix = vec![
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
            r#"
			      import { ref as refFoo, defineSlots as defineSlotsFoo, type computed } from '@vue/runtime-core'
			      "#,
            r#"
			      import { ref as refFoo, type computed } from '@vue/runtime-core'
			      "#,
            None,
        ),
    ];
    Tester::new(NoImportCompilerMacros::NAME, NoImportCompilerMacros::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
