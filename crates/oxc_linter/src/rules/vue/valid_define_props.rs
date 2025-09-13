use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::Rule,
    utils::{DefineMacroProblem, check_define_macro_call_expression, has_default_exports_property},
};

fn has_type_and_arguments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineProps` has both a type-only emit and an argument.")
        .with_help("remove the argument for better type inference.")
        .with_label(span)
}

fn called_multiple_times(span: Span, second_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineProps` has been called multiple times.")
        .with_help("combine all `defineProps` calls into a single `defineProps` call.")
        .with_labels([
            span.label("`defineProps` is called here"),
            second_span.label("`defineProps` is called here too"),
        ])
}

fn events_not_defined(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Props are not defined.")
        .with_help("Define at least one prop in `defineProps`.")
        .with_label(span)
}

fn referencing_locally(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineProps` is referencing locally declared variables.")
        .with_help("inline the variable or import it from another module.")
        .with_label(span)
}

fn define_in_both(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Props are defined in both `defineProps` and `export default {}`.")
        .with_help("Remove `export default`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidDefineProps;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks whether `defineProps` compiler macro is valid.
    ///
    /// This rule reports `defineProps` compiler macros in the following cases:
    ///
    /// - `defineProps` is referencing locally declared variables.
    /// - `defineProps` has both a literal type and an argument. e.g. `defineProps<{ /*props*/ }>({ /*props*/ })`
    /// - `defineProps` has been called multiple times.
    /// - Props are defined in both `defineProps` and `export default {}`.
    /// - Props are not defined in either `defineProps` or `export default {}`.
    ///
    /// ### Why is this bad?
    ///
    /// Misusing `defineProps` can lead to runtime errors, and lost type safety.
    /// Vue may still compile the code, but properties may break silently or be typed incorrectly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```vue
    /// <script setup>
    /// const def = { msg: String }
    /// defineProps(def)
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup lang="ts">
    /// defineProps<{ msg?: string }>({ msg: String })
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup>
    /// defineProps({ msg: String })
    /// defineProps({ count: Number })
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script>
    /// export default {
    ///   props: { msg: String }
    /// }
    /// </script>
    /// <script setup>
    /// defineProps({ count: Number })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```vue
    /// <script setup>
    /// defineProps({ msg: String })
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup>
    /// defineProps(['msg'])
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup lang="ts">
    /// defineProps<{ msg?: string }>()
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script>
    /// export default {
    ///   props: { msg: String }
    /// }
    /// </script>
    /// <script setup>
    /// defineProps()
    /// </script>
    /// ```
    ValidDefineProps,
    vue,
    correctness,
    pending  // TODO: removing empty `defineProps` and merging multiple `defineProps` calls
);

impl Rule for ValidDefineProps {
    fn run_once(&self, ctx: &LintContext) {
        let mut found: Option<Span> = None;

        let has_other_script_emits = has_default_exports_property(&ctx.other_file_hosts(), "props");
        for node in ctx.nodes() {
            let AstKind::CallExpression(call_expr) = node.kind() else {
                continue;
            };

            // only check call Expression which is `defineProps`
            if call_expr
                .callee
                .get_identifier_reference()
                .is_none_or(|reference| reference.name != "defineProps")
            {
                continue;
            }

            if let Some(other_span) = found {
                ctx.diagnostic(called_multiple_times(call_expr.span, other_span));
                continue;
            }
            found = Some(call_expr.span);

            let Some(problem) =
                check_define_macro_call_expression(call_expr, ctx, has_other_script_emits)
            else {
                continue;
            };

            let diagnostic = match problem {
                DefineMacroProblem::DefineInBoth => define_in_both(call_expr.span),
                DefineMacroProblem::HasTypeAndArguments => {
                    has_type_and_arguments_diagnostic(call_expr.span)
                }
                DefineMacroProblem::EventsNotDefined => events_not_defined(call_expr.span),
                DefineMacroProblem::ReferencingLocally => referencing_locally(call_expr.span),
            };
            ctx.diagnostic(diagnostic);
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
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
			      <script setup>
			        /* ✓ GOOD */
			        defineProps({ msg: String })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			        /* ✓ GOOD */
			        defineProps(['msg'])
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			        /* ✓ GOOD */
			        defineProps<{ msg?:string }>()
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      },
        (
            "
			      <script>
			        const def = { msg: String }
			      </script>
			      <script setup>
			        /* ✓ GOOD */
			        defineProps(def)
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			        defineProps({
			          addFunction: {
			            type: Function,
			            default (a, b) {
			              return a + b
			            }
			          }
			        })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      import type { PropType } from 'vue';

			      type X = string;

			      const props = defineProps({
			        myProp: Array as PropType<string[]>,
			      });

			      const emit = defineEmits({
			        myProp: (x: X) => true,
			      });
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			      <script setup lang="ts">
			      import type { PropType } from 'vue';

			      const strList = ['a', 'b', 'c']
			      const str = 'abc'

			      const props = defineProps({
			        myProp: Array as PropType<typeof strList>,
			      });

			      const emit = defineEmits({
			        myProp: (x: typeof str) => true,
			      });
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            "
			      <script setup>
			      import { propsDef, emitsDef } from './defs';

			      defineProps(propsDef);
			      defineEmits(emitsDef);
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
			      <script setup>
			        /* ✗ BAD */
			        const def = { msg: String }
			        defineProps(def)
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			        /* ✗ BAD */
			        defineProps<{ msg?:string }>({ msg: String })
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      },
        (
            "
			      <script setup>
			        /* ✗ BAD */
			        defineProps({ msg: String })
			        defineProps({ count: Number })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        props: { msg: String }
			      }
			      </script>
			      <script setup>
			        /* ✗ BAD */
			        defineProps({ count: Number })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			        /* ✗ BAD */
			        defineProps()
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(ValidDefineProps::NAME, ValidDefineProps::PLUGIN, pass, fail).test_and_snapshot();
}
