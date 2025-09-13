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
    OxcDiagnostic::warn("`defineEmits` has both a type-only emit and an argument.")
        .with_help("remove the argument for better type inference.")
        .with_label(span)
}

fn called_multiple_times(span: Span, second_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineEmits` has been called multiple times.")
        .with_help("combine all events into a single `defineEmits` call.")
        .with_labels([
            span.label("`defineEmits` is called here"),
            second_span.label("`defineEmits` is called here too"),
        ])
}

fn events_not_defined(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Custom events are not defined.")
        .with_help("Define at least one event in `defineEmits`.")
        .with_label(span)
}

fn referencing_locally(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineEmits` is referencing locally declared variables.")
        .with_help("inline the variable or import it from another module.")
        .with_label(span)
}

fn define_in_both(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Custom events are defined in both `defineEmits` and `export default {}`.")
        .with_help("Remove `export default`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidDefineEmits;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks whether defineEmits compiler macro is valid.
    ///
    /// This rule reports defineEmits compiler macros in the following cases:
    ///
    /// - `defineEmits` is referencing locally declared variables.
    /// - `defineEmits` has both a literal type and an argument. e.g. `defineEmits<(e: 'foo')=>void>(['bar'])`
    /// - `defineEmits` has been called multiple times.
    /// - Custom events are defined in both `defineEmits` and `export default {}`.
    /// - Custom events are not defined in either `defineEmits` or `export default {}`.
    ///
    /// ### Why is this bad?
    ///
    /// Misusing `defineEmits` can lead to runtime errors, unclear component contracts, and lost type safety.
    /// Vue may still compile the code, but emitted events may break silently or be typed incorrectly.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```vue
    /// <script setup>
    /// const def = { notify: null }
    /// defineEmits(def)
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup lang="ts">
    /// defineEmits<(e: 'notify') => void>({ submit: null })
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup>
    /// defineEmits({ notify: null })
    /// defineEmits({ submit: null })
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script>
    /// export default {
    ///   emits: ['notify']
    /// }
    /// </script>
    /// <script setup>
    /// defineEmits({ submit: null })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```vue
    /// <script setup>
    /// defineEmits({ notify: null })
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup>
    /// defineEmits(['notify'])
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script setup lang="ts">
    /// defineEmits<(e: 'notify') => void>()
    /// </script>
    /// ```
    ///
    /// ```vue
    /// <script>
    /// export default {
    ///   emits: ['notify']
    /// }
    /// </script>
    /// <script setup>
    /// defineEmits()
    /// </script>
    /// ```
    ValidDefineEmits,
    vue,
    correctness,
    pending  // TODO: removing empty `defineEmits` and merging multiple `defineEmits` calls
);

impl Rule for ValidDefineEmits {
    fn run_once(&self, ctx: &LintContext) {
        let mut found: Option<Span> = None;

        let has_other_script_emits = has_default_exports_property(&ctx.other_file_hosts(), "emits");
        for node in ctx.nodes() {
            let AstKind::CallExpression(call_expr) = node.kind() else {
                continue;
            };

            // only check call Expression which is `defineEmits`
            if call_expr
                .callee
                .get_identifier_reference()
                .is_none_or(|reference| reference.name != "defineEmits")
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
			        defineEmits({ notify: null })
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
			        defineEmits(['notify'])
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
			        defineEmits<(e: 'notify')=>void>()
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      },
        (
            "
        	      <script>
        	        const def = { notify: null }
        	      </script>
        	      <script setup>
        	        /* ✓ GOOD */
        	        defineEmits(def)
        	      </script>
        	      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			        defineEmits({
			          notify (payload) {
			            return typeof payload === 'string'
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
        (
            "
                  <script>
                  export default { emits: [] }
                  </script>
                  <script setup>
                  defineEmits();
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			      defineEmits(unResolvedVariable);
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
			        const def = { notify: null }
			        defineEmits(def)
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
			        defineEmits<(e: 'notify')=>void>({ submit: null })
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
			        defineEmits({ notify: null })
			        defineEmits({ submit: null })
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
        	        emits: ['notify']
        	      }
        	      </script>
        	      <script setup>
        	        /* ✗ BAD */
        	        defineEmits({ submit: null })
        	      </script>
        	      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  export default { emits: [] }
                  </script>
                  <script setup lang='ts'>
                    defineEmits<{
                      (e: 'change'): void
                    }>();
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			        /* ✗ BAD */
			        defineEmits()
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(ValidDefineEmits::NAME, ValidDefineEmits::PLUGIN, pass, fail).test_and_snapshot();
}
