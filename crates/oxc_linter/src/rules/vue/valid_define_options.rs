use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierReference, ObjectPropertyKind, TSType},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn referencing_locally_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineOptions` is referencing locally declared variables.")
        .with_label(span)
}

fn multiple_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineOptions` has been called multiple times.").with_label(span)
}

fn not_defined_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Options are not defined.").with_label(span)
}

fn disallow_prop_diagnostic(span: Span, prop_name: &str, instead_macro: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`defineOptions()` cannot be used to declare `{prop_name}`. Use `{instead_macro}()` instead."
    ))
    .with_label(span)
}

fn type_args_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`defineOptions()` cannot accept type arguments.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidDefineOptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce valid `defineOptions` compiler macro.
    ///
    /// ### Why is this bad?
    ///
    /// `defineOptions` is a compiler macro for `<script setup>`. It must be called
    /// with a single object literal containing component options that are evaluable
    /// at compile time. Misuse such as referencing locally declared variables,
    /// declaring `props`/`emits`/`expose`/`slots`, calling without arguments, or
    /// passing type arguments cannot be processed by the compiler.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup>
    /// defineOptions()                         // no options object
    /// defineOptions({ name: 'A' })
    /// defineOptions({ name: 'B' })            // multiple calls
    /// defineOptions({ props: { msg: String } }) // use `defineProps()` instead
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup>
    /// defineOptions({ name: 'foo', inheritAttrs: false })
    /// </script>
    /// ```
    ValidDefineOptions,
    vue,
    correctness,
    version = "next",
);

const DISALLOWED_PROPS: &[(&str, &str)] = &[
    ("props", "defineProps"),
    ("emits", "defineEmits"),
    ("expose", "defineExpose"),
    ("slots", "defineSlots"),
];

impl Rule for ValidDefineOptions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Program(program) = node.kind() else {
            return;
        };

        let mut visitor = DefineOptionsChecker::new(ctx);
        visitor.visit_program(program);

        if visitor.call_spans.len() > 1 {
            for span in &visitor.call_spans {
                ctx.diagnostic(multiple_diagnostic(*span));
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
            && ctx.frameworks_options() == FrameworkOptions::VueSetup
    }
}

struct DefineOptionsChecker<'a, 'b> {
    ctx: &'b LintContext<'a>,
    call_spans: Vec<Span>,
}

impl<'a> DefineOptionsChecker<'a, '_> {
    fn new<'b>(ctx: &'b LintContext<'a>) -> DefineOptionsChecker<'a, 'b> {
        DefineOptionsChecker { ctx, call_spans: Vec::new() }
    }

    fn check_call(&mut self, call: &CallExpression<'a>) {
        self.call_spans.push(call.span);

        if let Some(type_args) = &call.type_arguments {
            self.ctx.diagnostic(type_args_diagnostic(type_args.span));
        }

        let Some(first_arg_expr) = call.arguments.first().and_then(|arg| arg.as_expression())
        else {
            self.ctx.diagnostic(not_defined_diagnostic(call.span));
            return;
        };

        if let Expression::ObjectExpression(obj) = first_arg_expr {
            for prop in &obj.properties {
                let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
                    continue;
                };
                let Some(name) = obj_prop.key.static_name() else { continue };
                for &(prop_name, instead_macro) in DISALLOWED_PROPS {
                    if name == prop_name {
                        self.ctx.diagnostic(disallow_prop_diagnostic(
                            call.span,
                            prop_name,
                            instead_macro,
                        ));
                        break;
                    }
                }
            }
        }

        let mut local_checker = LocalReferenceChecker::new(self.ctx);
        local_checker.visit_expression(first_arg_expr);
        for span in local_checker.errors {
            self.ctx.diagnostic(referencing_locally_diagnostic(span));
        }
    }
}

impl<'a> Visit<'a> for DefineOptionsChecker<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(ident) = call.callee.get_identifier_reference()
            && ident.name == "defineOptions"
        {
            self.check_call(call);
        }
        walk::walk_call_expression(self, call);
    }
}

struct LocalReferenceChecker<'a, 'b> {
    ctx: &'b LintContext<'a>,
    errors: Vec<Span>,
}

impl<'a> LocalReferenceChecker<'a, '_> {
    fn new<'b>(ctx: &'b LintContext<'a>) -> LocalReferenceChecker<'a, 'b> {
        LocalReferenceChecker { ctx, errors: Vec::new() }
    }
}

impl<'a> Visit<'a> for LocalReferenceChecker<'a, '_> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if !is_non_local_reference(ident, self.ctx) {
            self.errors.push(ident.span);
        }
    }

    // skip TS type nodes — `as X`, `typeof str` 等の型部分は静的に解決可能
    fn visit_ts_type(&mut self, _ty: &TSType<'a>) {}
}

fn is_non_local_reference(ident: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    let Some(symbol_id) = ctx.semantic().scoping().get_root_binding(ident.name) else {
        // unresolved (e.g. defined in a sibling `<script>` block) → treat as non-local
        return true;
    };
    let decl = ctx.semantic().symbol_declaration(symbol_id);
    match decl.kind() {
        AstKind::ImportSpecifier(_)
        | AstKind::ImportDefaultSpecifier(_)
        | AstKind::ImportNamespaceSpecifier(_) => true,
        AstKind::VariableDeclarator(declarator) => {
            // `const x = <literal>` is statically resolvable, so allowed
            declarator.kind.is_const()
                && declarator.init.as_ref().is_some_and(is_literal_expression)
        }
        _ => false,
    }
}

fn is_literal_expression(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                  <script setup>
                  defineOptions({ name: 'foo' })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script>
                  const def = { name: 'foo' }
                  </script>
                  <script setup>
                  defineOptions(def)
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  type X = string;
            
                  defineOptions({ name: 'foo' as X })
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            r#"
                  <script setup lang="ts">
                  const str = 'abc'
            
                  defineOptions({ name: 'foo' as (typeof str) })
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            "
                  <script setup>
                  import { def } from './defs';
            
                  defineOptions(def);
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  const def = 'foo';
                  defineOptions({ name: def });
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
                    const def = { name: 'Foo' }
                    defineOptions(def)
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                    defineOptions({ name: 'Foo' })
                    defineOptions({ name: 'Bar' })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                    defineOptions()
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  defineOptions<{ name: 'Foo' }>()
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            "
                  <script setup>
                    defineOptions({ props: { msg: String } })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                    defineOptions({ emits: ['click'] })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                    defineOptions({ expose: ['foo'] })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                    defineOptions({ slots: Object })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // multiple disallowed props at once → 2 separate diagnostics
        (
            "
                  <script setup>
                    defineOptions({ props: { msg: String }, emits: ['click'] })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(ValidDefineOptions::NAME, ValidDefineOptions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
