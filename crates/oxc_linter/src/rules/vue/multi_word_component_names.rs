use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Semantic;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::Rule,
    utils::{find_property, is_vue_component_options_object, vue_casing},
};

const VUE_BUILTIN_COMPONENTS: &[&str] = &[
    "template",
    "slot",
    "component",
    "Component",
    "transition",
    "Transition",
    "transition-group",
    "TransitionGroup",
    "keep-alive",
    "KeepAlive",
    "teleport",
    "Teleport",
    "suspense",
    "Suspense",
];

fn multi_word_component_names_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(r#"Component name "{name}" should always be multi-word."#))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MultiWordComponentNames(Box<MultiWordComponentNamesConfig>);

#[derive(Debug, Default, Clone)]
pub struct MultiWordComponentNamesConfig {
    ignores: Vec<String>,
}

impl Deref for MultiWordComponentNames {
    type Target = MultiWordComponentNamesConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require component names to be always multi-word.
    ///
    /// ### Why is this bad?
    ///
    /// Single-word component names may conflict with existing and future
    /// HTML elements, since custom elements are required to have a hyphen
    /// in their name.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'Todo'
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'TodoItem'
    /// }
    /// </script>
    /// ```
    MultiWordComponentNames,
    vue,
    correctness,
    version = "next",
);

impl Rule for MultiWordComponentNames {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let ignores: Vec<String> = value
            .get(0)
            .and_then(|v| v.get("ignores"))
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        Ok(Self(Box::new(MultiWordComponentNamesConfig { ignores })))
    }

    fn run_once(&self, ctx: &LintContext) {
        let semantic = ctx.semantic();
        let is_script_setup = ctx.frameworks_options() == FrameworkOptions::VueSetup;
        let mut has_name = false;
        let mut has_vue = is_script_setup;

        // Visit current sub-host (the one whose context we report on).
        for node in semantic.nodes() {
            match node.kind() {
                AstKind::ObjectExpression(obj) if is_vue_component_options_object(node, ctx) => {
                    has_vue = true;
                    if let Some(prop) = find_property(obj, "name") {
                        has_name = true;
                        validate_value(ctx, &prop.value, &self.ignores);
                    }
                }
                AstKind::CallExpression(call) => {
                    if is_dot_component_call(call) {
                        has_vue = true;
                        if call.arguments.len() == 2
                            && let Some(arg) = call.arguments.first()
                        {
                            has_name = true;
                            validate_argument(ctx, arg, &self.ignores);
                        }
                    } else if is_script_setup
                        && is_define_options_call(call)
                        && let Some(Argument::ObjectExpression(obj)) = call.arguments.first()
                    {
                        has_vue = true;
                        if let Some(prop) = find_property(obj, "name") {
                            has_name = true;
                            validate_value(ctx, &prop.value, &self.ignores);
                        }
                    }
                }
                _ => {}
            }
        }

        // Filename fallback only runs once per file, on the first sub-host.
        if !ctx.is_first_sub_host() {
            return;
        }

        let mut body_count = semantic.nodes().program().body.len();

        // Collect state from other script blocks. `is_vue_component_options_object` ties to
        // the current `LintContext`, so a lightweight handwritten check is used for other hosts.
        for other in ctx.other_file_hosts() {
            let other_setup = other.framework_options() == FrameworkOptions::VueSetup;
            if other_setup {
                has_vue = true;
            }
            let other_semantic = other.semantic();
            body_count += other_semantic.nodes().program().body.len();
            collect_state_from_other(other_semantic, other_setup, &mut has_name, &mut has_vue);
        }

        if has_name {
            return;
        }

        // Mirrors the upstream `if (!hasVue && node.body.length > 0) return` guard.
        if !has_vue && body_count > 0 {
            return;
        }

        let path = ctx.file_path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("vue") {
            return;
        }
        let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return;
        };
        if is_valid_component_name(file_stem, &self.ignores) {
            return;
        }
        ctx.diagnostic(multi_word_component_names_diagnostic(Span::new(0, 0), file_stem));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

fn collect_state_from_other(
    semantic: &Semantic<'_>,
    is_script_setup: bool,
    has_name: &mut bool,
    has_vue: &mut bool,
) {
    for node in semantic.nodes() {
        match node.kind() {
            AstKind::ExportDefaultDeclaration(decl) if !is_script_setup => {
                let obj = match &decl.declaration {
                    ExportDefaultDeclarationKind::ObjectExpression(obj) => Some(obj.as_ref()),
                    ExportDefaultDeclarationKind::CallExpression(call)
                        if is_vue_definition_call(call) =>
                    {
                        call.arguments.last().and_then(extract_object_argument)
                    }
                    _ => None,
                };
                if let Some(obj) = obj {
                    *has_vue = true;
                    if find_property(obj, "name").is_some() {
                        *has_name = true;
                    }
                }
            }
            AstKind::CallExpression(call) => {
                if is_dot_component_call(call) && call.arguments.len() == 2 {
                    *has_vue = true;
                    *has_name = true;
                } else if is_script_setup
                    && is_define_options_call(call)
                    && let Some(Argument::ObjectExpression(obj)) = call.arguments.first()
                {
                    *has_vue = true;
                    if find_property(obj, "name").is_some() {
                        *has_name = true;
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_object_argument<'a, 'b>(arg: &'b Argument<'a>) -> Option<&'b ObjectExpression<'a>> {
    if let Argument::ObjectExpression(obj) = arg { Some(obj.as_ref()) } else { None }
}

fn is_vue_definition_call(call: &CallExpression) -> bool {
    call.callee.get_identifier_reference().is_some_and(|ident| {
        matches!(ident.name.as_str(), "defineComponent" | "defineNuxtComponent" | "createApp")
    })
}

fn validate_value(ctx: &LintContext, expr: &Expression, ignores: &[String]) {
    let Expression::StringLiteral(lit) = expr else {
        return;
    };
    let name = lit.value.as_str();
    if !is_valid_component_name(name, ignores) {
        ctx.diagnostic(multi_word_component_names_diagnostic(lit.span, name));
    }
}

fn validate_argument(ctx: &LintContext, arg: &Argument, ignores: &[String]) {
    let Argument::StringLiteral(lit) = arg else {
        return;
    };
    let name = lit.value.as_str();
    if !is_valid_component_name(name, ignores) {
        ctx.diagnostic(multi_word_component_names_diagnostic(lit.span, name));
    }
}

fn is_dot_component_call(call: &CallExpression) -> bool {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    member.property.name == "component"
}

fn is_define_options_call(call: &CallExpression) -> bool {
    let Expression::Identifier(ident) = &call.callee else {
        return false;
    };
    ident.name == "defineOptions"
}

fn is_valid_component_name(name: &str, ignores: &[String]) -> bool {
    if name == "App" || name == "app" {
        return true;
    }
    if VUE_BUILTIN_COMPONENTS.contains(&name) {
        return true;
    }
    for ignore in ignores {
        if ignore == name {
            return true;
        }
        if !ignore.is_empty()
            && vue_casing::is_pascal_case(ignore)
            && vue_casing::kebab_case(ignore) == name
        {
            return true;
        }
    }
    vue_casing::kebab_case(name).split('-').count() > 1
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        ("", None, None, Some(PathBuf::from("App.vue"))),
        (
            "
                    <script>
                    export default { name: 'App' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('App', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        ("", None, None, Some(PathBuf::from("app.vue"))),
        ("<script></script>", None, None, Some(PathBuf::from("path/to/app.vue"))),
        (
            "
                    <script>
                    export default { name: 'app' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('app', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        ("", None, None, Some(PathBuf::from("transition.vue"))),
        (
            "
                    <script>
                    export default { name: 'transition' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('transition', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        ("", None, None, Some(PathBuf::from("component.vue"))),
        (
            "
                    <script>
                    export default { name: 'component' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('component', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        ("", None, None, Some(PathBuf::from("multi-word.vue"))),
        (
            "
                    <script>
                    export default { name: 'multi-word' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('multi-word', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        ("", None, None, Some(PathBuf::from("multiWord.vue"))),
        (
            "
                    <script>
                    export default { name: 'multiWord' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('multiWord', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        ("", None, None, Some(PathBuf::from("MultiWord.vue"))),
        (
            "
                    <script>
                    export default { name: 'MultiWord' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('MultiWord', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                  <script>
                  export default { name: 'TheTest' }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("TheTest.vue")),
        ),
        (
            "
                  <script>
                  Vue.component('TheTest', {})
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("TheTest.vue")),
        ),
        (
            "
                  <script>
                  export default {
                    name: 'Todo'
                  }
                  </script>
                  ",
            Some(serde_json::json!([{ "ignores": ["Todo"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  new Vue({})
                  ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
                  import Vue from 'vue'
                  import VueCompositionAPI, { h } from '@vue/composition-api'
                  import i18n from '@/i18n'
                  import router from '@/router'
                  import store from '@/store'
                  // ...

                  Vue.use(VueCompositionAPI)

                  new Vue({
                      i18n,
                      router,
                      store,
                      setup() {
                          return () => h(App)
                      },
                  }).$mount('#app')
                  ",
            None,
            None,
            Some(PathBuf::from("main.ts")),
        ),
        (
            r#"
                  <template>
                    <AppButton />
                  </template>

                  <script setup lang="ts">
                  import AppButton from "@/components/AppButton.vue";
                  </script>"#,
            None,
            None,
            Some(PathBuf::from("MultiWord.vue")),
        ),
        (
            "
                  <script setup>
                  defineOptions({name: 'MultiWord'})
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("Single.vue")),
        ),
    ];

    let fail = vec![
        // Upstream additionally has `("", "test.vue")` but oxlint's partial loader
        // skips `.vue` files without any `<script>` block, so the filename fallback
        // cannot run for that case. Tracked as follow-up.
        (
            "
                    <script>
                    export default { name: 'test' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('test', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default { name: 'invalid' }
                    </script>",
            None,
            None,
            Some(PathBuf::from("valid-name.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('invalid', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("valid-name.vue")),
        ),
        (
            "
                    <script>
                    export default {}
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                    <script>
                    Vue.component('', {})
                    </script>",
            None,
            None,
            Some(PathBuf::from("invalid.vue")),
        ),
        (
            "
                  <script>
                  export default {
                    name: 'Item'
                  }
                  </script>
                  ",
            Some(serde_json::json!([{ "ignores": ["Todo"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  import Item from "@/components/Item.vue";
                  </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  defineOptions({name: 'Single'})
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("MultiWord.vue")),
        ),
    ];

    Tester::new(MultiWordComponentNames::NAME, MultiWordComponentNames::PLUGIN, pass, fail)
        .test_and_snapshot();
}
