use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{find_property, is_vue_component_options_object_excluding_instance, vue_casing},
};

fn match_component_import_name_diagnostic(
    span: Span,
    alias: &str,
    pascal: &str,
    kebab: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Component alias {alias} should be one of: {pascal}, {kebab}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MatchComponentImportName;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires the key used in the `components` option to match the name of the
    /// imported component, either in `PascalCase` or in `kebab-case`.
    ///
    /// ### Why is this bad?
    ///
    /// Registering a component under an unrelated alias hides which import a
    /// tag in the template actually resolves to, which makes the code more
    /// difficult to follow for the reader.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// import SomeRandomName from './SomeRandomName.vue';
    ///
    /// export default {
    ///   components: { InvalidName: SomeRandomName }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// import ValidImport from './ValidImport.vue';
    ///
    /// export default {
    ///   components: { ValidImport, 'valid-import': ValidImport }
    /// }
    /// </script>
    /// ```
    MatchComponentImportName,
    vue,
    style,
    version = "next",
    short_description = "Require the registered component name to match the imported component name.",
);

impl Rule for MatchComponentImportName {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj) = node.kind() else {
            return;
        };
        let Some(components) = find_property(obj, "components") else {
            return;
        };
        // `without_parentheses`, not `get_inner_expression`: ESTree has no
        // parenthesized-expression node, so upstream sees through parens, but
        // it does *not* see through `as` casts.
        let Expression::ObjectExpression(components) = components.value.without_parentheses()
        else {
            return;
        };
        // Checked last: it walks the ancestor chain, so only objects that
        // actually carry a `components: { ... }` option pay for it.
        if !is_vue_component_options_object_excluding_instance(node, ctx) {
            return;
        }

        for prop in &components.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                continue;
            };
            if prop.computed {
                continue;
            }
            let Expression::Identifier(imported) = prop.value.without_parentheses() else {
                continue;
            };

            let Some(alias) = prop.key.static_name() else {
                continue;
            };
            let pascal = vue_casing::pascal_case(&imported.name);
            if alias == pascal.as_str() {
                continue;
            }
            let kebab = vue_casing::kebab_case(&imported.name);
            if alias == kebab.as_str() {
                continue;
            }

            ctx.diagnostic(match_component_import_name_diagnostic(
                prop.span, &alias, &pascal, &kebab,
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let vue = || Some(PathBuf::from("test.vue"));
    let js = || Some(PathBuf::from("test.js"));

    let pass = vec![
        ("<script> export default { components: { ValidImport } } </script>", None, None, vue()),
        (
            "<script> export default { components: { ValidImport: ValidImport } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { 'valid-import': ValidImport } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { ValidImport, ...SpreadImport } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { 'valid-import': ValidImport, [computedImport]: ComputedImport } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { ValidImport, [differentComputedImport]: ComputedImport } } </script>",
            None,
            None,
            vue(),
        ),
        // `components` that is not an object literal is ignored.
        ("<script> export default { components } </script>", None, None, vue()),
        ("<script> export default { components: [SomeRandomName] } </script>", None, None, vue()),
        // Values that are not plain identifiers are ignored.
        (
            "<script> export default { components: { InvalidExport: () => import('./Foo.vue') } } </script>",
            None,
            None,
            vue(),
        ),
        // Upstream compares `property.value.type` without unwrapping `as`
        // casts, so a cast value is skipped rather than reported.
        (
            "<script lang=\"ts\"> export default { components: { InvalidExport: SomeRandomName as any } } </script>",
            None,
            None,
            vue(),
        ),
        // A quoted key is compared the same as a bare one.
        (
            "<script> export default { components: { 'ValidImport': ValidImport } } </script>",
            None,
            None,
            vue(),
        ),
        // Parens around the value are invisible to upstream, so a name that
        // matches through them still passes.
        (
            "<script> export default { components: { ValidImport: (ValidImport) } } </script>",
            None,
            None,
            vue(),
        ),
        // Not a component options object.
        (
            "<script> export default { foo: { components: { InvalidExport: SomeRandomName } } } </script>",
            None,
            None,
            vue(),
        ),
        // `export default` in a non-vue file is not a component.
        ("export default { components: { InvalidExport: SomeRandomName } }", None, None, js()),
        // `new Vue({...})` is an instance, not a component definition.
        ("new Vue({ components: { InvalidExport: SomeRandomName } })", None, None, js()),
        // Component definition calls are checked in plain JS too.
        ("defineComponent({ components: { ValidImport } })", None, None, js()),
    ];

    let fail = vec![
        (
            "<script> export default { components: { InvalidExport: SomeRandomName } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { 'invalid-export': SomeRandomName } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { validImport: ValidImport } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: { ValidImport, InvalidExport: SomeRandomName } } </script>",
            None,
            None,
            vue(),
        ),
        ("defineComponent({ components: { InvalidExport: SomeRandomName } })", None, None, js()),
        (
            "<script> Vue.component('Foo', { components: { InvalidExport: SomeRandomName } }) </script>",
            None,
            None,
            vue(),
        ),
        // A camelCase shorthand matches neither expected casing.
        ("<script> export default { components: { someRandomName } } </script>", None, None, vue()),
        // Parens are invisible to upstream, so these are still reported.
        (
            "<script> export default { components: { InvalidExport: (SomeRandomName) } } </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script> export default { components: ({ InvalidExport: SomeRandomName }) } </script>",
            None,
            None,
            vue(),
        ),
    ];

    Tester::new(MatchComponentImportName::NAME, MatchComponentImportName::PLUGIN, pass, fail)
        .test_and_snapshot();
}
