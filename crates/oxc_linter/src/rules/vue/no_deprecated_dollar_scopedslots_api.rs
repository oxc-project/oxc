use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::get_declaration_from_reference_id,
    context::LintContext,
    rule::Rule,
    utils::{is_in_vue_component_instance_method, is_this_alias},
};

fn no_deprecated_dollar_scopedslots_api_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`$scopedSlots` is deprecated.")
        .with_help(
            "Use `$slots` instead; in Vue 3, all slots are exposed as functions on `$slots`.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedDollarScopedslotsApi;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated `$scopedSlots` (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// In Vue 3, the `$scopedSlots` instance property was removed. Scoped
    /// slots are unified with regular slots and exposed as functions on
    /// `$slots`, so accessing `this.$scopedSlots` no longer works.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render() {
    ///     return this.$scopedSlots.default('data')
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render() {
    ///     return this.$slots.default('data')
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedDollarScopedslotsApi,
    vue,
    correctness,
    suggestion,
    version = "next",
    short_description = "Disallow using deprecated `$scopedSlots` (in Vue.js 3.0.0+).",
);

impl Rule for NoDeprecatedDollarScopedslotsApi {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(member) = node.kind() else { return };
        if member.property.name != "$scopedSlots" {
            return;
        }

        // The component context is anchored at the `this` usage itself, or at
        // the `const vm = this` declaration when accessed through an alias, so
        // an alias captured by a nested function still reports.
        let in_component = match member.object.get_inner_expression() {
            Expression::ThisExpression(_) => is_in_vue_component_instance_method(node, ctx),
            Expression::Identifier(ident) => {
                is_this_alias(ident, ctx)
                    && get_declaration_from_reference_id(ident.reference_id(), ctx.semantic())
                        .is_some_and(|decl| is_in_vue_component_instance_method(decl, ctx))
            }
            _ => false,
        };

        if in_component {
            let span = member.property.span;
            ctx.diagnostic_with_suggestion(
                no_deprecated_dollar_scopedslots_api_diagnostic(span),
                |fixer| fixer.replace(span, "$slots"),
            );
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;
    // ref: https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/no-deprecated-dollar-scopedslots-api.test.ts

    let pass = vec![
        (
            r#"
                    <template>
                      <div v-bind="$attrs"/>
                    </template>
                    <script>
                    export default {
                      mounted () {
                        this.$emit('start')
                      }
                    }
                    </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      methods: {
                        click () {
                          this.$emit('click')
                        }
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
                    }
                    const another = function () {
                      console.log(this.$scopedSlots)
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                    <template>
                      <div foo="$scopedSlots"/>
                    </template>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                    <template>
                      <div v-on="() => {
                        function click ($scopedSlots) {
                          fn(foo.$scopedSlots)
                          fn($scopedSlots)
                        }
                      }"/>
                      <div v-for="$scopedSlots in list">
                        <div v-on="$scopedSlots">
                      </div>
                      <VueComp>
                        <template v-slot="{$scopedSlots}">
                          <div v-on="$scopedSlots">
                        </template>
                      </VueComp>
                    </template>
                    <script>
                    export default {
                      methods: {
                        click ($scopedSlots) {
                          foo.$scopedSlots
                        }
                      }
                    }
                    </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      computed: {
                        foo () {
                          const {vm} = this
                          return vm.$scopedSlots
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
            r#"
                    <template>
                      <div v-if="$scopedSlots.default"/>
                    </template>
                    <script>
                    export default {
                      render() {
                        return this.$scopedSlots.foo('bar')
                      }
                    }
                    </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                    <template>
                      <div v-for="slot in $scopedSlots"/>
                      <div :foo="$scopedSlots"/>
                    </template>
                    <script>
                    export default {
                      computed: {
                        foo () {
                          fn(this.$scopedSlots)
                        }
                      }
                    }
                    </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      render() {
                        const vm = this
                        return vm.$scopedSlots.foo('bar')
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
                      render() {
                        const vm = this
                        function fn() {
                          return vm.$scopedSlots
                        }
                        return fn().foo('bar')
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
                      render () {
                        const vm = this
                        const a = vm?.$scopedSlots
                        const b = this?.$scopedSlots
                        return a.foo('bar')
                      }
                    }
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix = vec![
        (
            r#"
                    <template>
                      <div v-if="$scopedSlots.default"/>
                    </template>
                    <script>
                    export default {
                      render() {
                        return this.$scopedSlots.foo('bar')
                      }
                    }
                    </script>
                  "#,
            // Template is not parsed, so only the script usage is fixed.
            r#"
                    <template>
                      <div v-if="$scopedSlots.default"/>
                    </template>
                    <script>
                    export default {
                      render() {
                        return this.$slots.foo('bar')
                      }
                    }
                    </script>
                  "#,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                    <template>
                      <div v-for="slot in $scopedSlots"/>
                      <div :foo="$scopedSlots"/>
                    </template>
                    <script>
                    export default {
                      computed: {
                        foo () {
                          fn(this.$scopedSlots)
                        }
                      }
                    }
                    </script>
                  "#,
            r#"
                    <template>
                      <div v-for="slot in $scopedSlots"/>
                      <div :foo="$scopedSlots"/>
                    </template>
                    <script>
                    export default {
                      computed: {
                        foo () {
                          fn(this.$slots)
                        }
                      }
                    }
                    </script>
                  "#,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      render() {
                        const vm = this
                        return vm.$scopedSlots.foo('bar')
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      render() {
                        const vm = this
                        return vm.$slots.foo('bar')
                      }
                    }
                    </script>
                  ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      render() {
                        const vm = this
                        function fn() {
                          return vm.$scopedSlots
                        }
                        return fn().foo('bar')
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      render() {
                        const vm = this
                        function fn() {
                          return vm.$slots
                        }
                        return fn().foo('bar')
                      }
                    }
                    </script>
                  ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      render () {
                        const vm = this
                        const a = vm?.$scopedSlots
                        const b = this?.$scopedSlots
                        return a.foo('bar')
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      render () {
                        const vm = this
                        const a = vm?.$slots
                        const b = this?.$slots
                        return a.foo('bar')
                      }
                    }
                    </script>
                  ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(
        NoDeprecatedDollarScopedslotsApi::NAME,
        NoDeprecatedDollarScopedslotsApi::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
