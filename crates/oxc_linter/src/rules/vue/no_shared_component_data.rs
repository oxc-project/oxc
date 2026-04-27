use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_shared_component_data_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`data` property in component must be a function.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSharedComponentData;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that the `data` property of a Vue component definition is a
    /// function.
    ///
    /// ### Why is this bad?
    ///
    /// When `data` is declared as an object literal, the same object is shared
    /// across every instance of the component, which causes cross-instance
    /// state pollution. Returning a fresh object from a function avoids that.
    ///
    /// This rule targets component definitions reached through
    /// `Vue.component(...)`, `Vue.extend(...)`, `Vue.mixin(...)`,
    /// `app.component(...)`, `app.mixin(...)`, `component(...)`,
    /// `createApp(...)`, `defineComponent(...)`, `defineNuxtComponent(...)`,
    /// and `export default {}` inside `.vue` files. `new Vue({...})` is not
    /// covered (the instance does not share `data` between components).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// Vue.component('some-comp', {
    ///   data: {
    ///     foo: 'bar'
    ///   }
    /// })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// Vue.component('some-comp', {
    ///   data() {
    ///     return { foo: 'bar' }
    ///   }
    /// })
    /// </script>
    /// ```
    NoSharedComponentData,
    vue,
    correctness,
    pending,
    version = "next",
);

impl Rule for NoSharedComponentData {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectProperty(prop) = node.kind() else { return };

        if !prop.key.is_specific_static_name("data") {
            return;
        }

        if !matches!(prop.value.get_inner_expression(), Expression::ObjectExpression(_)) {
            return;
        }

        let mut ancestors = ctx.nodes().ancestors(node.id());
        let Some(parent) = ancestors.next() else { return };
        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        let Some(grand) = ancestors.next() else { return };
        let in_vue_component = match grand.kind() {
            AstKind::ExportDefaultDeclaration(_) => true,
            AstKind::CallExpression(call) => is_vue_component_definition_call(call),
            _ => false,
        };
        if !in_vue_component {
            return;
        }

        ctx.diagnostic(no_shared_component_data_diagnostic(prop.span));
    }
}

fn is_vue_component_definition_call(call: &CallExpression<'_>) -> bool {
    let callee = call.callee.get_inner_expression();

    if let Expression::Identifier(ident) = callee {
        return matches!(
            ident.name.as_str(),
            "defineComponent" | "component" | "createApp" | "defineNuxtComponent"
        );
    }

    let Some(MemberExpression::StaticMemberExpression(static_member)) =
        callee.as_member_expression()
    else {
        return false;
    };
    let prop_name = static_member.property.name.as_str();
    if let Expression::Identifier(obj_ident) = static_member.object.get_inner_expression()
        && obj_ident.name == "Vue"
    {
        return matches!(prop_name, "component" | "mixin" | "extend");
    }
    matches!(prop_name, "component" | "mixin")
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        // `new Vue({...})` is excluded — instances don't share `data` across components.
        (
            "
                <script>
                new Vue({
                  data: {
                    foo: 'bar'
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  data: function () {
                    return { foo: 'bar' }
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  ...data,
                  data () {
                    return { foo: 'bar' }
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                Vue.component('some-comp', {
                  data: function () {
                    return { foo: 'bar' }
                  }
                })
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
                  data: function () {
                    return { foo: 'bar' }
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
                  ...foo
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
                  data
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
                  data: () => ({ foo: 'bar' })
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
                defineComponent({
                  data () { return { foo: 'bar' } }
                })
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
                Vue.component('some-comp', {
                  data: {
                    foo: 'bar'
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                app.component('some-comp', {
                  data: {
                    foo: 'bar'
                  }
                })
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
                  data: {
                    foo: 'bar'
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
                  data: /*a*/ (/*b*/{
                    foo: 'bar'
                  })
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
                Vue.extend({
                  data: { foo: 'bar' }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                createApp({
                  data: { foo: 'bar' }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                defineComponent({
                  data: { foo: 'bar' }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoSharedComponentData::NAME, NoSharedComponentData::PLUGIN, pass, fail)
        .test_and_snapshot();
}
