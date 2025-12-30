use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ExportDefaultDeclarationKind, Expression, NewExpression,
        ObjectExpression, ObjectPropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{Fix, RuleFix, RuleFixer},
    rule::Rule,
};

/// Information about a property in the component definition
struct PropertyInfo {
    /// The name of the property
    name: String,
    /// The position in the order configuration (None if not in order list)
    order_position: Option<usize>,
    /// The span of the property for error reporting
    span: Span,
    /// Index in the original properties array
    index: usize,
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

#[derive(Debug, Clone)]
pub struct OrderInComponents {
    /// Cached expanded order for performance (avoid repeated allocations)
    order: Vec<Vec<String>>,
}

impl Default for OrderInComponents {
    fn default() -> Self {
        Self { order: expand_default_order() }
    }
}

/// Expand the default order constant to owned Strings
fn expand_default_order() -> Vec<Vec<String>> {
    DEFAULT_ORDER.iter().map(|g| g.iter().map(|s| (*s).to_string()).collect()).collect()
}

/// Expand custom order configuration, handling LIFECYCLE_HOOKS and ROUTER_GUARDS placeholders
fn expand_custom_order(custom_order: &[OrderElement]) -> Vec<Vec<String>> {
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
    fix,
    config = OrderInComponentsConfig,
);

impl Rule for OrderInComponents {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value
            .get(0)
            .and_then(|v| serde_json::from_value::<OrderInComponentsConfig>(v.clone()).ok())
            .unwrap_or_default();

        // Pre-compute and cache the expanded order at configuration time
        let order = if let Some(ref custom_order) = config.order {
            expand_custom_order(custom_order)
        } else {
            expand_default_order()
        };

        Self { order }
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
    fn check_order<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        // Collect properties with their names, order positions, and indices
        // SpreadProperties are tracked but excluded from ordering checks
        let mut properties: Vec<PropertyInfo> = Vec::with_capacity(obj.properties.len());
        let mut has_spread = false;

        for (index, prop) in obj.properties.iter().enumerate() {
            match prop {
                ObjectPropertyKind::ObjectProperty(property) => {
                    if let Some(name) = property.key.static_name() {
                        let order_position = get_order_position(&name, &self.order);
                        properties.push(PropertyInfo {
                            name: name.to_string(),
                            order_position,
                            span: property.span,
                            index,
                        });
                    }
                }
                ObjectPropertyKind::SpreadProperty(_) => {
                    has_spread = true;
                }
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
                    // Find the target property (the first property with order_position >= pos)
                    let target_index = properties
                        .iter()
                        .position(|p| p.order_position.is_some_and(|o| o >= pos))
                        .unwrap_or(0);

                    // Don't offer auto-fix when spread properties exist (like ESLint)
                    // Reordering could change semantics due to spread evaluation order
                    if has_spread {
                        ctx.diagnostic(order_in_components_diagnostic(
                            &prop.name,
                            &max_position_name,
                            prop.span,
                        ));
                    } else {
                        ctx.diagnostic_with_fix(
                            order_in_components_diagnostic(
                                &prop.name,
                                &max_position_name,
                                prop.span,
                            ),
                            |fixer| {
                                self.create_reorder_fix(
                                    fixer.source_text(),
                                    obj,
                                    prop.index,
                                    target_index,
                                    &fixer,
                                )
                            },
                        );
                    }
                }
                if max_position.is_none_or(|max| pos >= max) {
                    max_position = Some(pos);
                    max_position_name.clone_from(&prop.name);
                }
            }
        }
    }

    fn create_reorder_fix<'a>(
        &self,
        source_text: &str,
        obj: &ObjectExpression<'a>,
        from_index: usize,
        to_index: usize,
        fixer: &RuleFixer<'_, 'a>,
    ) -> RuleFix {
        let mut fix = fixer.new_fix_with_capacity(2);

        // Get the property to move
        let Some(ObjectPropertyKind::ObjectProperty(from_prop)) = obj.properties.get(from_index)
        else {
            return fix;
        };

        // Get the target property (where we want to insert before)
        let Some(ObjectPropertyKind::ObjectProperty(to_prop)) = obj.properties.get(to_index) else {
            return fix;
        };

        // Find the comma/brace before the property we're moving
        let text_before_from = &source_text[..from_prop.span.start as usize];
        let comma_before_from = text_before_from.rfind(',');
        let brace_before_from = text_before_from.rfind('{');

        // Find if there's a comma after the property we're moving
        let text_after_from = &source_text[from_prop.span.end as usize..];
        let comma_after_from = text_after_from.find(',');

        // Determine the code start position (after the comma/brace before this property)
        let code_start = if let Some(comma_pos) = comma_before_from {
            comma_pos + 1 // After the comma
        } else if let Some(brace_pos) = brace_before_from {
            brace_pos + 1 // After the opening brace
        } else {
            from_prop.span.start as usize
        };

        // Determine the code end position
        let code_end = if let Some(offset) = comma_after_from {
            from_prop.span.end as usize + offset + 1 // Include the comma after
        } else {
            from_prop.span.end as usize
        };

        // Extract the property code (including leading whitespace)
        let property_code_with_whitespace = &source_text[code_start..code_end];

        // Determine what to delete
        let delete_start: u32;
        let delete_end: u32;

        if comma_after_from.is_some() {
            // Property has a comma after it - delete from code_start to code_end
            delete_start = code_start as u32;
            delete_end = code_end as u32;
        } else if let Some(comma_pos) = comma_before_from {
            // Property is last (no comma after) - delete including the comma before
            delete_start = comma_pos as u32;
            delete_end = from_prop.span.end;
        } else {
            // First and only property or no comma found
            delete_start = from_prop.span.start;
            delete_end = from_prop.span.end;
        }

        fix.push(Fix::delete(Span::new(delete_start, delete_end)));

        // Determine insert position and text
        // Find the comma/brace before the target property
        let text_before_to = &source_text[..to_prop.span.start as usize];
        let comma_before_to = text_before_to.rfind(',');
        let brace_before_to = text_before_to.rfind('{');

        let insert_pos = if let Some(comma_pos) = comma_before_to {
            // Insert after the comma before target
            comma_pos as u32 + 1
        } else if let Some(brace_pos) = brace_before_to {
            // Insert after the opening brace
            brace_pos as u32 + 1
        } else {
            to_prop.span.start
        };

        // Prepare the insert text
        let insert_text = if comma_after_from.is_some() {
            // Already has comma, use as-is
            property_code_with_whitespace.to_string()
        } else {
            // Need to add comma after
            let trimmed = property_code_with_whitespace.trim_end();
            format!("{trimmed},")
        };

        fix.push(Fix::new(insert_text, Span::sized(insert_pos, 0)));

        fix.with_message(format!(
            "Move `{}` to correct position",
            from_prop.key.static_name().unwrap_or_else(|| "<property>".into())
        ))
    }
}

fn get_order_position(name: &str, order: &[Vec<String>]) -> Option<usize> {
    order.iter().position(|group| group.iter().any(|s| s == name))
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

    let fix = vec![
        // Simple case: name should be before data
        (
            "export default {data(){},name:'burger'};",
            "export default {name:'burger',data(){}};",
            None,
        ),
        // Multi-line: props should be before data
        (
            "
export default {
  name: 'app',
  data () {
    return {
      msg: 'Welcome'
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
      msg: 'Welcome'
    }
  },
}
",
            None,
        ),
        // Vue.component registration
        (
            "
Vue.component('smart-list', {
  name: 'app',
  data () {
    return { msg: 'Hello' }
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
    return { msg: 'Hello' }
  },
  template: '<div></div>'
})
",
            None,
        ),
        // new Vue
        (
            "
new Vue({
  name: 'app',
  el: '#app',
  data () {
    return { msg: 'Hello' }
  },
})
",
            "
new Vue({
  el: '#app',
  name: 'app',
  data () {
    return { msg: 'Hello' }
  },
})
",
            None,
        ),
        // name should be before methods (last property)
        (
            "
export default {
  data() {
    return { isActive: false };
  },
  methods: {
    toggle() { this.isActive = !this.isActive; }
  },
  name: 'burger',
};
",
            "
export default {
  name: 'burger',
  data() {
    return { isActive: false };
  },
  methods: {
    toggle() { this.isActive = !this.isActive; }
  },
};
",
            None,
        ),
    ];

    Tester::new(OrderInComponents::NAME, OrderInComponents::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
