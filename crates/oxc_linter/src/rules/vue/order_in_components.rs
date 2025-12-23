use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ExportDefaultDeclarationKind, Expression, NewExpression,
        ObjectExpression, ObjectPropertyKind, PropertyKey,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext, rule::Rule};

/// Information about a property in the component definition
struct PropertyInfo {
    /// The name of the property
    name: String,
    /// The position in the order configuration (None if not in order list)
    order_position: Option<usize>,
    /// The span of the property for error reporting
    span: Span,
}

fn order_in_components_diagnostic(
    current_name: &str,
    should_be_before: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The \"{current_name}\" property should be above the \"{should_be_before}\" property."
    ))
    .with_help("Enforce a convention in the order of component options for better readability.")
    .with_label(span)
}

/// Default order based on Vue style guide
const DEFAULT_ORDER: &[&[&str]] = &[
    // Side effects
    &["el"],
    // Global awareness
    &["name", "key"],
    &["parent"],
    // Component type
    &["functional"],
    // Template modifiers
    &["delimiters", "comments"],
    // Template dependencies
    &["components", "directives", "filters"],
    // Composition
    &["extends"],
    &["mixins"],
    &["provide", "inject"],
    // Page options (Nuxt)
    &[
        "validate",
        "asyncData",
        "fetch",
        "head",
        "scrollToTop",
        "transition",
        "loading",
        "layout",
        "middleware",
        "watchQuery",
    ],
    // Interface
    &["inheritAttrs", "model"],
    &["props", "propsData"],
    &["emits"],
    &["slots"],
    &["expose"],
    // Local state
    &["setup"],
    &["data"],
    &["computed"],
    // Events
    &["watch"],
    // Lifecycle hooks
    &[
        "beforeCreate",
        "created",
        "beforeMount",
        "mounted",
        "beforeUpdate",
        "updated",
        "activated",
        "deactivated",
        "beforeUnmount",
        "unmounted",
        "beforeDestroy",
        "destroyed",
        "renderTracked",
        "renderTriggered",
        "errorCaptured",
    ],
    // Router guards
    &["beforeRouteEnter", "beforeRouteUpdate", "beforeRouteLeave"],
    // Non-reactive properties
    &["methods"],
    // Rendering
    &["template", "render"],
    &["renderError"],
];

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct OrderInComponentsConfig {
    /// Custom order of component options
    order: Option<Vec<OrderElement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum OrderElement {
    Single(String),
    Group(Vec<String>),
}

#[derive(Debug, Default, Clone)]
pub struct OrderInComponents(Box<OrderInComponentsConfig>);

impl std::ops::Deref for OrderInComponents {
    type Target = OrderInComponentsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent order of properties in Vue component definitions.
    ///
    /// ### Why is this bad?
    ///
    /// Having an inconsistent order of component properties makes the code harder
    /// to read and maintain. Following a standard order helps developers quickly
    /// find specific options in components.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   data() { return { foo: 'bar' } },
    ///   name: 'MyComponent',  // should be before data
    ///   props: ['value'],     // should be before data
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'MyComponent',
    ///   props: ['value'],
    ///   data() { return { foo: 'bar' } },
    /// }
    /// </script>
    /// ```
    OrderInComponents,
    vue,
    style,
    config = OrderInComponentsConfig,
);

impl Rule for OrderInComponents {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value
            .get(0)
            .and_then(|v| serde_json::from_value::<OrderInComponentsConfig>(v.clone()).ok())
            .unwrap_or_default();
        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let obj = match node.kind() {
            AstKind::ExportDefaultDeclaration(export) => {
                // export default { ... } or export default defineComponent({ ... })
                match &export.declaration {
                    ExportDefaultDeclarationKind::ObjectExpression(obj) => Some(obj),
                    ExportDefaultDeclarationKind::CallExpression(call) => {
                        if is_vue_component_call(call) {
                            call.arguments.first().and_then(|arg| {
                                if let Argument::ObjectExpression(obj) = arg {
                                    Some(obj)
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            AstKind::CallExpression(call) => {
                if is_vue_component_registration(call) || is_define_options_call(call) {
                    // Get the object argument:
                    // - For defineOptions(): first argument
                    // - For Vue.component('name', {...}) / app.component('name', {...}): second argument
                    // - For component(...): first argument (fallback for destructured usage)
                    let arg = if is_define_options_call(call) {
                        call.arguments.first()
                    } else if call.arguments.len() >= 2 {
                        call.arguments.get(1)
                    } else {
                        call.arguments.first()
                    };
                    arg.and_then(|a| {
                        if let Argument::ObjectExpression(obj) = a { Some(obj) } else { None }
                    })
                } else {
                    None
                }
            }
            AstKind::NewExpression(new_expr) => {
                // new Vue({ ... })
                if is_new_vue(new_expr) {
                    new_expr.arguments.first().and_then(|arg| {
                        if let Argument::ObjectExpression(obj) = arg { Some(obj) } else { None }
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(obj) = obj {
            self.check_order(obj, ctx);
        }
    }
}

impl OrderInComponents {
    fn get_order(&self) -> Vec<Vec<String>> {
        if let Some(ref custom_order) = self.order {
            custom_order
                .iter()
                .map(|el| match el {
                    OrderElement::Single(s) => {
                        if s == "LIFECYCLE_HOOKS" {
                            vec![
                                "beforeCreate".to_string(),
                                "created".to_string(),
                                "beforeMount".to_string(),
                                "mounted".to_string(),
                                "beforeUpdate".to_string(),
                                "updated".to_string(),
                                "activated".to_string(),
                                "deactivated".to_string(),
                                "beforeUnmount".to_string(),
                                "unmounted".to_string(),
                                "beforeDestroy".to_string(),
                                "destroyed".to_string(),
                                "renderTracked".to_string(),
                                "renderTriggered".to_string(),
                                "errorCaptured".to_string(),
                            ]
                        } else if s == "ROUTER_GUARDS" {
                            vec![
                                "beforeRouteEnter".to_string(),
                                "beforeRouteUpdate".to_string(),
                                "beforeRouteLeave".to_string(),
                            ]
                        } else {
                            vec![s.clone()]
                        }
                    }
                    OrderElement::Group(g) => g.clone(),
                })
                .collect()
        } else {
            DEFAULT_ORDER.iter().map(|g| g.iter().map(|s| (*s).to_string()).collect()).collect()
        }
    }

    fn check_order<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        let order = self.get_order();

        // Collect properties with their names and order positions
        let mut properties: Vec<PropertyInfo> = Vec::new();

        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(property) = prop
                && let Some(name) = get_property_name(&property.key)
            {
                let order_position = get_order_position(&name, &order);
                properties.push(PropertyInfo { name, order_position, span: property.span });
            }
        }

        // Check for out-of-order properties
        let mut max_position: Option<usize> = None;
        let mut max_position_name = String::new();

        for prop in &properties {
            if let Some(pos) = prop.order_position {
                if let Some(max) = max_position
                    && pos < max
                {
                    // This property should be before the previous one
                    ctx.diagnostic(order_in_components_diagnostic(
                        &prop.name,
                        &max_position_name,
                        prop.span,
                    ));
                    // Continue checking remaining properties to report all ordering errors
                }
                if max_position.is_none_or(|max| pos >= max) {
                    max_position = Some(pos);
                    max_position_name.clone_from(&prop.name);
                }
            }
        }
    }
}

fn get_order_position(name: &str, order: &[Vec<String>]) -> Option<usize> {
    order.iter().position(|group| group.iter().any(|s| s == name))
}

fn get_property_name(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.to_string()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
        _ => None,
    }
}

fn is_vue_component_call(call: &CallExpression) -> bool {
    match &call.callee {
        Expression::Identifier(id) => {
            matches!(id.name.as_str(), "defineComponent" | "defineNuxtComponent")
        }
        _ => false,
    }
}

fn is_vue_component_registration(call: &CallExpression) -> bool {
    if let Expression::StaticMemberExpression(member) = &call.callee
        && member.property.name == "component"
        && let Expression::Identifier(id) = &member.object
    {
        return matches!(id.name.as_str(), "Vue" | "app");
    }

    // Also handle destructured: `const { component } = Vue; component(...)`.
    //
    // NOTE: This is a heuristic and will treat *any* identifier named `component`
    // as a Vue component registration call, even if it was not actually
    // destructured from `Vue` or `app`. This can introduce false positives for
    // unrelated functions named `component`, but is currently accepted in
    // order to support common destructuring patterns.
    matches!(&call.callee, Expression::Identifier(id) if id.name == "component")
}

fn is_define_options_call(call: &CallExpression) -> bool {
    matches!(&call.callee, Expression::Identifier(id) if id.name == "defineOptions")
}

fn is_new_vue(new_expr: &NewExpression) -> bool {
    matches!(&new_expr.callee, Expression::Identifier(id) if id.name == "Vue")
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
			        export default {
			          name: 'app',
			          props: {
			            propA: Number,
			          },
			          ...a,
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          el,
			          name,
			          parent,
			          functional,
			          delimiters, comments,
			          components, directives, filters,
			          extends: MyComp,
			          mixins,
			          provide, inject,
			          inheritAttrs,
			          model,
			          props, propsData,
			          emits,
			          slots,
			          expose,
			          setup,
			          data,
			          computed,
			          watch,
			          beforeCreate,
			          created,
			          beforeMount,
			          mounted,
			          beforeUpdate,
			          updated,
			          activated,
			          deactivated,
			          beforeUnmount,
			          unmounted,
			          beforeDestroy,
			          destroyed,
			          renderTracked,
			          renderTriggered,
			          errorCaptured,
			          methods,
			          template, render,
			          renderError,
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {}
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default 'example-text'
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.jsx")),
        ), // languageOptions,
        (
            "
			        export default {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          computed: {
			            ...mapStates(['foo'])
			          },
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        Vue.component('smart-list', {
			          name: 'app',
			          components: {},
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          }
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        Vue.component('example')
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        const { component } = Vue;
			        component('smart-list', {
			          name: 'app',
			          components: {},
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          }
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        new Vue({
			          el: '#app',
			          name: 'app',
			          components: {},
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          }
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        new Vue()
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			      <script setup>
			        defineOptions({
			          name: 'Foo',
			          inheritAttrs: true,
			        })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // { "parser": require("vue-eslint-parser") }
    ];

    let fail = vec![
        (
            "
			        export default {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        import { defineNuxtComponent } from '#app'
			        export default defineNuxtComponent({
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          render (h) {
			            return (
			              <span>{ this.msg }</span>
			            )
			          },
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        }
			      ",
            None,
            None,
            Some(PathBuf::from("test.jsx")),
        ), // {        "ecmaVersion": 6,        "sourceType": "module",        "parserOptions": {          "ecmaFeatures": { "jsx": true }        }      },
        (
            "
			        Vue.component('smart-list', {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        app.component('smart-list', {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        const { component } = Vue;
			        component('smart-list', {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        new Vue({
			          name: 'app',
			          el: '#app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ), // { "ecmaVersion": 6 },
        (
            "
			        export default {
			          data() {
			            return {
			              isActive: false,
			            };
			          },
			          methods: {
			            toggleMenu() {
			              this.isActive = !this.isActive;
			            },
			            closeMenu() {
			              this.isActive = false;
			            }
			          },
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          name: 'burger',
			          test: 'ok'
			        };
			      ",
            Some(serde_json::json!([{ "order": ["data", "test", "name"] }])),
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          /** data provider */
			          data() {
			          },
			          /** name of vue component */
			          name: 'burger'
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          /** data provider */
			          data() {
			          }/*test*/,
			          /** name of vue component */
			          name: 'burger'
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "export default {data(){},name:'burger'};",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: obj.fn(),
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: new MyClass(),
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: i++,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: i = 0,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: template`${foo}`,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          [obj.fn()]: 'test',
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: {test: obj.fn()},
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: [obj.fn(), 1],
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: obj.fn().prop,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: delete obj.prop,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: fn() + a + b,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: a ? fn() : null,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          test: `test ${fn()} ${a}`,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          computed: {
			            ...mapStates(['foo'])
			          },
			          data() {
			          },
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          name: 'burger',
			          test: fn(),
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          data() {
			          },
			          testArray: [1, 2, 3, true, false, 'a', 'b', 'c'],
			          testRegExp: /[a-z]*/,
			          testSpreadElement: [...array],
			          testOperator: (!!(a - b + c * d / e % f)) || (a && b),
			          testArrow: (a) => a,
			          testConditional: a ? b : c,
			          testYield: function* () {},
			          testTemplate: `a:${a},b:${b},c:${c}.`,
			          testNullish: a ?? b,
			          testOptionalChaining: a?.b?.c,
			          name: 'burger',
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            r#"
			        <script lang="ts">
			          export default {
			            setup () {},
			            props: {
			              foo: { type: Array as PropType<number[]> },
			            },
			          };
			        </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // {        "parser": require("vue-eslint-parser"),        ...languageOptions,        "parserOptions": {          "parser": { "ts": require.resolve("@typescript-eslint/parser") }        }      },
        (
            "
			      <script setup>
			        defineOptions({
			          inheritAttrs: true,
			          name: 'Foo',
			        })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // { "parser": require("vue-eslint-parser") },
        (
            "
			        export default {
			          setup,
			          slots,
			          expose,
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "
			        export default {
			          slots,
			          setup,
			          expose,
			        };
			      ",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions
    ];

    let _fix = vec![
        (
            "
			        export default {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        }
			      ",
            "
			        export default {
			          name: 'app',
			          props: {
			            propA: Number,
			          },
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			        }
			      ",
            None,
        ),
        (
            "
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        })
			      ",
            "
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          name: 'app',
			          props: {
			            propA: Number,
			          },
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			        })
			      ",
            None,
        ),
        (
            "
			        import { defineNuxtComponent } from '#app'
			        export default defineNuxtComponent({
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        })
			      ",
            "
			        import { defineNuxtComponent } from '#app'
			        export default defineNuxtComponent({
			          name: 'app',
			          props: {
			            propA: Number,
			          },
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			        })
			      ",
            None,
        ),
        (
            "
			        export default {
			          render (h) {
			            return (
			              <span>{ this.msg }</span>
			            )
			          },
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        }
			      ",
            "
			        export default {
			          name: 'app',
			          render (h) {
			            return (
			              <span>{ this.msg }</span>
			            )
			          },
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          props: {
			            propA: Number,
			          },
			        }
			      ",
            None,
        ),
        (
            "
			        Vue.component('smart-list', {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            "
			        Vue.component('smart-list', {
			          name: 'app',
			          components: {},
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          template: '<div></div>'
			        })
			      ",
            None,
        ),
        (
            "
			        app.component('smart-list', {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            "
			        app.component('smart-list', {
			          name: 'app',
			          components: {},
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          template: '<div></div>'
			        })
			      ",
            None,
        ),
        (
            "
			        const { component } = Vue;
			        component('smart-list', {
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            "
			        const { component } = Vue;
			        component('smart-list', {
			          name: 'app',
			          components: {},
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          template: '<div></div>'
			        })
			      ",
            None,
        ),
        (
            "
			        new Vue({
			          name: 'app',
			          el: '#app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            "
			        new Vue({
			          el: '#app',
			          name: 'app',
			          data () {
			            return {
			              msg: 'Welcome to Your Vue.js App'
			            }
			          },
			          components: {},
			          template: '<div></div>'
			        })
			      ",
            None,
        ),
        (
            "
			        export default {
			          data() {
			            return {
			              isActive: false,
			            };
			          },
			          methods: {
			            toggleMenu() {
			              this.isActive = !this.isActive;
			            },
			            closeMenu() {
			              this.isActive = false;
			            }
			          },
			          name: 'burger',
			        };
			      ",
            "
			        export default {
			          name: 'burger',
			          data() {
			            return {
			              isActive: false,
			            };
			          },
			          methods: {
			            toggleMenu() {
			              this.isActive = !this.isActive;
			            },
			            closeMenu() {
			              this.isActive = false;
			            }
			          },
			        };
			      ",
            None,
        ),
        (
            "
			        export default {
			          data() {
			          },
			          name: 'burger',
			          test: 'ok'
			        };
			      ",
            "
			        export default {
			          data() {
			          },
			          test: 'ok',
			          name: 'burger'
			        };
			      ",
            Some(serde_json::json!([{ "order": ["data", "test", "name"] }])),
        ),
        (
            "
			        export default {
			          /** data provider */
			          data() {
			          },
			          /** name of vue component */
			          name: 'burger'
			        };
			      ",
            "
			        export default {
			          /** name of vue component */
			          name: 'burger',
			          /** data provider */
			          data() {
			          }
			        };
			      ",
            None,
        ),
        (
            "
			        export default {
			          /** data provider */
			          data() {
			          }/*test*/,
			          /** name of vue component */
			          name: 'burger'
			        };
			      ",
            "
			        export default {
			          /** name of vue component */
			          name: 'burger',
			          /** data provider */
			          data() {
			          }/*test*/
			        };
			      ",
            None,
        ),
        (
            "export default {data(){},name:'burger'};",
            "export default {name:'burger',data(){}};",
            None,
        ),
        (
            "
			        export default {
			          data() {
			          },
			          name: 'burger',
			          test: fn(),
			        };
			      ",
            "
			        export default {
			          name: 'burger',
			          data() {
			          },
			          test: fn(),
			        };
			      ",
            None,
        ),
        (
            "
			        export default {
			          data() {
			          },
			          testArray: [1, 2, 3, true, false, 'a', 'b', 'c'],
			          testRegExp: /[a-z]*/,
			          testSpreadElement: [...array],
			          testOperator: (!!(a - b + c * d / e % f)) || (a && b),
			          testArrow: (a) => a,
			          testConditional: a ? b : c,
			          testYield: function* () {},
			          testTemplate: `a:${a},b:${b},c:${c}.`,
			          testNullish: a ?? b,
			          testOptionalChaining: a?.b?.c,
			          name: 'burger',
			        };
			      ",
            "
			        export default {
			          name: 'burger',
			          data() {
			          },
			          testArray: [1, 2, 3, true, false, 'a', 'b', 'c'],
			          testRegExp: /[a-z]*/,
			          testSpreadElement: [...array],
			          testOperator: (!!(a - b + c * d / e % f)) || (a && b),
			          testArrow: (a) => a,
			          testConditional: a ? b : c,
			          testYield: function* () {},
			          testTemplate: `a:${a},b:${b},c:${c}.`,
			          testNullish: a ?? b,
			          testOptionalChaining: a?.b?.c,
			        };
			      ",
            None,
        ),
        (
            r#"
			        <script lang="ts">
			          export default {
			            setup () {},
			            props: {
			              foo: { type: Array as PropType<number[]> },
			            },
			          };
			        </script>
			      "#,
            r#"
			        <script lang="ts">
			          export default {
			            props: {
			              foo: { type: Array as PropType<number[]> },
			            },
			            setup () {},
			          };
			        </script>
			      "#,
            None,
        ),
        (
            "
			      <script setup>
			        defineOptions({
			          inheritAttrs: true,
			          name: 'Foo',
			        })
			      </script>
			      ",
            "
			      <script setup>
			        defineOptions({
			          name: 'Foo',
			          inheritAttrs: true,
			        })
			      </script>
			      ",
            None,
        ),
        (
            "
			        export default {
			          setup,
			          slots,
			          expose,
			        };
			      ",
            "
			        export default {
			          slots,
			          setup,
			          expose,
			        };
			      ",
            None,
        ),
        (
            "
			        export default {
			          slots,
			          setup,
			          expose,
			        };
			      ",
            "
			        export default {
			          slots,
			          expose,
			          setup,
			        };
			      ",
            None,
        ),
    ];

    Tester::new(OrderInComponents::NAME, OrderInComponents::PLUGIN, pass, fail)
        // .expect_fix(fix)
        .test_and_snapshot();
}
