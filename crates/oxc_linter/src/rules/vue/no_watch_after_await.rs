use rustc_hash::FxHashMap;

use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, AwaitExpression, ChainElement, ExportDefaultDeclarationKind,
        Expression, ExpressionStatement, Function, ObjectExpression,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeFlags, Scoping, SymbolId};
use oxc_span::Span;

use crate::module_record::{ImportEntry, ImportImportName};
use crate::{
    AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule, utils::find_property,
};

fn no_watch_after_await_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}` is forbidden after an `await` expression."))
        .with_help("`watch` and `watchEffect` should be called synchronously in `setup()`. Move the call before the first `await`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoWatchAfterAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow asynchronously registered `watch`.
    ///
    /// ### Why is this bad?
    ///
    /// `watch` and `watchEffect` registered after an `await` expression in
    /// `setup()` may not work as expected because they are registered after
    /// the component instance has finished setting up.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// import { watch } from 'vue'
    /// export default {
    ///   async setup() {
    ///     await doSomething()
    ///     watch(foo, () => { /* ... */ }) // error
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// import { watch } from 'vue'
    /// export default {
    ///   async setup() {
    ///     watch(foo, () => { /* ... */ }) // ok
    ///     await doSomething()
    ///   }
    /// }
    /// </script>
    /// ```
    NoWatchAfterAwait,
    vue,
    correctness,
    version = "1.67.0",
);

const WATCH_FUNCTIONS: &[&str] = &["watch", "watchEffect"];

impl Rule for NoWatchAfterAwait {
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
    let Some(setup_prop) = find_property(obj_expr, "setup") else {
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
    let mut symbol_to_import_entry: FxHashMap<SymbolId, &ImportEntry> = FxHashMap::default();

    for import_entry in &module_record.import_entries {
        if import_entry.module_request.name() != "vue" {
            continue;
        }
        let import_name = match &import_entry.import_name {
            ImportImportName::Name(name_span) => name_span.name(),
            _ => continue,
        };

        if !WATCH_FUNCTIONS.contains(&import_name) {
            continue;
        }
        if let Some(symbol_id) = scoping.get_root_binding(import_entry.local_name.name().into()) {
            symbol_to_import_entry.insert(symbol_id, import_entry);
        }
    }

    let mut visitor = WatchAfterAwaitVisitor::new(scoping, symbol_to_import_entry);
    visitor.visit_function_body(function_body);

    visitor.errors.iter().for_each(|(span, name)| {
        ctx.diagnostic(no_watch_after_await_diagnostic(*span, name));
    });
}

struct WatchAfterAwaitVisitor<'a> {
    found: bool,
    errors: Vec<(Span, String)>,
    scoping: &'a Scoping,
    symbol_to_import_entry: FxHashMap<SymbolId, &'a ImportEntry>,
}

impl<'a> WatchAfterAwaitVisitor<'a> {
    fn new(
        scoping: &'a Scoping,
        symbol_to_import_entry: FxHashMap<SymbolId, &'a ImportEntry>,
    ) -> Self {
        Self { found: false, errors: Vec::new(), scoping, symbol_to_import_entry }
    }
}

impl<'a> Visit<'a> for WatchAfterAwaitVisitor<'a> {
    fn visit_await_expression(&mut self, _expr: &AwaitExpression) {
        self.found = true;
    }

    // Only `ExpressionStatement` direct children are reported. Stop handles such
    // as `var a = watch()`, `c = watch()`, `d(watch())`, `{ foo: watch() }`,
    // `[watch()]` are wrapped in another expression and are intentionally ignored.
    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        if !self.found {
            walk::walk_expression_statement(self, stmt);
            return;
        }

        let inner = stmt.expression.get_inner_expression();
        let call_expr = match inner {
            Expression::CallExpression(c) => c,
            Expression::ChainExpression(chain) => {
                let ChainElement::CallExpression(c) = &chain.expression else {
                    walk::walk_expression_statement(self, stmt);
                    return;
                };
                c
            }
            _ => {
                walk::walk_expression_statement(self, stmt);
                return;
            }
        };

        let Some(ident) = call_expr.callee.get_inner_expression().get_identifier_reference() else {
            walk::walk_expression_statement(self, stmt);
            return;
        };

        let reference = self.scoping.get_reference(ident.reference_id());
        let Some(symbol_id) = reference.symbol_id() else {
            walk::walk_expression_statement(self, stmt);
            return;
        };

        if let Some(import_entry) = self.symbol_to_import_entry.get(&symbol_id)
            && let ImportImportName::Name(name_span) = &import_entry.import_name
        {
            self.errors.push((call_expr.span, name_span.name().to_string()));
        }
        walk::walk_expression_statement(self, stmt);
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _func: &ArrowFunctionExpression<'a>) {}
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      watch(foo, () => { /* ... */ }) // ok
            
                      await doSomething()
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      watch(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {watch, watchEffect} from 'vue'
                  export default {
                    async setup() {
                      watchEffect(() => { /* ... */ })
                      watch(foo, () => { /* ... */ })
                      await doSomething()
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {onMounted} from 'vue'
                  export default {
                    async _setup() {
                      await doSomething()
            
                      onMounted(() => { /* ... */ }) // error
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {watch, watchEffect} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
                      const a = watchEffect(() => { /* ... */ })
                      const b = watch(foo, () => { /* ... */ })
                      c = watch()
                      d(watch())
                      e = {
                        foo: watch()
                      }
                      f = [watch()]
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  Vue.component('test', {
                    el: foo()
                  })
                ",
            None,
            None,
            None,
        ),
        (
            "
                  <script>
                  import {watch, watchEffect} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
                      const a = watchEffect?.(() => { /* ... */ })
                      const b = watch?.(foo, () => { /* ... */ })
                      c = watch?.()
                      d(watch?.())
                      e = {
                        foo: watch?.()
                      }
                      f = [watch?.()]
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import {watchEffect} from 'vue'
                  watchEffect(() => { /* ... */ })
                  await doSomething()
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 2022 },
        (
            "
                  <script setup>
                  await doSomething()
                  </script>
                  <script>
                  import {watchEffect} from 'vue'
                  watchEffect(() => { /* ... */ }) // not error
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 2022 },
        (
            "
                  <script setup>
                  </script>
                  <script>
                  import {watchEffect} from 'vue'
                  await doSomething()
                  watchEffect(() => { /* ... */ }) // not error
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 2022 },
        (
            "
                  <script setup>
                  import {watch} from 'vue'
                  watch(foo, () => { /* ... */ })

                  await doSomething()

                  watch(foo, () => { /* ... */ })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 2022 }
        // a function defined after `await` is fine
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
                      function logMessage() {
                        watch(foo, () => { /* ... */ })
                      }
                    }
                  }
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
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
            
                      watch(foo, () => { /* ... */ }) // error
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {watch, watchEffect} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
            
                      watchEffect(() => { /* ... */ })
                      watch(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()

                      watch(foo, () => { /* ... */ })

                      await doSomething()

                      watch(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // parenthesized callee
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      await doSomething();
                      (watch)(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `export default defineComponent({...})`
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default defineComponent({
                    async setup() {
                      await doSomething()
                      watch(foo, () => { /* ... */ })
                    }
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `defineComponent({...})` without `export default`
        (
            "
                  <script>
                  import {watch} from 'vue'
                  defineComponent({
                    async setup() {
                      await doSomething()
                      watch(foo, () => { /* ... */ })
                    }
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `setup: async () => {...}`
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    setup: async () => {
                      await doSomething()
                      watch(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // optional chain at expression statement (not stop handle)
        (
            "
                  <script>
                  import {watch} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
                      watch?.(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // import alias
        (
            "
                  <script>
                  import {watch as A} from 'vue'
                  export default {
                    async setup() {
                      await doSomething()
                      A(foo, () => { /* ... */ })
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoWatchAfterAwait::NAME, NoWatchAfterAwait::PLUGIN, pass, fail).test_and_snapshot();
}
