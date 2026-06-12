use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression,
        Statement,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNodes, NodeId};
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext, frameworks::FrameworkOptions, rule::Rule,
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

/// Identifiers that component-definition calls are reachable from without an AST walk:
/// either as the bare callee (`defineComponent({})`) or as the member object (`Vue.extend({})`).
const COMPONENT_FACTORY_NAMES: [&str; 5] =
    ["Vue", "component", "createApp", "defineComponent", "defineNuxtComponent"];

impl Rule for OneComponentPerFile {
    fn run_once(&self, ctx: &LintContext) {
        let is_vue_file = ctx.file_extension().is_some_and(|ext| ext == "vue");
        let is_script_setup = ctx.frameworks_options() == FrameworkOptions::VueSetup;

        // `<anyIdentifier>.component(...)` (and the `*.mixin(...)` exclusion) cannot be
        // reached from named references, but they cannot exist unless the property name
        // appears as a word in the source text. Only such files pay for a full AST walk.
        let component_spans = if contains_word(ctx.source_text(), "component")
            || contains_word(ctx.source_text(), "mixin")
        {
            let mut visitor =
                ComponentCollector { component_spans: Vec::new(), is_vue_file, is_script_setup };
            visitor.visit_program(ctx.nodes().program());
            visitor.component_spans
        } else {
            let mut spans = Vec::new();
            collect_from_references(ctx, is_vue_file, is_script_setup, &mut spans);
            spans.sort_unstable_by_key(|span| span.start);
            spans
        };

        if component_spans.len() > 1 {
            for span in &component_spans {
                ctx.diagnostic(one_component_per_file_diagnostic(*span));
            }
        }
    }
}

/// Whether `word` appears in `text` outside of any larger identifier
/// (e.g. `components:` does not count as `component`).
fn contains_word(text: &str, word: &str) -> bool {
    let bytes = text.as_bytes();
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let begin = start + pos;
        let end = begin + word.len();
        let before_ok = begin == 0 || !is_identifier_byte(bytes[begin - 1]);
        let after_ok = end == bytes.len() || !is_identifier_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        start = begin + 1;
    }
    false
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || !byte.is_ascii()
}

/// Mirrors [`ComponentCollector`] without walking the AST: `export default {...}` can only
/// appear in `program.body`, and every other component definition has one of
/// [`COMPONENT_FACTORY_NAMES`] as its callee or member object, so all definition sites are
/// reachable through the references to those names.
fn collect_from_references(
    ctx: &LintContext,
    is_vue_file: bool,
    is_script_setup: bool,
    spans: &mut Vec<Span>,
) {
    if is_vue_file && !is_script_setup {
        for stmt in &ctx.nodes().program().body {
            if let Statement::ExportDefaultDeclaration(export) = stmt
                && let ExportDefaultDeclarationKind::ObjectExpression(obj) = &export.declaration
            {
                spans.push(obj.span);
            }
        }
    }

    let scoping = ctx.scoping();
    let nodes = ctx.nodes();
    for name in COMPONENT_FACTORY_NAMES {
        if let Some(ref_ids) = scoping.root_unresolved_references().get(name) {
            for &ref_id in ref_ids {
                check_reference(nodes, scoping.get_reference(ref_id).node_id(), spans);
            }
        }
    }
    for symbol_id in scoping.symbol_ids() {
        if COMPONENT_FACTORY_NAMES.contains(&scoping.symbol_name(symbol_id)) {
            for reference in scoping.get_resolved_references(symbol_id) {
                check_reference(nodes, reference.node_id(), spans);
            }
        }
    }
}

fn check_reference(nodes: &AstNodes, ident_node_id: NodeId, spans: &mut Vec<Span>) {
    let ident_span = nodes.get_node(ident_node_id).kind().span();
    for ancestor in nodes.ancestors(ident_node_id) {
        match ancestor.kind() {
            AstKind::ParenthesizedExpression(_)
            | AstKind::ChainExpression(_)
            | AstKind::TSAsExpression(_)
            | AstKind::TSSatisfiesExpression(_)
            | AstKind::TSNonNullExpression(_)
            | AstKind::TSTypeAssertion(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::ComputedMemberExpression(_) => {}
            AstKind::CallExpression(call) => {
                // The span check ensures this reference is the identifier that
                // `is_vue_component_options_call` matches on, not e.g. an argument,
                // keeping the collected set identical to the AST-walk path.
                if callee_identifier_span(call) == Some(ident_span)
                    && is_vue_component_options_call(call)
                    && !is_mixin_call(call)
                    && let Some(last_arg) =
                        call.arguments.last().and_then(|arg| arg.as_expression())
                    && matches!(last_arg, Expression::ObjectExpression(_))
                {
                    spans.push(last_arg.span());
                }
                return;
            }
            _ => return,
        }
    }
}

/// The span of the identifier that drives [`is_vue_component_options_call`]:
/// the bare callee, or the object of the callee member expression.
fn callee_identifier_span(call: &CallExpression) -> Option<Span> {
    if let Some(ident) = call.callee.get_identifier_reference() {
        return Some(ident.span);
    }
    let member = call.callee.get_member_expr()?;
    if let Expression::Identifier(obj) = member.object().get_inner_expression() {
        return Some(obj.span);
    }
    None
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
        ("defineComponent({ name: 'A' })", None, None, Some(PathBuf::from("test.js"))),
        (
            "
                    import { defineComponent } from 'vue'
                    export const a = defineComponent({ name: 'A' })
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                    import { createApp } from 'vue'
                    createApp({ name: 'A' })
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
        (
            "
                    defineComponent({ name: 'A' })
                    defineComponent({ name: 'B' })
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                    import { defineComponent } from 'vue'
                    export const a = defineComponent({ name: 'A' })
                    export const b = defineComponent({ name: 'B' })
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                    import Vue from 'vue'
                    Vue.extend({ name: 'A' })
                    Vue.extend({ name: 'B' })
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "<script>
                  import { defineNuxtComponent } from '#imports'
                  defineNuxtComponent({ name: 'A' })
                  export default {}
                  </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    function setup() {
                      defineComponent({ name: 'A' })
                    }
                    defineComponent({ name: 'B' })
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                    app['component']('a', { name: 'A' })
                    app['component']('b', { name: 'B' })
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    Tester::new(OneComponentPerFile::NAME, OneComponentPerFile::PLUGIN, pass, fail)
        .test_and_snapshot();
}
