use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::Rule,
};

fn missing_default_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing default export.").with_label(span)
}

fn must_be_default_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Component must be the default export.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireDefaultExport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require components to be the default export.
    ///
    /// ### Why is this bad?
    ///
    /// Using SFCs (Single File Components) without a default export is
    /// not supported in Vue 3. Components should be exported as the default export.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// const foo = 'foo';
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   data() {
    ///     return {
    ///       foo: 'foo'
    ///     };
    ///   }
    /// };
    /// </script>
    /// ```
    RequireDefaultExport,
    vue,
    suspicious,
);

impl Rule for RequireDefaultExport {
    fn run_once(&self, ctx: &LintContext) {
        let has_define_component = ctx.nodes().iter().any(|node| {
            let AstKind::CallExpression(call_expr) = node.kind() else {
                return false;
            };

            match call_expr.callee.get_inner_expression() {
                Expression::Identifier(identifier) => identifier.name == "defineComponent",
                Expression::StaticMemberExpression(member_expr) => {
                    let Expression::Identifier(object_identifier) =
                        member_expr.object.get_inner_expression()
                    else {
                        return false;
                    };

                    object_identifier.name == "Vue" && member_expr.property.name == "component"
                }
                _ => false,
            }
        });

        #[expect(clippy::cast_possible_truncation)]
        let span = Span::sized(
            ctx.source_text().len() as u32,
            9, // `</script>` length
        );

        if has_define_component {
            ctx.diagnostic(must_be_default_export_diagnostic(span));
        } else {
            ctx.diagnostic(missing_default_export_diagnostic(span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        // only on vue files
        if ctx.file_extension().is_none_or(|ext| ext != "vue") {
            return false;
        }

        // only with `<script>`, not `<script setup>`
        if ctx.frameworks_options() == FrameworkOptions::VueSetup {
            return false;
        }

        // only when no default export is present
        if ctx.module_record().export_default.is_some() {
            return false;
        }

        // only when no `<script setup>` is present in the current file
        !ctx.other_file_hosts()
            .iter()
            .any(|host| host.framework_options() == FrameworkOptions::VueSetup)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
			      <template>Without script</template>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			        import { ref } from 'vue';

			        export default {}
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			        const foo = 'foo';
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      const component = {};

			      export default component;
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      import {defineComponent} from 'vue';

			      export default defineComponent({});
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      const foo = 'foo';
			      export const bar = 'bar';
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
			      import {defineComponent} from 'vue';
			      defineComponent({});
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    let fail = vec![
        (
            "
			      <script>
			      const foo = 'foo';
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export const foo = 'foo';
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      const foo = 'foo';

			      export { foo };
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export const foo = 'foo';
			      export const bar = 'bar';
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      import { defineComponent } from 'vue';

			      export const component = defineComponent({});
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      import Vue from 'vue';

			      const component = Vue.component('foo', {});
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(RequireDefaultExport::NAME, RequireDefaultExport::PLUGIN, pass, fail)
        .test_and_snapshot();
}
