use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, ExpressionKind, ObjectExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::{find_property, is_vue_component_options_object, vue_casing},
};

fn component_definition_name_casing_diagnostic(
    span: Span,
    value: &str,
    case_type: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Property name \"{value}\" is not {case_type}.")).with_label(span)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum CaseType {
    #[default]
    #[serde(rename = "PascalCase")]
    PascalCase,
    #[serde(rename = "kebab-case")]
    KebabCase,
}

impl CaseType {
    fn as_str(self) -> &'static str {
        match self {
            CaseType::PascalCase => "PascalCase",
            CaseType::KebabCase => "kebab-case",
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ComponentDefinitionNameCasing(CaseType);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce specific casing for component definition names.
    ///
    /// ### Why is this bad?
    ///
    /// Defining component names without a consistent casing makes templates
    /// harder to read and harder to grep. Picking either `PascalCase` or
    /// `kebab-case` and sticking with it across the codebase removes a class
    /// of bikeshedding and search misses.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (default `PascalCase`):
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'foo-bar'
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule (default `PascalCase`):
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'FooBar'
    /// }
    /// </script>
    /// ```
    ComponentDefinitionNameCasing,
    vue,
    style,
    fix,
    config = CaseType,
    version = "1.68.0",
    short_description = "Enforce specific casing for component definition names.",
);

impl Rule for ComponentDefinitionNameCasing {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => {
                self.check_call_expression(call, ctx);
            }
            AstKind::ObjectExpression(_) => {
                self.check_options_object(node, ctx);
            }
            _ => {}
        }
    }
}

impl ComponentDefinitionNameCasing {
    fn check_call_expression<'a>(&self, call: &CallExpression<'a>, ctx: &LintContext<'a>) {
        // `defineOptions({ name: '...' })`
        if let Some(ident) = call.callee.get_identifier_reference()
            && ident.name == "defineOptions"
            && ctx.frameworks_options() == FrameworkOptions::VueSetup
            && let Some(arg) = call.arguments.first()
            && let Some(obj) = arg.as_expression().and_then(Expression::as_object_expression)
        {
            self.check_name_property(obj, ctx);
            return;
        }

        // `Vue.component('Name', ...)` / `app.component('Name', ...)` /
        // `(Vue as VueConstructor<Vue>).component('Name', ...)`
        if let Some(member_expr) = call.callee.get_inner_expression().as_member_expression()
            && member_expr.static_property_name().is_some_and(|prop_name| prop_name == "component")
            && call.arguments.len() == 2
            && let Some(first_arg) = call.arguments.first()
            && let Some(first_expr) = first_arg.as_expression()
        {
            self.check_name_node(first_expr, ctx);
        }
    }

    fn check_options_object<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj) = node.kind() else { return };
        if !is_vue_component_options_object(node, ctx) {
            return;
        }
        self.check_name_property(obj, ctx);
    }

    fn check_name_property<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        let Some(prop) = find_property(obj, "name") else { return };
        self.check_name_node(&prop.value, ctx);
    }

    fn check_name_node(&self, expr: &Expression<'_>, ctx: &LintContext<'_>) {
        let inner = expr.get_inner_expression();
        let Some((value, inner_span)) = extract_convertible(inner) else { return };

        let case_type = self.0;
        if check_case(&value, case_type) {
            return;
        }

        let report_span = inner.span();
        let case_type_str = case_type.as_str();
        let diagnostic =
            component_definition_name_casing_diagnostic(report_span, value.as_str(), case_type_str);

        if let Some(converted) = exact_convert(&value, case_type) {
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace(inner_span, converted));
        } else {
            ctx.diagnostic(diagnostic);
        }
    }
}

/// Returns `(value, inner_span)` if `expr` is a string literal or a
/// simple template literal (no expressions, single quasi). `inner_span`
/// is the range of the literal *contents* (excluding the quotes / backticks),
/// suitable as a fix target.
fn extract_convertible(expr: &Expression<'_>) -> Option<(String, Span)> {
    match expr.kind() {
        ExpressionKind::StringLiteral(lit) => {
            let inner = Span::new(lit.span.start + 1, lit.span.end - 1);
            Some((lit.value.to_string(), inner))
        }
        ExpressionKind::TemplateLiteral(tpl) => {
            if !tpl.expressions.is_empty() || tpl.quasis.len() != 1 {
                return None;
            }
            let quasi = tpl.quasis.first()?;
            let cooked = quasi.value.cooked.as_ref()?;
            let inner = Span::new(tpl.span.start + 1, tpl.span.end - 1);
            Some((cooked.to_string(), inner))
        }
        _ => None,
    }
}

fn check_case(s: &str, case_type: CaseType) -> bool {
    match case_type {
        CaseType::PascalCase => vue_casing::is_pascal_case(s),
        CaseType::KebabCase => vue_casing::is_kebab_case(s),
    }
}

/// Mirror of `getExactConverter(name)(str)`. Returns `None` when the
/// converted value still does not satisfy the checker — upstream returns
/// the original value in that case, which ESLint then treats as "no
/// effective fix". On our side we surface that as "no fix available" so
/// that the diagnostic is emitted without an autofix.
fn exact_convert(s: &str, case_type: CaseType) -> Option<String> {
    let converted = match case_type {
        CaseType::PascalCase => vue_casing::pascal_case(s),
        CaseType::KebabCase => vue_casing::kebab_case(s),
    };
    if check_case(&converted, case_type) { Some(converted) } else { None }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let vue = || Some(PathBuf::from("test.vue"));
    let js = || Some(PathBuf::from("test.js"));

    let pass = vec![
        (
            "<script>
                    export default {
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      ...name
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'FooBar'
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'FooBar'
                    }
                    </script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo-bar'
                    }
                    </script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        ("<script>Vue.component('FooBar', {})</script>", None, None, vue()),
        (
            "<script>Vue.component('FooBar', {})</script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        (
            "<script>Vue.component('foo-bar', {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        (
            "<script>Vue.component(fooBar, {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        ("<script>Vue.component('FooBar', component)</script>", None, None, vue()),
        (
            "<script>Vue.component('FooBar', component)</script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        (
            "<script>Vue.component('foo-bar', component)</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        (
            "<script>Vue.component(fooBar, component)</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        (
            "<script>app.component('FooBar', component)</script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        ("<script>Vue.mixin({})</script>", None, None, vue()),
        ("<script>foo({})</script>", None, None, vue()),
        ("<script>foo('foo-bar', {})</script>", None, None, vue()),
        (
            "<script>Vue.component(`fooBar${foo}`, component)</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        (
            "<script>app.component(`fooBar${foo}`, component)</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        // https://github.com/vuejs/eslint-plugin-vue/issues/1018
        ("fn1(component.data)", None, None, js()),
        ("<script setup> defineOptions({}) </script>", None, None, vue()),
        ("<script> defineOptions({name: 'foo-bar'}) </script>", None, None, vue()),
        ("defineOptions({name: 'foo-bar'})", None, None, js()),
        ("<template>{{ Vue.component('foo-bar', {}) }}</template>", None, None, vue()),
        (
            "<script setup> defineOptions({name: 'FooBar'}) </script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        (
            "<script setup> defineOptions({name: 'foo-bar'}) </script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
    ];

    let fail = vec![
        (
            "<script>
                    export default {
                      name: 'foo-bar'
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo  bar'
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo!bar'
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "
                    new Vue({
                      name: 'foo!bar'
                    })
                  ",
            None,
            None,
            js(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo_bar'
                    }
                    </script>",
            None,
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo_bar'
                    }
                    </script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo_bar'
                    }
                    </script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        ("<script>Vue.component('foo-bar', component)</script>", None, None, vue()),
        ("<script>app.component('foo-bar', component)</script>", None, None, vue()),
        (
            "<script lang=\"ts\">(Vue as VueConstructor<Vue>).component('foo-bar', component)</script>",
            None,
            None,
            vue(),
        ),
        ("<script>Vue.component('foo-bar', {})</script>", None, None, vue()),
        ("<script>app.component('foo-bar', {})</script>", None, None, vue()),
        ("Vue.component('foo_bar', {})", Some(serde_json::json!(["PascalCase"])), None, js()),
        (
            "<script>Vue.component('foo_bar', {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        (
            "<script>Vue.component(`foo_bar`, {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        ("<script>Vue.component('foo-é', {})</script>", None, None, vue()),
        (
            "<script>Vue.component('$Foo', {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
        (
            "<script setup> defineOptions({name: 'foo-bar'}) </script>",
            Some(serde_json::json!(["PascalCase"])),
            None,
            vue(),
        ),
        (
            "<script setup> defineOptions({name: 'FooBar'}) </script>",
            Some(serde_json::json!(["kebab-case"])),
            None,
            vue(),
        ),
    ];

    let fix = vec![
        (
            "<script>
                    export default {
                      name: 'foo-bar'
                    }
                    </script>",
            "<script>
                    export default {
                      name: 'FooBar'
                    }
                    </script>",
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo_bar'
                    }
                    </script>",
            "<script>
                    export default {
                      name: 'FooBar'
                    }
                    </script>",
            None,
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo_bar'
                    }
                    </script>",
            "<script>
                    export default {
                      name: 'FooBar'
                    }
                    </script>",
            Some(serde_json::json!(["PascalCase"])),
            vue(),
        ),
        (
            "<script>
                    export default {
                      name: 'foo_bar'
                    }
                    </script>",
            "<script>
                    export default {
                      name: 'foo-bar'
                    }
                    </script>",
            Some(serde_json::json!(["kebab-case"])),
            vue(),
        ),
        (
            "<script>Vue.component('foo-bar', component)</script>",
            "<script>Vue.component('FooBar', component)</script>",
            None,
            vue(),
        ),
        (
            "<script>app.component('foo-bar', component)</script>",
            "<script>app.component('FooBar', component)</script>",
            None,
            vue(),
        ),
        (
            "<script lang=\"ts\">(Vue as VueConstructor<Vue>).component('foo-bar', component)</script>",
            "<script lang=\"ts\">(Vue as VueConstructor<Vue>).component('FooBar', component)</script>",
            None,
            vue(),
        ),
        (
            "<script>Vue.component('foo-bar', {})</script>",
            "<script>Vue.component('FooBar', {})</script>",
            None,
            vue(),
        ),
        (
            "<script>app.component('foo-bar', {})</script>",
            "<script>app.component('FooBar', {})</script>",
            None,
            vue(),
        ),
        (
            "<script>Vue.component('foo_bar', {})</script>",
            "<script>Vue.component('FooBar', {})</script>",
            Some(serde_json::json!(["PascalCase"])),
            vue(),
        ),
        (
            "<script>Vue.component('foo_bar', {})</script>",
            "<script>Vue.component('foo-bar', {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            vue(),
        ),
        (
            "<script>Vue.component(`foo_bar`, {})</script>",
            "<script>Vue.component(`foo-bar`, {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            vue(),
        ),
        (
            "<script>Vue.component('$Foo', {})</script>",
            "<script>Vue.component('$foo', {})</script>",
            Some(serde_json::json!(["kebab-case"])),
            vue(),
        ),
        (
            "<script setup> defineOptions({name: 'foo-bar'}) </script>",
            "<script setup> defineOptions({name: 'FooBar'}) </script>",
            Some(serde_json::json!(["PascalCase"])),
            vue(),
        ),
        (
            "<script setup> defineOptions({name: 'FooBar'}) </script>",
            "<script setup> defineOptions({name: 'foo-bar'}) </script>",
            Some(serde_json::json!(["kebab-case"])),
            vue(),
        ),
    ];

    Tester::new(
        ComponentDefinitionNameCasing::NAME,
        ComponentDefinitionNameCasing::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
