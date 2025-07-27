use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, ImportDeclarationSpecifier, ImportOrExportKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{FrameworkOptions, context::LintContext, rule::Rule};

fn has_type_and_arguments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineEmits` has both a type-only emit and an argument.")
        .with_help("remove the argument for better type inference.")
        .with_label(span)
}

fn called_multiple_times(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineEmits` has been called multiple times.")
        .with_help("combine all events into a single `defineEmits` call.")
        .with_label(span)
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

#[derive(Debug, Default, Clone)]
pub struct ValidDefineEmits;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks whether defineEmits compiler macro is valid.
    ///
    /// This rule reports defineEmits compiler macros in the following cases:
    ///
    /// - `defineEmits` is referencing locally declared variables.
    /// - `defineEmits` has both a literal type and an argument. e.g. defineEmits<(e: 'foo')=>void>(['bar'])
    /// - `defineEmits` has been called multiple times.
    ///
    /// ::: danger NOTE
    ///
    /// The rules does not support analyzing `export default {}` from the second `<script>` tag.
    /// It only supports `<script setup>` tag.
    ///
    /// :::
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
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
    ValidDefineEmits,
    vue,
    nursery, // TODO: change category to `correctness`, when cross `<script>` analysis is implemented. See out-commented tests
    pending  // TODO: removing empty `defineEmits` and merging multiple `defineEmits` calls
);

impl Rule for ValidDefineEmits {
    fn run_once(&self, ctx: &LintContext) {
        let mut found = false;
        let mut imported_names = vec![];

        for node in ctx.nodes() {
            match node.kind() {
                AstKind::ImportDeclaration(import_decl) => {
                    if import_decl.import_kind == ImportOrExportKind::Type {
                        continue; // Skip type imports
                    }

                    let Some(specifiers) = &import_decl.specifiers else {
                        continue; // Skip if this is `import 'module'`
                    };

                    for specifier in specifiers {
                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                                imported_names.push(spec.local.name);
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                                imported_names.push(spec.local.name);
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                                imported_names.push(spec.local.name);
                            }
                        }
                    }
                }
                AstKind::CallExpression(call_expr) => {
                    // only check call Expression which is `defineEmits`
                    if call_expr
                        .callee
                        .get_identifier_reference()
                        .is_none_or(|reference| reference.name != "defineEmits")
                    {
                        continue;
                    }

                    if found {
                        ctx.diagnostic(called_multiple_times(call_expr.span));
                        continue;
                    }

                    found = true;

                    handle_call_expression(call_expr, &imported_names, ctx);
                }
                _ => {}
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.frameworks_options() == FrameworkOptions::VueSetup
    }
}

fn handle_call_expression(
    call_expr: &CallExpression,
    imported_names: &Vec<Atom>,
    ctx: &LintContext,
) {
    let has_type_args =
        call_expr.type_arguments.as_ref().is_some_and(|args| !args.params.is_empty());

    // `defineEmits` has type arguments and js arguments. Vue Compiler allows only one of them.
    if has_type_args && !call_expr.arguments.is_empty() {
        ctx.diagnostic(has_type_and_arguments_diagnostic(call_expr.span));
        return; // Skip if there are type arguments
    }

    if has_type_args {
        // If there are type arguments, we don't need to check the arguments.
        return;
    }

    if call_expr.arguments.is_empty() {
        ctx.diagnostic(events_not_defined(call_expr.span));
        return;
    }

    let Some(expression) = call_expr.arguments.first().unwrap().as_expression() else {
        ctx.diagnostic(events_not_defined(call_expr.span));
        return;
    };

    match expression {
        Expression::ArrayExpression(array) => {
            if array.elements.is_empty() {
                ctx.diagnostic(events_not_defined(call_expr.span));
            }
        }
        Expression::ObjectExpression(object) => {
            if object.properties.is_empty() {
                ctx.diagnostic(events_not_defined(call_expr.span));
            }
        }
        Expression::Identifier(identifier) => {
            if !imported_names.contains(&identifier.name) {
                ctx.diagnostic(referencing_locally(call_expr.span));
            }
        }
        _ => {
            ctx.diagnostic(referencing_locally(call_expr.span));
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
        // currently not supported, because we need to know the other script
        // (
        //     "
        // 	      <script>
        // 	        const def = { notify: null }
        // 	      </script>
        // 	      <script setup>
        // 	        /* ✓ GOOD */
        // 	        defineEmits(def)
        // 	      </script>
        // 	      ",
        //     None,
        //     None,
        //     Some(PathBuf::from("test.vue")),
        // ),
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
        // currently not supported, because we need to know the other script
        // (
        //     "
        // 	      <script>
        // 	      export default {
        // 	        emits: ['notify']
        // 	      }
        // 	      </script>
        // 	      <script setup>
        // 	        /* ✗ BAD */
        // 	        defineEmits({ submit: null })
        // 	      </script>
        // 	      ",
        //     None,
        //     None,
        //     Some(PathBuf::from("test.vue")),
        // ),
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
