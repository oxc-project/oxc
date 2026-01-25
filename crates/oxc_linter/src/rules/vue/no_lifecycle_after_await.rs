use oxc_ast::{
    AstKind,
    ast::{
        AwaitExpression, CallExpression, ExportDefaultDeclarationKind, Expression, Function,
        ObjectExpression, ObjectPropertyKind,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeFlags, Scoping, SymbolId};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::module_record::{ImportEntry, ImportImportName};
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
        match node.kind() {
            // e.g. `export default { setup() {} }`
            AstKind::ExportDefaultDeclaration(export_decl) => {
                if let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) =
                    &export_decl.declaration
                {
                    check_setup_in_object(obj_expr, ctx);
                }
            }
            // e.g. `defineComponent({ setup() {} })`
            AstKind::CallExpression(call_expr) => {
                if let Some(ident) = call_expr.callee.get_identifier_reference()
                    && ident.name == "defineComponent"
                    && let Some(first_arg) = call_expr.arguments.first()
                    && let Some(Expression::ObjectExpression(obj_expr)) = first_arg.as_expression()
                {
                    check_setup_in_object(obj_expr, ctx);
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
            && ctx.frameworks_options() != FrameworkOptions::VueSetup
    }
}

fn check_setup_in_object<'a>(obj_expr: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
    let Some(setup_prop) = obj_expr.properties.iter().find_map(|prop| {
        let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
            return None;
        };
        obj_prop.key.static_name().is_some_and(|name| name == "setup").then_some(obj_prop)
    }) else {
        return;
    };

    let function_body_opt = match &setup_prop.value {
        Expression::FunctionExpression(func_expr) => func_expr.body.as_ref(),
        Expression::ArrowFunctionExpression(arrow_func_expr) => Some(&arrow_func_expr.body),
        _ => None,
    };
    let Some(function_body) = function_body_opt else {
        return;
    };

    let module_record = ctx.module_record();
    let scoping = ctx.scoping();
    // map the `symbol_id` to the `import_entry`
    // e.g `import { onMounted } from 'vue'; onMounted();` -> `symbol_id: import_entry`
    // so we can find the `import_entry` by the `symbol_id` later
    let mut symbol_to_import_entry: FxHashMap<SymbolId, &ImportEntry> = FxHashMap::default();

    for import_entry in &module_record.import_entries {
        if import_entry.module_request.name() != "vue" {
            continue;
        }
        let import_name = match &import_entry.import_name {
            ImportImportName::Name(name_span) => name_span.name(),
            _ => continue,
        };

        // e.g `import { onMounted as A } from 'vue'; A();`
        if !LIFECYCLE_HOOKS.contains(&import_name) {
            continue;
        }
        if let Some(symbol_id) = scoping.get_root_binding(import_entry.local_name.name()) {
            symbol_to_import_entry.insert(symbol_id, import_entry);
        }
    }

    let mut visitor = LifecycleAfterAwaitVisitor::new(scoping, symbol_to_import_entry);
    visitor.visit_function_body(function_body);

    visitor.errors.iter().for_each(|(span, hook_name)| {
        ctx.diagnostic(no_lifecycle_after_await_diagnostic(*span, hook_name));
    });
}

struct LifecycleAfterAwaitVisitor<'a> {
    found: bool,
    errors: Vec<(Span, String)>,
    scoping: &'a Scoping,
    symbol_to_import_entry: FxHashMap<SymbolId, &'a ImportEntry>,
}

impl<'a> LifecycleAfterAwaitVisitor<'a> {
    fn new(
        scoping: &'a Scoping,
        symbol_to_import_entry: FxHashMap<SymbolId, &'a ImportEntry>,
    ) -> Self {
        Self { found: false, errors: Vec::new(), scoping, symbol_to_import_entry }
    }
}

impl<'a> Visit<'a> for LifecycleAfterAwaitVisitor<'a> {
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

        if call_expr.arguments.len() >= 2 {
            walk::walk_call_expression(self, call_expr);
            return;
        }

        let Some(ident) = call_expr.callee.get_inner_expression().get_identifier_reference() else {
            walk::walk_call_expression(self, call_expr);
            return;
        };

        let reference = self.scoping.get_reference(ident.reference_id());
        let Some(symbol_id) = reference.symbol_id() else {
            walk::walk_call_expression(self, call_expr);
            return;
        };

        if let Some(import_entry) = self.symbol_to_import_entry.get(&symbol_id)
            && let ImportImportName::Name(name_span) = &import_entry.import_name
        {
            self.errors.push((call_expr.span, name_span.name().to_string()));
        }
        walk::walk_call_expression(self, call_expr);
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(
        &mut self,
        _func: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
    }
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
                      ("
			      <script>
			      import {onMounted} from 'vue'
                  let a = {
                    async setup() {
                      await doSomething()
                      onMounted(() => { /* ... */ }) // ok
                    }
                  }
			      export default a;
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                      ("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
                        await doSomething()
                        onUpdated(() => { /* ... */ }) // ok
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                      ("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
                      function onMounted(callback) {
                        return;
                      }
                      await doSomething()
			          onMounted(() => { /* ... */ }) // ok
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
                  import { h } from 'vue'
			      export default {
			        async setup() {
                      await doSomething();
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
			      ", None, None, Some(PathBuf::from("test.vue"))),
                      ("
			      <script>
			      import {onMounted} from 'vue'
			      let a = {
			        async setup() {
			          await d();
			          onMounted(() => {});
			        }
			      }
			      export default a;
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
			      ", None, None, Some(PathBuf::from("test.vue"))),
                  // https://github.com/oxc-project/oxc/issues/18298
                      ("
			      <script>
                  import { onMounted } from 'vue';
                  export default {
                    async setup() {
                        const doNothing = async () => {
                            await doSomething();
                        };

                        onMounted(() => {
                        /* ... */
                        }); // error
                    },
                  };
                  </script>
			      ", None, None, Some(PathBuf::from("test.vue")))
        ];

    let fail = vec![
        ("
			      <script>
			      import {onMounted} from 'vue'
			      export default {
			        async setup() {
			          await doSomething();

			          (onMounted)(() => { /* ... */ }) // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                  ("
			      <script>
			      import {onMounted} from 'vue'
			      export default defineComponent({
                    name: 'Index',
                    async setup() {
                        await doSomething();
                        onMounted(() => {});
                    },
                    });
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                  ("
			      <script>
			      import {onMounted} from 'vue'
			      defineComponent({
                    name: 'Index',
                    async setup() {
                        await doSomething();
                        onMounted(() => {});
                    },
                    });
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
			      import {onMounted as A} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          A?.(() => { /* ... */ }) // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
                  ("
			      <script>
			      import {onMounted as A} from 'vue'
			      export default {
			        async setup() {
			          await doSomething()

			          A() // error
			        }
			      }
			      </script>
			      ", None, None, Some(PathBuf::from("test.vue"))),
    ];

    Tester::new(NoLifecycleAfterAwait::NAME, NoLifecycleAfterAwait::PLUGIN, pass, fail)
        .test_and_snapshot();
}
