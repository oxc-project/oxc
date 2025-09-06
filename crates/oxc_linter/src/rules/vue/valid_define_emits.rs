use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclarationKind, Expression, IdentifierReference,
        ObjectPropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ContextSubHost, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

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

        let has_other_script_emits = has_default_emits_exports(&ctx.other_file_hosts());
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

            handle_call_expression(call_expr, ctx, has_other_script_emits);
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.frameworks_options() == FrameworkOptions::VueSetup
    }
}

fn handle_call_expression(
    call_expr: &CallExpression,
    ctx: &LintContext,
    has_other_script_emits: bool,
) {
    let has_type_args = call_expr.type_arguments.is_some();

    if has_type_args && has_other_script_emits {
        ctx.diagnostic(define_in_both(call_expr.span));
        return;
    }

    // `defineEmits` has type arguments and js arguments. Vue Compiler allows only one of them.
    if has_type_args && !call_expr.arguments.is_empty() {
        ctx.diagnostic(has_type_and_arguments_diagnostic(call_expr.span));
        return; // Skip if there are type arguments
    }

    if has_type_args {
        // If there are type arguments, we don't need to check the arguments.
        return;
    }

    let Some(expression) = call_expr.arguments.first().and_then(|first| first.as_expression())
    else {
        // `defineEmits();` is valid when `export default { emits: [] }` is defined
        if !has_other_script_emits {
            ctx.diagnostic(events_not_defined(call_expr.span));
        }
        return;
    };

    if has_other_script_emits {
        ctx.diagnostic(define_in_both(call_expr.span));
        return;
    }

    match expression {
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => {}
        Expression::Identifier(identifier) => {
            if !is_non_local_reference(identifier, ctx) {
                ctx.diagnostic(referencing_locally(call_expr.span));
            }
        }
        _ => {
            ctx.diagnostic(referencing_locally(call_expr.span));
        }
    }
}

pub fn is_non_local_reference(identifier: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    if let Some(symbol_id) = ctx.semantic().scoping().get_root_binding(&identifier.name) {
        return matches!(
            ctx.semantic().symbol_declaration(symbol_id).kind(),
            AstKind::ImportSpecifier(_)
        );
    }

    // variables outside the current `<script>` block are valid.
    // This is the same for unresolved variables.
    true
}

fn has_default_emits_exports(others: &Vec<&ContextSubHost<'_>>) -> bool {
    for host in others {
        for other_node in host.semantic().nodes() {
            let AstKind::ExportDefaultDeclaration(export) = other_node.kind() else {
                continue;
            };

            let ExportDefaultDeclarationKind::ObjectExpression(export_obj) = &export.declaration
            else {
                continue;
            };

            let has_emits_exports = export_obj.properties.iter().any(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return false;
                };

                property.key.name().is_some_and(|name| name == "emits")
            });

            if has_emits_exports {
                return true;
            }
        }
    }

    false
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
