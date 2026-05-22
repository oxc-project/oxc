use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, AwaitExpression, BindingPattern, CallExpression, ChainElement,
        ExportDefaultDeclarationKind, Expression, ExpressionStatement, Function, ObjectExpression,
        ObjectPropertyKind, Program, Statement,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeFlags, Scoping, SymbolId};
use oxc_span::Span;

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn no_expose_after_await_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}` is forbidden after an `await` expression."))
        .with_help(
            "`expose` should be called synchronously in `setup()` \
            (or `defineExpose()` in `<script setup>`). Move the call before the first `await`.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExposeAfterAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow asynchronously registered `expose`.
    ///
    /// ### Why is this bad?
    ///
    /// `defineExpose` and `context.expose()` registered after an `await`
    /// expression in `<script setup>` or `setup()` may not work as expected
    /// because they are registered after the component instance has finished
    /// setting up.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup>
    /// await doSomething()
    /// defineExpose({ /* ... */ }) // error
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup>
    /// defineExpose({ /* ... */ }) // ok
    /// await doSomething()
    /// </script>
    /// ```
    NoExposeAfterAwait,
    vue,
    correctness,
    version = "next",
);

impl Rule for NoExposeAfterAwait {
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
            // <script setup>
            AstKind::Program(program) if ctx.frameworks_options() == FrameworkOptions::VueSetup => {
                check_script_setup(program, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
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

    let (params, body) = match &setup_prop.value {
        Expression::FunctionExpression(func) => (&func.params, func.body.as_ref()),
        Expression::ArrowFunctionExpression(arrow) => (&arrow.params, Some(&arrow.body)),
        _ => return,
    };
    let Some(body) = body else {
        return;
    };

    let Some(second_param) = params.items.get(1) else {
        return;
    };

    let binding = match &second_param.pattern {
        // `setup(_, {expose})` — destructured `expose`
        BindingPattern::ObjectPattern(obj_pat) => obj_pat.properties.iter().find_map(|prop| {
            let key_name = prop.key.static_name()?;
            if key_name != "expose" {
                return None;
            }
            let binding_id = prop.value.get_binding_identifier()?;
            binding_id.symbol_id.get().map(ExposeBinding::Expose)
        }),
        // `setup(_, ctx)` — whole context bound to a name
        BindingPattern::BindingIdentifier(binding_id) => {
            binding_id.symbol_id.get().map(ExposeBinding::Ctx)
        }
        _ => None,
    };

    let Some(binding) = binding else {
        return;
    };

    let scoping = ctx.scoping();
    let mut visitor = ExposeAfterAwaitVisitor::new(scoping, binding);
    visitor.visit_function_body(body);

    visitor.errors.iter().for_each(|(span, name)| {
        ctx.diagnostic(no_expose_after_await_diagnostic(*span, name));
    });
}

fn check_script_setup<'a>(program: &Program<'a>, ctx: &LintContext<'a>) {
    let mut after_await = false;
    for stmt in &program.body {
        if after_await
            && let Statement::ExpressionStatement(expr_stmt) = stmt
            && let Some(call_expr) = extract_call_expression(&expr_stmt.expression)
            && let Some(ident) = call_expr.callee.get_inner_expression().get_identifier_reference()
            && ident.name == "defineExpose"
        {
            ctx.diagnostic(no_expose_after_await_diagnostic(call_expr.span, "defineExpose"));
        }

        if !after_await {
            let mut detector = AwaitDetector::default();
            detector.visit_statement(stmt);
            if detector.found {
                after_await = true;
            }
        }
    }
}

fn extract_call_expression<'a, 'b>(expr: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expr.get_inner_expression() {
        Expression::CallExpression(c) => Some(c),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(c) => Some(c),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Default)]
struct AwaitDetector {
    found: bool,
}

impl<'a> Visit<'a> for AwaitDetector {
    fn visit_await_expression(&mut self, _: &AwaitExpression) {
        self.found = true;
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}

enum ExposeBinding {
    /// `setup(_, {expose})` — detect bare `expose(...)` calls
    Expose(SymbolId),
    /// `setup(_, ctx)` — detect `ctx.expose(...)` member calls
    Ctx(SymbolId),
}

struct ExposeAfterAwaitVisitor<'a> {
    found: bool,
    errors: Vec<(Span, String)>,
    scoping: &'a Scoping,
    binding: ExposeBinding,
}

impl<'a> ExposeAfterAwaitVisitor<'a> {
    fn new(scoping: &'a Scoping, binding: ExposeBinding) -> Self {
        Self { found: false, errors: Vec::new(), scoping, binding }
    }

    fn matches_target(&self, call_expr: &CallExpression<'a>) -> bool {
        let callee = call_expr.callee.get_inner_expression();

        match self.binding {
            ExposeBinding::Expose(expose_id) => {
                let Some(ident) = callee.get_identifier_reference() else {
                    return false;
                };
                let reference = self.scoping.get_reference(ident.reference_id());
                reference.symbol_id() == Some(expose_id)
            }
            ExposeBinding::Ctx(ctx_id) => {
                let Expression::StaticMemberExpression(member) = callee else {
                    return false;
                };
                if member.property.name != "expose" {
                    return false;
                }
                let Some(obj_ident) =
                    member.object.get_inner_expression().get_identifier_reference()
                else {
                    return false;
                };
                let reference = self.scoping.get_reference(obj_ident.reference_id());
                reference.symbol_id() == Some(ctx_id)
            }
        }
    }
}

impl<'a> Visit<'a> for ExposeAfterAwaitVisitor<'a> {
    fn visit_await_expression(&mut self, _expr: &AwaitExpression) {
        self.found = true;
    }

    // Only `ExpressionStatement` direct children are reported. Stop handles such
    // as `var a = expose()`, `c = expose()`, `d(expose())`, `{ foo: expose() }`,
    // `[expose()]` are wrapped in another expression and are intentionally ignored.
    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        if !self.found {
            walk::walk_expression_statement(self, stmt);
            return;
        }

        if let Some(call_expr) = extract_call_expression(&stmt.expression)
            && self.matches_target(call_expr)
        {
            self.errors.push((call_expr.span, "expose".to_string()));
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
                  export default {
                    async setup(_, {expose}) {
                      expose({ /* ... */ }) // ok
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
                  export default {
                    async setup(_, ctx) {
                      ctx.expose({ /* ... */ }) // ok
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
                  export default {
                    async setup(_, {expose}) {
                      expose({ /* ... */ })
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
                  export default {
                    async setup(_, ctx) {
                      ctx.expose({ /* ... */ })
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
                  export default {
                    async _setup(_, {expose}) {
                      expose({ /* ... */ })
                      await doSomething()
                      expose({ /* ... */ })
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
                  export default {
                    async _setup(_, ctx) {
                      ctx.expose({ /* ... */ })
                      await doSomething()
                      ctx.expose({ /* ... */ })
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
                  defineExpose({ /* ... */ })
                  await doSomething()
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  await doSomething()
                  {
                    defineExpose({ /* ... */ })
                  }
                  function defineExpose() {}
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  import { onMounted } from 'vue';
                  await doSomething()
                  onMounted(() => {})
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
                  export default {
                    async setup(_, {expose}) {
                      await doSomething()
                      expose({ /* ... */ })
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
                  export default {
                    async setup(_, ctx) {
                      await doSomething()
                      ctx.expose({ /* ... */ })
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
                  await doSomething()
                  defineExpose({ /* ... */ })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoExposeAfterAwait::NAME, NoExposeAfterAwait::PLUGIN, pass, fail)
        .test_and_snapshot();
}
