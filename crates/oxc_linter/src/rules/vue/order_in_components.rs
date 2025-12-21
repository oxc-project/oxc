use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey},
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
                    oxc_ast::ast::ExportDefaultDeclarationKind::ObjectExpression(obj) => Some(obj),
                    oxc_ast::ast::ExportDefaultDeclarationKind::CallExpression(call) => {
                        if is_vue_component_call(call) {
                            call.arguments.first().and_then(|arg| {
                                if let oxc_ast::ast::Argument::ObjectExpression(obj) = arg {
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
                // Vue.component('name', { ... }) or new Vue({ ... }) or defineOptions({ ... })
                if is_vue_component_registration(call) || is_define_options_call(call) {
                    // Get the object argument (second arg for component, first for new Vue/defineOptions)
                    let arg = if is_define_options_call(call) {
                        call.arguments.first()
                    } else if call.arguments.len() >= 2 {
                        call.arguments.get(1)
                    } else {
                        call.arguments.first()
                    };
                    arg.and_then(|a| {
                        if let oxc_ast::ast::Argument::ObjectExpression(obj) = a {
                            Some(obj)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            }
            AstKind::NewExpression(new_expr) => {
                // new Vue({ ... })
                if is_new_vue(new_expr) {
                    new_expr.arguments.first().and_then(|arg| {
                        if let oxc_ast::ast::Argument::ObjectExpression(obj) = arg {
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
                if max_position.is_none() || pos >= max_position.unwrap_or(0) {
                    max_position = Some(pos);
                    max_position_name.clone_from(&prop.name);
                }
            }
        }
    }
}

fn get_order_position(name: &str, order: &[Vec<String>]) -> Option<usize> {
    for (index, group) in order.iter().enumerate() {
        if group.iter().any(|s| s == name) {
            return Some(index);
        }
    }
    None
}

fn get_property_name(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.to_string()),
        PropertyKey::StringLiteral(lit) => Some(lit.value.to_string()),
        _ => None,
    }
}

fn is_vue_component_call(call: &oxc_ast::ast::CallExpression) -> bool {
    match &call.callee {
        Expression::Identifier(id) => {
            matches!(id.name.as_str(), "defineComponent" | "defineNuxtComponent")
        }
        _ => false,
    }
}

fn is_vue_component_registration(call: &oxc_ast::ast::CallExpression) -> bool {
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
    if let Expression::Identifier(id) = &call.callee {
        return id.name == "component";
    }

    false
}

fn is_define_options_call(call: &oxc_ast::ast::CallExpression) -> bool {
    if let Expression::Identifier(id) = &call.callee {
        return id.name == "defineOptions";
    }
    false
}

fn is_new_vue(new_expr: &oxc_ast::ast::NewExpression) -> bool {
    if let Expression::Identifier(id) = &new_expr.callee {
        return id.name == "Vue";
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Correct order: name before data
        ("export default { name: 'app', data() { return {}; } }", None),
        // Empty object
        ("export default {}", None),
        // Not an object
        ("export default 'example-text'", None),
        // Full correct order
        ("export default { name: 'app', props: {}, data() {}, computed: {}, methods: {} }", None),
        // Props before data
        ("export default { props: ['msg'], data() { return {} } }", None),
        // Lifecycle hooks in correct order
        (
            "export default { name: 'a', data() {}, beforeCreate() {}, created() {}, mounted() {} }",
            None,
        ),
        // Unknown properties are allowed anywhere
        ("export default { name: 'a', unknownProp: true, data() {} }", None),
        // defineComponent
        (
            "import { defineComponent } from 'vue'; export default defineComponent({ name: 'app', props: {}, data() {} })",
            None,
        ),
        // Vue.component
        ("Vue.component('name', { name: 'app', data() {} })", None),
        // new Vue
        ("new Vue({ el: '#app', data: {} })", None),
        // Methods before render (correct)
        ("export default { methods: {}, render() {} }", None),
        // Computed before watch (correct)
        ("export default { computed: {}, watch: {} }", None),
        // Custom order config
        (
            "export default { data() {}, name: 'app' }",
            Some(serde_json::json!([{ "order": ["data", "name"] }])),
        ),
        // LIFECYCLE_HOOKS placeholder in custom order
        (
            "export default { data() {}, beforeCreate() {}, created() {} }",
            Some(serde_json::json!([{ "order": ["data", "LIFECYCLE_HOOKS"] }])),
        ),
    ];

    let fail = vec![
        // data before name (wrong order)
        ("export default { data() {}, name: 'burger' }", None),
        // data before props (wrong order)
        ("export default { name: 'a', data() {}, props: {} }", None),
        // methods before computed
        ("export default { methods: {}, computed: {} }", None),
        // render before methods
        ("export default { render() {}, methods: {} }", None),
        // mounted before data
        ("export default { mounted() {}, data() {} }", None),
        // watch before computed
        ("export default { watch: {}, computed: {} }", None),
        // defineComponent with wrong order
        (
            "import { defineComponent } from 'vue'; export default defineComponent({ data() {}, name: 'app' })",
            None,
        ),
        // props before name
        ("export default { props: {}, name: 'a' }", None),
    ];

    Tester::new(OrderInComponents::NAME, OrderInComponents::PLUGIN, pass, fail).test_and_snapshot();
}
