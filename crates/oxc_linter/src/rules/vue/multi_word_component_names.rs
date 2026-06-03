use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ExportDefaultDeclaration, ExportDefaultDeclarationKind,
        Expression, NewExpression, ObjectExpression,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::{find_property, vue_casing},
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiWordComponentNames(Box<MultiWordComponentNamesConfig>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct MultiWordComponentNamesConfig {
    /// Component names to allow.
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
    config = MultiWordComponentNames,
    version = "next",
);

impl Rule for MultiWordComponentNames {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Program(program) = node.kind() else {
            return;
        };

        let is_script_setup = ctx.frameworks_options() == FrameworkOptions::VueSetup;

        // Scan and report violations on the current sub-host with a focused
        // visitor (no whole-`semantic.nodes()` walk).
        let mut visitor = NameChecker::reporting(ctx, &self.ignores, is_script_setup);
        visitor.visit_program(program);
        let current = ScanState { has_name: visitor.has_name, has_vue: visitor.has_vue };

        // Filename fallback only runs once per file, on the first sub-host.
        if !ctx.is_first_sub_host() {
            return;
        }
        if current.has_name {
            return;
        }

        let mut has_name = false;
        let mut has_vue = is_script_setup || current.has_vue;
        let mut body_count = program.body.len();

        for other in ctx.other_file_hosts() {
            let other_setup = other.framework_options() == FrameworkOptions::VueSetup;
            if other_setup {
                has_vue = true;
            }
            let other_semantic = other.semantic();
            let other_program = other_semantic.nodes().program();
            body_count += other_program.body.len();

            let mut other_visitor = NameChecker::scan_only(other_setup);
            other_visitor.visit_program(other_program);
            if other_visitor.has_vue {
                has_vue = true;
            }
            if other_visitor.has_name {
                has_name = true;
                break;
            }
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

struct ScanState {
    has_name: bool,
    has_vue: bool,
}

/// AST visitor that locates Vue component options objects via their entry
/// points (`export default { ... }`, `defineComponent(...)`, `new Vue(...)`,
/// `Vue.component(...)`, `defineOptions(...)`), updates `has_name`/`has_vue`,
/// and (in reporting mode) emits multi-word violations as it walks. Used
/// instead of `is_vue_component_options_object`, which requires an
/// `AstNode` and an ancestor walk.
struct NameChecker<'a, 'b> {
    /// `Some` for the current sub-host (report individual violations through
    /// `ctx.diagnostic`); `None` for scan-only passes over other sub-hosts
    /// where we just want the aggregated flags.
    ctx: Option<&'b LintContext<'a>>,
    /// Slice of upstream `ignores` option values. Only used in reporting mode.
    ignores: &'b [String],
    is_script_setup: bool,
    has_name: bool,
    has_vue: bool,
}

impl<'a> NameChecker<'a, '_> {
    fn reporting<'b>(
        ctx: &'b LintContext<'a>,
        ignores: &'b [String],
        is_script_setup: bool,
    ) -> NameChecker<'a, 'b> {
        NameChecker { ctx: Some(ctx), ignores, is_script_setup, has_name: false, has_vue: false }
    }

    fn scan_only<'b>(is_script_setup: bool) -> NameChecker<'a, 'b> {
        NameChecker { ctx: None, ignores: &[], is_script_setup, has_name: false, has_vue: false }
    }

    fn check_options_object(&mut self, obj: &ObjectExpression<'a>) {
        self.has_vue = true;
        let Some(prop) = find_property(obj, "name") else { return };
        self.has_name = true;
        if let Some(ctx) = self.ctx {
            validate_value(ctx, &prop.value, self.ignores);
        }
    }
}

impl<'a> Visit<'a> for NameChecker<'a, '_> {
    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        // `<script>` only: `export default {...}` is a Vue options object.
        // In `<script setup>`, `export default` is invalid SFC and is ignored.
        if !self.is_script_setup {
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
                self.check_options_object(obj);
            }
        }
        walk::walk_export_default_declaration(self, decl);
    }

    fn visit_new_expression(&mut self, new_expr: &NewExpression<'a>) {
        // `new Vue({...})` — Instance kind.
        if let Expression::Identifier(callee) = &new_expr.callee
            && callee.name == "Vue"
            && let Some(arg) = new_expr.arguments.first().and_then(extract_object_argument)
        {
            self.check_options_object(arg);
        }
        walk::walk_new_expression(self, new_expr);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_dot_component_call(call) {
            self.has_vue = true;
            if call.arguments.len() == 2
                && let Some(arg) = call.arguments.first()
            {
                self.has_name = true;
                if let Some(ctx) = self.ctx {
                    validate_argument(ctx, arg, self.ignores);
                }
            }
        } else if self.is_script_setup
            && is_define_options_call(call)
            && let Some(Argument::ObjectExpression(obj)) = call.arguments.first()
        {
            self.check_options_object(obj);
        }
        walk::walk_call_expression(self, call);
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
