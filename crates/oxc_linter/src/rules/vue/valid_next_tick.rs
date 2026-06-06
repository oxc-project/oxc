use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_in_vue_component_instance_method, is_this_object, is_vue_next_tick_import},
};

fn should_be_function_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`nextTick` is a function.").with_label(span)
}

fn missing_callback_or_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Await the Promise returned by `nextTick` or pass a callback function.")
        .with_label(span)
}

fn too_many_parameters_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`nextTick` expects zero or one parameters.").with_label(span)
}

fn either_await_or_callback_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Either await the Promise or pass a callback function to `nextTick`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ValidNextTick;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce valid `nextTick` function calls.
    ///
    /// ### Why is this bad?
    ///
    /// `nextTick` is a function that takes either a callback or returns a Promise.
    /// Misuse (accessing it as a value, passing extra arguments, both awaiting and
    /// passing a callback) is almost always a bug.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// import { nextTick } from 'vue'
    /// export default {
    ///   async mounted() {
    ///     nextTick()                     // missing await or callback
    ///     this.$nextTick                 // not invoked
    ///     this.$nextTick(a, b)           // too many args
    ///     await this.$nextTick(callback) // both await and callback
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// import { nextTick } from 'vue'
    /// export default {
    ///   async mounted() {
    ///     await nextTick()
    ///     this.$nextTick(callback)
    ///     this.$nextTick().then(callback)
    ///   }
    /// }
    /// </script>
    /// ```
    ValidNextTick,
    vue,
    correctness,
    fix,
    version = "1.67.0",
);

impl Rule for ValidNextTick {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (next_tick_node, report_span) = match node.kind() {
            AstKind::StaticMemberExpression(m) => {
                let prop_name = m.property.name.as_str();
                let object = &m.object;
                let matches = match prop_name {
                    "$nextTick" => is_this_object(object, ctx),
                    "nextTick" => {
                        matches!(object.get_inner_expression(), Expression::Identifier(id) if id.name == "Vue")
                    }
                    _ => false,
                };
                if !matches {
                    return;
                }
                (node, m.property.span)
            }
            AstKind::IdentifierReference(ident) => {
                if !is_vue_next_tick_import(ident, ctx) {
                    return;
                }
                (node, ident.span)
            }
            _ => return,
        };

        if !is_in_vue_component_instance_method(next_tick_node, ctx) {
            return;
        }

        // Skip `ConditionalExpression`: `bar ? nt : undefined`.
        let mut parent = ctx.nodes().parent_node(next_tick_node.id());
        if matches!(parent.kind(), AstKind::ConditionalExpression(_)) {
            parent = ctx.nodes().parent_node(parent.id());
        }

        match parent.kind() {
            AstKind::CallExpression(call) if call.callee.span() == next_tick_node.kind().span() => {
                check_call(parent, call, report_span, ctx);
            }
            // OK: `foo.then(nt)` (passed as value) / `let foo = nt` / `foo = nt`
            AstKind::CallExpression(_)
            | AstKind::VariableDeclarator(_)
            | AstKind::AssignmentExpression(_) => {}
            _ => {
                let end = next_tick_node.kind().span().end;
                ctx.diagnostic_with_fix(should_be_function_diagnostic(report_span), |fixer| {
                    fixer.insert_text_after_range(Span::new(end, end), "()")
                });
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

fn check_call<'a>(
    call_node: &AstNode<'a>,
    call: &CallExpression<'a>,
    report_span: Span,
    ctx: &LintContext<'a>,
) {
    let args_len = call.arguments.len();
    let awaited = is_awaited_promise(call_node, ctx);

    if args_len == 0 {
        if !awaited {
            ctx.diagnostic(missing_callback_or_await_diagnostic(report_span));
        }
        return;
    }
    if args_len > 1 {
        ctx.diagnostic(too_many_parameters_diagnostic(report_span));
        return;
    }
    if awaited {
        ctx.diagnostic(either_await_or_callback_diagnostic(report_span));
    }
}

fn is_awaited_promise(call_node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let parent = ctx.nodes().parent_node(call_node.id());
    match parent.kind() {
        AstKind::AwaitExpression(_)
        | AstKind::ReturnStatement(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::AssignmentExpression(_) => true,
        // `() => nextTick()` (expression body arrow). The call's direct parent is
        // an `ExpressionStatement` whose grandparent is the arrow's `FunctionBody`.
        AstKind::ExpressionStatement(_) => {
            let gp = ctx.nodes().parent_node(parent.id());
            if !matches!(gp.kind(), AstKind::FunctionBody(_)) {
                return false;
            }
            let ggp = ctx.nodes().parent_node(gp.id());
            matches!(ggp.kind(), AstKind::ArrowFunctionExpression(arrow) if arrow.expression)
        }
        // `nextTick().then(...)`
        AstKind::StaticMemberExpression(m) => m.property.name == "then",
        // `Promise.all([nextTick()])`
        AstKind::ArrayExpression(_) => {
            let grandparent = ctx.nodes().parent_node(parent.id());
            if let AstKind::CallExpression(c) = grandparent.kind()
                && let Some(member) = c.callee.get_member_expr()
                && let Expression::Identifier(obj) = member.object().get_inner_expression()
                && obj.name == "Promise"
            {
                return true;
            }
            false
        }
        _ => false,
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        ("", None, None, Some(PathBuf::from("test.vue"))),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    async mounted() {
                      await nt();
                      await Vue.nextTick();
                      await this.$nextTick();

                      nt().then(callback);
                      Vue.nextTick().then(callback);
                      this.$nextTick().then(callback);

                      nt(callback);
                      Vue.nextTick(callback);
                      this.$nextTick(callback);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      foo.then(nt);
                      foo.then(Vue.nextTick);
                      foo.then(this.$nextTick);

                      foo.then(nt, catchHandler);
                      foo.then(Vue.nextTick, catchHandler);
                      foo.then(this.$nextTick, catchHandler);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      let foo = nt;
                      foo = Vue.nextTick;
                      foo = this.$nextTick;
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      Promise.all([nt(), someOtherPromise]);
                      Promise.all([Vue.nextTick(), someOtherPromise]);
                      Promise.all([this.$nextTick(), someOtherPromise]);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    created() {
                      let queue = nt();
                      queue = queue.then(nt);
                      return nt();
                    },
                    mounted() {
                      const queue = Vue.nextTick();
                      return Vue.nextTick();
                    },
                    updated() {
                      const queue = this.$nextTick();
                      return this.$nextTick();
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>;
                  export default {
                    methods: {
                      fn1 () {
                        return this.$nextTick()
                      },
                      fn2 () {
                        return this.$nextTick()
                          .then(() => this.$nextTick())
                      },
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      let foo = bar ? nt : undefined;
                      foo = bar ? Vue.nextTick : undefined;
                      foo = bar ? this.$nextTick : undefined;
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Vue 2 legacy: detected via the rule, but properly used = pass
        (
            "<script>
                  new Vue({
                    async mounted() {
                      await this.$nextTick();
                      this.$nextTick(callback);
                    }
                  })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  Vue.extend({
                    async mounted() {
                      await this.$nextTick();
                      this.$nextTick(callback);
                    }
                  })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default {
                    mounted() {
                      const { vm } = this
                      vm.$nextTick()
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default {
                    mounted() {
                      let vm = this
                      vm = other
                      vm.$nextTick()
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    async mounted() {
                      nt();
                      Vue.nextTick();
                      this.$nextTick();
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      nt;
                      Vue.nextTick;
                      this.$nextTick;

                      nt.then(callback);
                      Vue.nextTick.then(callback);
                      this.$nextTick.then(callback);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    async mounted() {
                      await nt;
                      await Vue.nextTick;
                      return this.$nextTick;
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      Promise.all([nt, someOtherPromise]);
                      Promise.all([Vue.nextTick, someOtherPromise]);
                      Promise.all([this.$nextTick, someOtherPromise]);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      nt(callback, anotherCallback);
                      Vue.nextTick(callback, anotherCallback);
                      this.$nextTick(callback, anotherCallback);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    async mounted() {
                      nt(callback).then(anotherCallback);
                      Vue.nextTick(callback).then(anotherCallback);
                      this.$nextTick(callback).then(anotherCallback);

                      await nt(callback);
                      await Vue.nextTick(callback);
                      await this.$nextTick(callback);
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // Vue 2 legacy: misused inside `new Vue({...})` / `Vue.extend({...})`
        (
            "<script>
                  new Vue({
                    mounted() {
                      this.$nextTick;
                    }
                  })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  Vue.extend({
                    mounted() {
                      this.$nextTick;
                    }
                  })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  Vue.mixin({
                    mounted() {
                      this.$nextTick;
                    }
                  })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
                  defineNuxtComponent({
                    mounted() {
                      this.$nextTick;
                    }
                  })</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default {
                    mounted() {
                      const vm = this
                      vm.$nextTick()
                    }
                  }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix = vec![
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      nt;
                      Vue.nextTick;
                      this.$nextTick;

                      nt.then(callback);
                      Vue.nextTick.then(callback);
                      this.$nextTick.then(callback);
                    }
                  }</script>",
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      nt();
                      Vue.nextTick();
                      this.$nextTick();

                      nt().then(callback);
                      Vue.nextTick().then(callback);
                      this.$nextTick().then(callback);
                    }
                  }</script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    async mounted() {
                      await nt;
                      await Vue.nextTick;
                      return this.$nextTick;
                    }
                  }</script>",
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    async mounted() {
                      await nt();
                      await Vue.nextTick();
                      return this.$nextTick();
                    }
                  }</script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      Promise.all([nt, someOtherPromise]);
                      Promise.all([Vue.nextTick, someOtherPromise]);
                      Promise.all([this.$nextTick, someOtherPromise]);
                    }
                  }</script>",
            "<script>import { nextTick as nt } from 'vue';
                  export default {
                    mounted() {
                      Promise.all([nt(), someOtherPromise]);
                      Promise.all([Vue.nextTick(), someOtherPromise]);
                      Promise.all([this.$nextTick(), someOtherPromise]);
                    }
                  }</script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(ValidNextTick::NAME, ValidNextTick::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
