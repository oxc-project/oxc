use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind, Comment,
    ast::{
        ArrayExpressionElement, ChainElement, Expression, ObjectExpression, ObjectProperty,
        ObjectPropertyKind, UnaryOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::is_vue_component_options_object,
};

/// Lifecycle hooks expanded in place of the `LIFECYCLE_HOOKS` placeholder.
const LIFECYCLE_HOOKS: &[&str] = &[
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
];

/// Router guards expanded in place of the `ROUTER_GUARDS` placeholder.
const ROUTER_GUARDS: &[&str] = &["beforeRouteEnter", "beforeRouteUpdate", "beforeRouteLeave"];

/// Default recommended order. Each inner slice is a group of names that share
/// the same rank; `LIFECYCLE_HOOKS` / `ROUTER_GUARDS` are placeholders expanded
/// at that rank.
const DEFAULT_ORDER: &[&[&str]] = &[
    &["el"],
    &["name"],
    &["key"],
    &["parent"],
    &["functional"],
    &["delimiters", "comments"],
    &["components", "directives", "filters"],
    &["extends"],
    &["mixins"],
    &["provide", "inject"],
    &["ROUTER_GUARDS"],
    &["layout"],
    &["middleware"],
    &["validate"],
    &["scrollToTop"],
    &["transition"],
    &["loading"],
    &["inheritAttrs"],
    &["model"],
    &["props", "propsData"],
    &["emits"],
    &["slots"],
    &["expose"],
    &["setup"],
    &["asyncData"],
    &["data"],
    &["fetch"],
    &["head"],
    &["computed"],
    &["watch"],
    &["watchQuery"],
    &["LIFECYCLE_HOOKS"],
    &["methods"],
    &["template", "render"],
    &["renderError"],
];

fn order_diagnostic(span: Span, name: &str, above: &str, line: usize) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The \"{name}\" property should be above the \"{above}\" property on line {line}."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct OrderInComponentsConfig {
    /// Custom property order. Each entry is a single property name or a group of
    /// names sharing the same rank. Defaults to Vue's recommended order.
    order: Option<Vec<OrderEntry>>,
}

/// One entry in the `order` option: either a single name or a group of names
/// that share the same position.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum OrderEntry {
    Single(String),
    Group(Vec<String>),
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OrderInComponents(Box<OrderInComponentsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent order for the properties of a component definition,
    /// following the order recommended by the Vue style guide.
    ///
    /// ### Why is this bad?
    ///
    /// A predictable property order makes components easier to scan and review.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   data() { return {}; },
    ///   name: 'app',
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   name: 'app',
    ///   data() { return {}; },
    /// }
    /// </script>
    /// ```
    OrderInComponents,
    vue,
    style,
    conditional_fix_suggestion,
    config = OrderInComponents,
    version = "next",
);

impl Rule for OrderInComponents {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Options API: `export default {…}`, `defineComponent({…})`,
            // `Vue.component('x', {…})`, `Vue.extend({…})`, `new Vue({…})`.
            AstKind::ObjectExpression(obj) => {
                if is_vue_component_options_object(node, ctx) {
                    self.check_order(obj, ctx);
                }
            }
            // `<script setup>`: `defineOptions({…})`.
            AstKind::CallExpression(call) => {
                if ctx.frameworks_options() != FrameworkOptions::VueSetup {
                    return;
                }
                if !call
                    .callee
                    .get_identifier_reference()
                    .is_some_and(|i| i.name == "defineOptions")
                {
                    return;
                }
                if let Some(Expression::ObjectExpression(obj)) = call
                    .arguments
                    .first()
                    .and_then(|a| a.as_expression())
                    .map(Expression::get_inner_expression)
                {
                    self.check_order(obj, ctx);
                }
            }
            _ => {}
        }
    }
}

impl OrderInComponents {
    fn order_map(&self) -> FxHashMap<String, usize> {
        let mut map = FxHashMap::default();
        let insert = |map: &mut FxHashMap<String, usize>, name: &str, rank: usize| match name {
            "LIFECYCLE_HOOKS" => {
                for hook in LIFECYCLE_HOOKS {
                    map.insert((*hook).to_string(), rank);
                }
            }
            "ROUTER_GUARDS" => {
                for guard in ROUTER_GUARDS {
                    map.insert((*guard).to_string(), rank);
                }
            }
            _ => {
                map.insert(name.to_string(), rank);
            }
        };
        match &self.0.order {
            Some(order) => {
                for (rank, entry) in order.iter().enumerate() {
                    match entry {
                        OrderEntry::Single(name) => insert(&mut map, name, rank),
                        OrderEntry::Group(names) => {
                            for name in names {
                                insert(&mut map, name, rank);
                            }
                        }
                    }
                }
            }
            None => {
                for (rank, group) in DEFAULT_ORDER.iter().enumerate() {
                    for name in *group {
                        insert(&mut map, name, rank);
                    }
                }
            }
        }
        map
    }

    fn check_order<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        let order_map = self.order_map();
        let position = |name: &str| order_map.get(name).copied();

        // Keep the original index so the side-effect range can include spreads.
        let properties: Vec<(usize, &ObjectProperty<'a>, String)> = obj
            .properties
            .iter()
            .enumerate()
            .filter_map(|(idx, prop)| match prop {
                ObjectPropertyKind::ObjectProperty(p) => {
                    // Mirrors upstream: a static name, else the bare identifier of a
                    // computed key like `[name]`, else empty.
                    let name = p.key.static_name().map_or_else(
                        || match p.key.as_expression().map(Expression::get_inner_expression) {
                            Some(Expression::Identifier(ident)) => ident.name.to_string(),
                            _ => String::new(),
                        },
                        std::borrow::Cow::into_owned,
                    );
                    Some((idx, p.as_ref(), name))
                }
                ObjectPropertyKind::SpreadProperty(_) => None,
            })
            .collect();

        for (list_idx, (orig_idx, prop, name)) in properties.iter().enumerate() {
            let Some(order_pos) = position(name) else { continue };

            let mut unordered: Vec<&(usize, &ObjectProperty<'a>, String)> = properties[..list_idx]
                .iter()
                .filter(|(_, _, n)| position(n).is_some_and(|p| p > order_pos))
                .collect();
            unordered.sort_by_key(|(_, _, n)| position(n).unwrap_or(0));

            let Some(first) = unordered.first() else { continue };
            let (first_idx, first_node, first_name) = *first;
            let line = source_line(ctx.source_text(), first_node.span().start);

            // A property between the target slot and the current one whose value
            // may run side effects makes auto-reordering unsafe; downgrade to a
            // manual suggestion in that case.
            let has_side_effects = obj.properties[*first_idx..=*orig_idx]
                .iter()
                .any(|p| !property_is_side_effect_free(p));

            // Upstream always reports the `order` message; the side-effect variant
            // is only used as the description of the manual suggestion.
            let diagnostic = order_diagnostic(prop.span(), name, first_name, line);

            let prop_span = prop.span();
            let target_span = first_node.span();
            let message = if has_side_effects {
                format!(
                    "Manually move \"{name}\" property above \"{first_name}\" property on line {line} (might break side effects)."
                )
            } else {
                format!("Move \"{name}\" property above \"{first_name}\"")
            };
            let make_fix = move |fixer: crate::fixer::RuleFixer<'_, 'a>| {
                build_move_fix(fixer, ctx, prop_span, target_span, &message)
            };

            if has_side_effects {
                ctx.diagnostic_with_suggestion(diagnostic, make_fix);
            } else {
                ctx.diagnostic_with_fix(diagnostic, make_fix);
            }
        }
    }
}

/// 1-based line number of `offset` within `source`.
fn source_line(source: &str, offset: u32) -> usize {
    source[..offset as usize].bytes().filter(|&b| b == b'\n').count() + 1
}

/// Move the property at `prop_span` to just before `target_span`, mirroring the
/// upstream fixer's comma handling.
fn build_move_fix<'a>(
    fixer: crate::fixer::RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    prop_span: Span,
    target_span: Span,
    message: &str,
) -> crate::fixer::RuleFix {
    let src = ctx.source_text();
    let comments = ctx.comments();
    let code_start = delimiter_before(src, comments, prop_span.start) + 1;
    let after_comma = comma_after(src, comments, prop_span.end);
    let has_after_comma = after_comma.is_some();
    let code_end = after_comma.unwrap_or(prop_span.end);
    let remove_start =
        if has_after_comma { code_start } else { delimiter_before(src, comments, prop_span.start) };

    let mut property_code = src[code_start as usize..code_end as usize].to_string();
    if !has_after_comma {
        property_code.push(',');
    }
    let insert_pos = delimiter_before(src, comments, target_span.start) + 1;

    let multi = fixer.for_multifix();
    let mut fixes = multi.new_fix_with_capacity(2);
    fixes.push(fixer.delete_range(Span::new(remove_start, code_end)));
    fixes.push(fixer.insert_text_after(&Span::empty(insert_pos), property_code));
    fixes.with_message(message.to_string())
}

/// Byte offset of the `{` or `,` immediately before `start`, skipping whitespace
/// and comments (so a property's leading comment moves with it, mirroring
/// upstream `getTokenBefore`). Comments come from the parser, so `//` and `*/`
/// inside string literals are not mistaken for comment delimiters.
#[expect(clippy::cast_possible_truncation)] // byte offsets within a source file fit in u32
fn delimiter_before(src: &str, comments: &[Comment], start: u32) -> u32 {
    let bytes = src.as_bytes();
    let mut i = start as usize;
    loop {
        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }
        if let Some(comment) = comments.iter().find(|c| c.span.end as usize == i) {
            i = comment.span.start as usize;
            continue;
        }
        if i == 0 {
            return 0;
        }
        return i as u32 - 1;
    }
}

/// Byte offset just after the `,` that follows `end` (skipping whitespace and
/// comments), or `None` when the next token is not a comma.
#[expect(clippy::cast_possible_truncation)] // byte offsets within a source file fit in u32
fn comma_after(src: &str, comments: &[Comment], end: u32) -> Option<u32> {
    let bytes = src.as_bytes();
    let mut i = end as usize;
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if let Some(comment) = comments.iter().find(|c| c.span.start as usize == i) {
            i = comment.span.end as usize;
            continue;
        }
        if i < bytes.len() && bytes[i] == b',' {
            return Some(i as u32 + 1);
        }
        return None;
    }
}

fn property_is_side_effect_free(prop: &ObjectPropertyKind) -> bool {
    match prop {
        ObjectPropertyKind::ObjectProperty(p) => {
            // A computed key such as `[obj.fn()]` can run side effects too, so it
            // is part of the property the same way the value is.
            p.key.as_expression().is_none_or(expr_is_side_effect_free)
                && expr_is_side_effect_free(&p.value)
        }
        ObjectPropertyKind::SpreadProperty(s) => expr_is_side_effect_free(&s.argument),
    }
}

/// A link in an optional chain is safe unless it (or anything inside it) is a
/// call. Mirrors upstream traversing the whole `ChainExpression` for calls.
fn chain_element_is_side_effect_free(element: &ChainElement) -> bool {
    match element {
        ChainElement::CallExpression(_) => false,
        ChainElement::TSNonNullExpression(e) => expr_is_side_effect_free(&e.expression),
        ChainElement::StaticMemberExpression(m) => expr_is_side_effect_free(&m.object),
        ChainElement::ComputedMemberExpression(m) => {
            expr_is_side_effect_free(&m.object) && expr_is_side_effect_free(&m.expression)
        }
        ChainElement::PrivateFieldExpression(m) => expr_is_side_effect_free(&m.object),
    }
}

/// Mirrors upstream `isNotSideEffectsNode`: a value is safe to move when it is
/// free of side effects. Functions are leaves (not invoked here), so their
/// bodies are not inspected.
fn expr_is_side_effect_free(expr: &Expression) -> bool {
    match expr {
        // Leaves / non-invoked nodes. `as` casts are skipped wholesale upstream.
        Expression::Identifier(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::TSAsExpression(_) => true,
        Expression::TemplateLiteral(t) => t.expressions.iter().all(expr_is_side_effect_free),
        Expression::ObjectExpression(o) => o.properties.iter().all(property_is_side_effect_free),
        Expression::ArrayExpression(a) => a.elements.iter().all(|el| match el {
            // `[...load()]` runs the spread argument, like `{...load()}` does.
            ArrayExpressionElement::SpreadElement(s) => expr_is_side_effect_free(&s.argument),
            _ => el.as_expression().is_none_or(expr_is_side_effect_free),
        }),
        Expression::UnaryExpression(u) => {
            matches!(
                u.operator,
                UnaryOperator::LogicalNot
                    | UnaryOperator::BitwiseNot
                    | UnaryOperator::UnaryPlus
                    | UnaryOperator::UnaryNegation
                    | UnaryOperator::Typeof
            ) && expr_is_side_effect_free(&u.argument)
        }
        Expression::BinaryExpression(b) => {
            expr_is_side_effect_free(&b.left) && expr_is_side_effect_free(&b.right)
        }
        Expression::LogicalExpression(l) => {
            expr_is_side_effect_free(&l.left) && expr_is_side_effect_free(&l.right)
        }
        Expression::ConditionalExpression(c) => {
            expr_is_side_effect_free(&c.test)
                && expr_is_side_effect_free(&c.consequent)
                && expr_is_side_effect_free(&c.alternate)
        }
        Expression::StaticMemberExpression(m) => expr_is_side_effect_free(&m.object),
        Expression::ComputedMemberExpression(m) => {
            expr_is_side_effect_free(&m.object) && expr_is_side_effect_free(&m.expression)
        }
        // `a?.b` is safe, but a call anywhere in the chain (`a?.b()`, `a?.b().c`)
        // is not.
        Expression::ChainExpression(c) => chain_element_is_side_effect_free(&c.expression),
        Expression::ParenthesizedExpression(p) => expr_is_side_effect_free(&p.expression),
        _ => false,
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "<script>\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {}
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default 'example-text'
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      name: 'app',
                      data () {
                        return {
                          msg: 'Welcome to Your Vue.js App'
                        }
                      },
                    }
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\n
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
                  \n</script>",
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
        ), // { "parser": vueEslintParser }
    ];

    let fail = vec![
        (
            "<script>\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script>\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 6, "sourceType": "module", "parserOptions": { "ecmaFeatures": { "jsx": true } } },
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
            "<script>\n
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
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      name: 'burger',
                      test: 'ok'
                    };
                  \n</script>",
            Some(serde_json::json!([{ "order": ["data", "test", "name"] }])),
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      /** data provider */
                      data() {
                      },
                      /** name of vue component */
                      name: 'burger'
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      /** data provider */
                      data() {
                      }/*test*/,
                      /** name of vue component */
                      name: 'burger'
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\nexport default {data(){},name:'burger'};\n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      // data provider
                      data() {
                      },
                      // name of vue component
                      name: 'burger'
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: obj.fn(),
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: new MyClass(),
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: i++,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: i = 0,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: template`${foo}`,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      [obj.fn()]: 'test',
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: {test: obj.fn()},
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: [obj.fn(), 1],
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: [...load()],
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      [name]: 'x',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: obj.fn().prop,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: delete obj.prop,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: fn() + a + b,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: a ? fn() : null,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: a?.b().c,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      test: `test ${fn()} ${a}`,
                      name: 'burger',
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      computed: {
                        ...mapStates(['foo'])
                      },
                      data() {
                      },
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      name: 'burger',
                      test: fn(),
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
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
                  \n</script>",
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
        ), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": { "ts": require.resolve("@typescript-eslint/parser") } } },
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
        ), // { "parser": vueEslintParser },
        (
            "<script>\n
                    export default {
                      setup,
                      slots,
                      expose,
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions,
        (
            "<script>\n
                    export default {
                      slots,
                      setup,
                      expose,
                    };
                  \n</script>",
            None,
            None,
            Some(PathBuf::from("example.vue")),
        ), // languageOptions
    ];

    let fix = vec![
        (
            "<script>\n
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
                  \n</script>",
            "<script>\n
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
                  \n</script>",
            None,
        ),
        (
            "<script>\n
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
                  \n</script>",
            "<script>\n
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
                  \n</script>",
            None,
        ),
        (
            "<script>\n
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
                  \n</script>",
            "<script>\n
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
                  \n</script>",
            None,
        ),
        (
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            None,
        ),
        (
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            None,
        ),
        (
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            None,
        ),
        (
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            None,
        ),
        (
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            "<script lang=\"tsx\">\n
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
                  \n</script>",
            None,
        ),
        (
            "<script>\n
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
                  \n</script>",
            "<script>\n
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
                  \n</script>",
            None,
        ),
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      name: 'burger',
                      test: 'ok'
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      data() {
                      },
                      test: 'ok',
                      name: 'burger'
                    };
                  \n</script>",
            Some(serde_json::json!([{ "order": ["data", "test", "name"] }])),
        ),
        (
            "<script>\n
                    export default {
                      /** data provider */
                      data() {
                      },
                      /** name of vue component */
                      name: 'burger'
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      /** name of vue component */
                      name: 'burger',
                      /** data provider */
                      data() {
                      }
                    };
                  \n</script>",
            None,
        ),
        (
            "<script>\n
                    export default {
                      /** data provider */
                      data() {
                      }/*test*/,
                      /** name of vue component */
                      name: 'burger'
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      /** name of vue component */
                      name: 'burger',
                      /** data provider */
                      data() {
                      }/*test*/
                    };
                  \n</script>",
            None,
        ),
        (
            "<script>\nexport default {data(){},name:'burger'};\n</script>",
            "<script>\nexport default {name:'burger',data(){}};\n</script>",
            None,
        ),
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      url: 'https://example.com',
                      name: 'burger',
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      name: 'burger',
                      data() {
                      },
                      url: 'https://example.com',
                    };
                  \n</script>",
            None,
        ),
        (
            "<script>\n
                    export default {
                      // data provider
                      data() {
                      },
                      // name of vue component
                      name: 'burger'
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      // name of vue component
                      name: 'burger',
                      // data provider
                      data() {
                      }
                    };
                  \n</script>",
            None,
        ),
        (
            "<script>\n
                    export default {
                      data() {
                      },
                      name: 'burger',
                      test: fn(),
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      name: 'burger',
                      data() {
                      },
                      test: fn(),
                    };
                  \n</script>",
            None,
        ),
        (
            "<script>\n
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
                  \n</script>",
            "<script>\n
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
                  \n</script>",
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
            "<script>\n
                    export default {
                      setup,
                      slots,
                      expose,
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      slots,
                      setup,
                      expose,
                    };
                  \n</script>",
            None,
        ),
        (
            "<script>\n
                    export default {
                      slots,
                      setup,
                      expose,
                    };
                  \n</script>",
            "<script>\n
                    export default {
                      slots,
                      expose,
                      setup,
                    };
                  \n</script>",
            None,
        ),
    ];

    Tester::new(OrderInComponents::NAME, OrderInComponents::PLUGIN, pass, fail)
        .change_rule_path_extension("vue")
        .expect_fix(fix)
        .test_and_snapshot();
}
