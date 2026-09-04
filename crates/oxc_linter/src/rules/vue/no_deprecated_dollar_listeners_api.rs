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

fn no_deprecated_dollar_listeners_api_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`$listeners` is deprecated.")
        .with_help("Use `$attrs` instead; in Vue 3, listeners are included in `$attrs`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedDollarListenersApi;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated `$listeners` (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// In Vue 3, the `$listeners` instance property was removed. Listeners are
    /// now part of `$attrs`, so accessing `this.$listeners` no longer works.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     listeners() {
    ///       return this.$listeners
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     listeners() {
    ///       return this.$attrs
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedDollarListenersApi,
    vue,
    correctness,
    version = "next",
    short_description = "Disallow using deprecated `$listeners` (in Vue.js 3.0.0+).",
);

impl Rule for NoDeprecatedDollarListenersApi {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(member) = node.kind() else { return };
        if member.property.name != "$listeners" {
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
            ctx.diagnostic(no_deprecated_dollar_listeners_api_diagnostic(member.property.span));
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;
    // ref: https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/no-deprecated-dollar-listeners-api.test.ts

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
                      console.log(this.$listeners)
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
                      <div foo="$listeners"/>
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
                        function click ($listeners) {
                          fn(foo.$listeners)
                          fn($listeners)
                        }
                      }"/>
                      <div v-for="$listeners in list">
                        <div v-on="$listeners">
                      </div>
                      <VueComp>
                        <template v-slot="{$listeners}">
                          <div v-on="$listeners">
                        </template>
                      </VueComp>
                    </template>
                    <script>
                    export default {
                      methods: {
                        click ($listeners) {
                          foo.$listeners
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
                          return vm.$listeners
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
                      <div v-on="$listeners"/>
                    </template>
                    <script>
                    export default {
                      computed: {
                        foo () {
                          return this.$listeners
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
            r#"
                    <template>
                      <div v-for="listener in $listeners"/>
                      <div :foo="$listeners"/>
                    </template>
                    <script>
                    export default {
                      computed: {
                        foo () {
                          fn(this.$listeners)
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
                          const vm = this
                          return vm.$listeners
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
                      computed: {
                        foo () {
                          const vm = this
                          function fn() {
                            return vm.$listeners
                          }
                          return fn()
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
                      computed: {
                        foo () {
                          const vm = this
                          const a = vm?.$listeners
                          const b = this?.$listeners
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

    Tester::new(
        NoDeprecatedDollarListenersApi::NAME,
        NoDeprecatedDollarListenersApi::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
