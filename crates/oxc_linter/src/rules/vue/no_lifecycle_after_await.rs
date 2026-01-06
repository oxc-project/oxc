use oxc_ast::{
    AstKind,
    ast::{AwaitExpression, CallExpression, Expression, Function},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn no_lifecycle_after_await_diagnostic(span: Span, hook_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Lifecycle hook `{hook_name}` is called after `await` in `setup()`."
    ))
    .with_help("Lifecycle hooks should be called synchronously in `setup()`. Move the hook call before the first `await`.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLifecycleAfterAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow asynchronously registered lifecycle hooks.
    ///
    /// ### Why is this bad?
    ///
    /// Lifecycle hooks must be registered synchronously during `setup()` execution.
    /// If a lifecycle hook is called after an `await` statement, it may be registered
    /// too late and might not work as expected.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// import { onMounted } from 'vue'
    /// export default {
    ///   async setup() {
    ///     await doSomething()
    ///     onMounted(() => { /* ... */ }) // error
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// import { onMounted } from 'vue'
    /// export default {
    ///   async setup() {
    ///     onMounted(() => { /* ... */ }) // ok
    ///     await doSomething()
    ///   }
    /// }
    /// </script>
    /// ```
    NoLifecycleAfterAwait,
    vue,
    correctness
);

const LIFECYCLE_HOOKS: &[&str] = &[
    "onBeforeMount",
    "onBeforeUnmount",
    "onBeforeUpdate",
    "onErrorCaptured",
    "onMounted",
    "onRenderTracked",
    "onRenderTriggered",
    "onUnmounted",
    "onUpdated",
    "onActivated",
    "onDeactivated",
];

impl Rule for NoLifecycleAfterAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectProperty(obj_prop) = node.kind() else {
            return;
        };

        // Check if this is a setup method
        if obj_prop.key.static_name().as_deref() != Some("setup") {
            return;
        }

        let function_body_opt = match &obj_prop.value {
            Expression::FunctionExpression(func_expr) => func_expr.body.as_ref(),
            Expression::ArrowFunctionExpression(arrow_func_expr) => Some(&arrow_func_expr.body),
            _ => None,
        };
        let Some(function_body) = function_body_opt else {
            return;
        };

        let mut visitor = LifecycleAfterAwaitVisitor::new();
        visitor.visit_function_body(function_body);

        visitor.errors.iter().for_each(|(span, hook_name)| {
            ctx.diagnostic(no_lifecycle_after_await_diagnostic(*span, hook_name));
        });
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
            && ctx.frameworks_options() != FrameworkOptions::VueSetup
    }
}

struct LifecycleAfterAwaitVisitor {
    found: bool,
    errors: Vec<(Span, String)>,
}

impl LifecycleAfterAwaitVisitor {
    fn new() -> Self {
        Self { found: false, errors: Vec::new() }
    }
}

impl<'a> Visit<'a> for LifecycleAfterAwaitVisitor {
    fn visit_await_expression(&mut self, _expr: &AwaitExpression) {
        if !self.found {
            self.found = true;
        }
    }

    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if !self.found {
            walk::walk_call_expression(self, call_expr);
            return;
        }

        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            walk::walk_call_expression(self, call_expr);
            return;
        };

        let hook_name = ident.name.as_str();

        if LIFECYCLE_HOOKS.contains(&hook_name) {
            self.errors.push((call_expr.span, hook_name.to_string()));
        }
        walk::walk_call_expression(self, call_expr);
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
			          onMounted(() => { /* ... */ }) // ok

			          await doSomething()
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                  (
                    "
                        <script>
                        import {onMounted} from 'vue'
                        export default {
                            async setup() {
                                await doSomething();
                                function foo() {
                                    onMounted(() => { /* ... */ }) // ok
                                }
                            }
                        }
                        </script>
                    ",
                    None,
                    None,
                    Some(PathBuf::from("test.vue"))
                  ),
                  (
                    "
                        <script>
                        import {onMounted} from 'vue'
                        export default {
                            async setup() {
                                async function bar() {
                                    await doSomething();
                                }
                                bar();
                                onMounted(() => { /* ... */ }) // ok
                            }
                        }
                        </script>
                    ",
                    None,
                    None,
                    Some(PathBuf::from("test.vue"))
                  ),
("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
			          onMounted(() => { /* ... */ })
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
("
			      <script>
			      import {onBeforeMount, onBeforeUnmount, onBeforeUpdate, onErrorCaptured, onMounted, onRenderTracked, onRenderTriggered, onUnmounted, onUpdated, onActivated, onDeactivated} from 'vue'
			      export default {
			        async setup() {
			          onBeforeMount(() => { /* ... */ })
			          onBeforeUnmount(() => { /* ... */ })
			          onBeforeUpdate(() => { /* ... */ })
			          onErrorCaptured(() => { /* ... */ })
			          onMounted(() => { /* ... */ })
			          onRenderTracked(() => { /* ... */ })
			          onRenderTriggered(() => { /* ... */ })
			          onUnmounted(() => { /* ... */ })
			          onUpdated(() => { /* ... */ })
			          onActivated(() => { /* ... */ })
			          onDeactivated(() => { /* ... */ })

			          await doSomething()
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async _setup() {
			          await doSomething()

			          onMounted(() => { /* ... */ }) // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
("
			      <script setup>
			      import {onMounted} from 'vue'
			      onMounted(() => { /* ... */ })
			      await doSomething()
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))), // { "ecmaVersion": 2022 },
("
			      <script setup>
			      await doSomething()
			      </script>
			      <script>
			      import {onMounted} from 'vue'
			      onMounted(() => { /* ... */ }) // not error
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))), // { "ecmaVersion": 2022 },
("
			      <script setup>
			      </script>
			      <script>
			      import {onMounted} from 'vue'
			      await doSomething()
			      onMounted(() => { /* ... */ }) // not error
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))), // { "ecmaVersion": 2022 },
("
			      <script setup>
			      import {onMounted} from 'vue'
			      await doSomething()

			      onMounted(() => { /* ... */ }) // not error
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))), // { "ecmaVersion": 2022 },
("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          function logMessage() {
			            onMounted(() => {
			              console.log('Component has been mounted')
			            })
			          }
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue")))
    ];

    let fail = vec![
        ("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          onMounted(() => { /* ... */ }) // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                  ("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        setup: async () => {
			          await doSomething()

			          onMounted(() => { /* ... */ }) // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
("
			      <script>
			      import {onBeforeMount, onBeforeUnmount, onBeforeUpdate, onErrorCaptured, onMounted, onRenderTracked, onRenderTriggered, onUnmounted, onUpdated, onActivated, onDeactivated} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          onBeforeMount(() => { /* ... */ })
			          onBeforeUnmount(() => { /* ... */ })
			          onBeforeUpdate(() => { /* ... */ })
			          onErrorCaptured(() => { /* ... */ })
			          onMounted(() => { /* ... */ })
			          onRenderTracked(() => { /* ... */ })
			          onRenderTriggered(() => { /* ... */ })
			          onUnmounted(() => { /* ... */ })
			          onUpdated(() => { /* ... */ })
			          onActivated(() => { /* ... */ })
			          onDeactivated(() => { /* ... */ })
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          onMounted?.(() => { /* ... */ }) // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
("
			      <script>
			      import {onBeforeMount, onBeforeUnmount, onBeforeUpdate, onErrorCaptured, onMounted, onRenderTracked, onRenderTriggered, onUnmounted, onUpdated, onActivated, onDeactivated} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          onBeforeMount(() => { /* ... */ }, instance)
			          onBeforeUnmount(() => { /* ... */ }, instance)
			          onBeforeUpdate(() => { /* ... */ }, instance)
			          onErrorCaptured(() => { /* ... */ }, instance)
			          onMounted(() => { /* ... */ }, instance)
			          onRenderTracked(() => { /* ... */ }, instance)
			          onRenderTriggered(() => { /* ... */ }, instance)
			          onUnmounted(() => { /* ... */ }, instance)
			          onUpdated(() => { /* ... */ }, instance)
			          onActivated(() => { /* ... */ }, instance)
			          onDeactivated(() => { /* ... */ }, instance)
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue")))
    ];

    Tester::new(NoLifecycleAfterAwait::NAME, NoLifecycleAfterAwait::PLUGIN, pass, fail)
        .test_and_snapshot();
}
