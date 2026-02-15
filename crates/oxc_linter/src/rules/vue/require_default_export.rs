use oxc_ast::{
    AstKind,
    ast::{Expression, JSXChild},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{get_script_statements_span, get_vue_sfc_struct},
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
        let Some(script_statements_span) = get_script_statements_span(ctx) else {
            return;
        };

        let mut has_define_component = false;
        for node in ctx.nodes() {
            if let AstKind::ExportDefaultDeclaration(_) = node.kind() {
                return;
            }

            let AstKind::CallExpression(call_expr) = node.kind() else {
                continue;
            };

            if match call_expr.callee.get_inner_expression() {
                Expression::Identifier(identifier) => identifier.name == "defineComponent",
                Expression::StaticMemberExpression(member_expr) => {
                    let Expression::Identifier(object_identifier) =
                        member_expr.object.get_inner_expression()
                    else {
                        continue;
                    };

                    object_identifier.name == "Vue" && member_expr.property.name == "component"
                }
                _ => continue,
            } {
                has_define_component = true;
            }
        }

        let span = get_vue_sfc_struct(ctx)
            .iter()
            .find(|child| child.span().contains_inclusive(script_statements_span))
            .map(|child| {
                let JSXChild::Element(element) = child else {
                    unreachable!();
                };
                element.closing_element.as_ref().unwrap().span
            })
            .unwrap();

        if has_define_component {
            ctx.diagnostic(must_be_default_export_diagnostic(span));
        } else {
            ctx.diagnostic(missing_default_export_diagnostic(span));
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        // only on vue files
        ctx.frameworks().is_vue()
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
