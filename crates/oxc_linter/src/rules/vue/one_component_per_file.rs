use oxc_ast::{
    AstKind,
    ast::{CallExpression, ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule,
    utils::is_vue_component_options_call,
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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Program(program) = node.kind() else {
            return;
        };

        let mut visitor = ComponentCollector {
            component_spans: Vec::new(),
            is_vue_file: ctx.file_extension().is_some_and(|ext| ext == "vue"),
            is_script_setup: ctx.frameworks_options() == FrameworkOptions::VueSetup,
        };
        visitor.visit_program(program);

        if visitor.component_spans.len() > 1 {
            for span in &visitor.component_spans {
                ctx.diagnostic(one_component_per_file_diagnostic(*span));
            }
        }
    }
}

struct ComponentCollector {
    component_spans: Vec<Span>,
    is_vue_file: bool,
    is_script_setup: bool,
}

impl<'a> Visit<'a> for ComponentCollector {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_vue_component_options_call(call)
            && !is_mixin_call(call)
            && let Some(last_arg) = call.arguments.last().and_then(|arg| arg.as_expression())
            && matches!(last_arg, Expression::ObjectExpression(_))
        {
            self.component_spans.push(last_arg.span());
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        if self.is_vue_file
            && !self.is_script_setup
            && let ExportDefaultDeclarationKind::ObjectExpression(obj) = &export.declaration
        {
            self.component_spans.push(obj.span);
        }
        walk::walk_export_default_declaration(self, export);
    }
}

/// Whether the call is a `*.mixin(...)` — these are mixins, not components,
/// so they shouldn't count toward the "more than one component" check.
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
