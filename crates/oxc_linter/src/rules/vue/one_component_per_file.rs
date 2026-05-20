use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{VueComponentObjectKind, vue_component_options_kind},
};

fn one_component_per_file_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("There is more than one component in this file.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct OneComponentPerFile;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that each component should be in its own file.
    ///
    /// ### Why is this bad?
    ///
    /// Keeping each Vue component in its own file helps discoverability,
    /// keeps tooling (HMR, code-splitting, generated docs) predictable,
    /// and matches the convention enforced by the Vue style guide.
    ///
    /// `Vue.mixin(...)` and `app.mixin(...)` are not components — they are
    /// excluded, as are `new Vue({...})` instances.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Vue.component('TodoList', { /* ... */ })
    /// Vue.component('TodoItem', { /* ... */ })
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Vue.component('TodoList', { /* ... */ })
    /// ```
    OneComponentPerFile,
    vue,
    style,
    version = "next",
);

impl Rule for OneComponentPerFile {
    fn run_once(&self, ctx: &LintContext) {
        let component_spans: Vec<Span> = ctx
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind(), AstKind::ObjectExpression(_)))
            .filter_map(|node| match vue_component_options_kind(node, ctx) {
                Some(VueComponentObjectKind::Export) => Some(node.kind().span()),
                Some(VueComponentObjectKind::Definition) if !is_mixin_definition(node, ctx) => {
                    Some(node.kind().span())
                }
                _ => None,
            })
            .collect();

        if component_spans.len() > 1 {
            for span in component_spans {
                ctx.diagnostic(one_component_per_file_diagnostic(span));
            }
        }
    }
}

/// Whether the `Definition`-kind options object is the argument of a `*.mixin(...)`
/// call — these are mixins, not components, so they shouldn't count toward the
/// "more than one component" check.
fn is_mixin_definition(node: &crate::AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.nodes().ancestors(node.id()).any(
        |ancestor| matches!(ancestor.kind(), AstKind::CallExpression(call) if is_mixin_call(call)),
    )
}

fn is_mixin_call(call: &CallExpression<'_>) -> bool {
    let Some(member_expr) = call.callee.get_member_expr() else {
        return false;
    };
    member_expr.static_property_name().is_some_and(|name| name == "mixin")
        && matches!(member_expr.object().get_inner_expression(), Expression::Identifier(_))
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        ("Vue.component('name', {})", None, None, Some(PathBuf::from("test.js"))),
        (
            "
                    Vue.component('name', {})
                    new Vue({})
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                    const foo = {}
                    new Vue({})
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        ("<script>export default {}</script>", None, None, Some(PathBuf::from("test.vue"))),
        (
            "<script>
                  export default {
                    components: {
                      test: {
                        name: 'foo'
                      }
                    }
                  }
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    Vue.mixin({})
                    Vue.component('name', {})
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    let fail = vec![
        (
            "
                    Vue.component('name', {})
                    Vue.component('name', {})
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                    Vue.component('TodoList', {
                      // ...
                    })

                    Vue.component('TodoItem', {
                      // ...
                    })
                    export default {}
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "<script>
                  Vue.component('name', {})
                  export default {}
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(OneComponentPerFile::NAME, OneComponentPerFile::PLUGIN, pass, fail)
        .test_and_snapshot();
}
