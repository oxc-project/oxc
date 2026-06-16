use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{CallExpression, ChainElement, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    module_record::ImportImportName,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_this_object, is_vue_component_options_object},
};

#[derive(Debug, Clone, Copy)]
enum AsyncKind {
    AsyncFunction,
    Await,
    NewPromise,
    Asynchronous,
    Timed,
}

impl AsyncKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::AsyncFunction => "async function declaration",
            Self::Await => "await operator",
            Self::NewPromise => "Promise object",
            Self::Asynchronous => "asynchronous action",
            Self::Timed => "timed function",
        }
    }
}

fn unexpected_in_property(span: Span, kind: AsyncKind, key: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected {} in \"{}\" computed property.", kind.as_str(), key))
        .with_label(span)
}

fn unexpected_in_function(span: Span, kind: AsyncKind) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected {} in computed function.", kind.as_str()))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoAsyncInComputedPropertiesConfig {
    /// Names of identifiers whose member-call chains (`.then` / `.catch` / `.finally`)
    /// should be ignored. Useful for libraries like Zod where `.catch(default)` is
    /// a builder API, not a Promise method.
    ignored_object_names: FxHashSet<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoAsyncInComputedProperties(Box<NoAsyncInComputedPropertiesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow asynchronous actions in computed properties.
    ///
    /// ### Why is this bad?
    ///
    /// Asynchronous actions inside computed properties may lead to an unexpected
    /// behavior. A computed property's value should be a synchronous function of
    /// its dependencies.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     pro() {
    ///       return Promise.all([new Promise((resolve, reject) => {})])
    ///     },
    ///     foo: async function () {
    ///       return await someFunc()
    ///     },
    ///     bar() {
    ///       return fetch(url).then(response => {})
    ///     },
    ///     tim() {
    ///       setTimeout(() => { }, 0)
    ///     },
    ///     inst() {
    ///       return new Promise((resolve, reject) => {})
    ///     },
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   computed: {
    ///     foo() {
    ///       return this.bar
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    NoAsyncInComputedProperties,
    vue,
    correctness,
    none,
    config = NoAsyncInComputedProperties,
    version = "1.71.0",
    short_description = "Disallow asynchronous actions in computed properties.",
);

impl Rule for NoAsyncInComputedProperties {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func) if func.r#async => {
                if let Some(c) = get_computed_getter_context(node, ctx) {
                    report(node.span(), &c, AsyncKind::AsyncFunction, ctx);
                }
            }
            AstKind::ArrowFunctionExpression(arrow) if arrow.r#async => {
                if let Some(c) = get_computed_getter_context(node, ctx) {
                    report(node.span(), &c, AsyncKind::AsyncFunction, ctx);
                }
            }
            AstKind::AwaitExpression(_) => {
                if let Some(c) = find_computed_context(node, ctx) {
                    report(node.span(), &c, AsyncKind::Await, ctx);
                }
            }
            AstKind::NewExpression(new_expr) => {
                if let Expression::Identifier(id) = new_expr.callee.get_inner_expression()
                    && id.name == "Promise"
                    && let Some(c) = find_computed_context(node, ctx)
                {
                    report(node.span(), &c, AsyncKind::NewPromise, ctx);
                }
            }
            AstKind::CallExpression(call) => {
                let kind = if is_promise_method_call(call, &self.0.ignored_object_names) {
                    AsyncKind::Asynchronous
                } else if is_timed_function_call(call) {
                    AsyncKind::Timed
                } else if is_next_tick_call(call, ctx) {
                    AsyncKind::Asynchronous
                } else {
                    return;
                };
                if let Some(c) = find_computed_context(node, ctx) {
                    report(node.span(), &c, kind, ctx);
                }
            }
            _ => {}
        }
    }
}

fn report(span: Span, ctx_kind: &ComputedContext, kind: AsyncKind, ctx: &LintContext<'_>) {
    match ctx_kind {
        ComputedContext::OptionsApi(key) => {
            let key = key.as_deref().unwrap_or("Unknown");
            ctx.diagnostic(unexpected_in_property(span, kind, key));
        }
        ComputedContext::CompositionApi => {
            ctx.diagnostic(unexpected_in_function(span, kind));
        }
    }
}

// Describe where a computed getter lives. Mirrors `no_side_effects_in_computed_properties`,
// but only the Options API variant needs to carry the property key — `no-async` does not
// inspect the getter body for setup variables.
enum ComputedContext {
    OptionsApi(Option<String>),
    CompositionApi,
}

/// Find the computed getter context for `node` by walking up to the nearest enclosing function
/// and checking if that function is a computed getter.
fn find_computed_context(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<ComputedContext> {
    let nodes = ctx.nodes();
    let mut current = nodes.parent_node(node.id());
    loop {
        match current.kind() {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                return get_computed_getter_context(current, ctx);
            }
            AstKind::Program(_) => return None,
            _ => {
                current = nodes.parent_node(current.id());
            }
        }
    }
}

/// Given a function node, return Some(ComputedContext) if it is a computed getter.
fn get_computed_getter_context(
    fn_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> Option<ComputedContext> {
    let nodes = ctx.nodes();
    let parent = nodes.parent_node(fn_node.id());

    match parent.kind() {
        AstKind::CallExpression(call) if is_vue_computed_call(call, ctx) => {
            return Some(ComputedContext::CompositionApi);
        }
        AstKind::ObjectProperty(prop) => {
            let grandparent = nodes.parent_node(parent.id());
            let AstKind::ObjectExpression(_) = grandparent.kind() else { return None };
            let great = nodes.parent_node(grandparent.id());

            match great.kind() {
                // Case A: `computed: { key() {} }` or `computed: { key: function() {} }`
                AstKind::ObjectProperty(outer)
                    if outer.key.is_specific_static_name("computed")
                        && matches!(fn_node.kind(), AstKind::Function(_)) =>
                {
                    let vue_options = nodes.parent_node(great.id());
                    if is_vue_component_options_object(vue_options, ctx) {
                        let key = prop.key.static_name().map(|s| s.to_string());
                        return Some(ComputedContext::OptionsApi(key));
                    }
                }

                // Case B: `computed: { key: { get() {} } }`
                AstKind::ObjectProperty(key_prop)
                    if prop.key.is_specific_static_name("get")
                        && matches!(fn_node.kind(), AstKind::Function(_)) =>
                {
                    let key_obj_expr = nodes.parent_node(great.id());
                    let AstKind::ObjectExpression(_) = key_obj_expr.kind() else { return None };
                    let computed_prop_node = nodes.parent_node(key_obj_expr.id());
                    if let AstKind::ObjectProperty(cp) = computed_prop_node.kind()
                        && cp.key.is_specific_static_name("computed")
                    {
                        let vue_options = nodes.parent_node(computed_prop_node.id());
                        if is_vue_component_options_object(vue_options, ctx) {
                            let key = key_prop.key.static_name().map(|s| s.to_string());
                            return Some(ComputedContext::OptionsApi(key));
                        }
                    }
                }

                // Case C: Composition API with `computed({ get() {}, set() {} })`
                AstKind::CallExpression(call)
                    if prop.key.is_specific_static_name("get")
                        && is_vue_computed_call(call, ctx) =>
                {
                    return Some(ComputedContext::CompositionApi);
                }

                _ => {}
            }
        }

        _ => {}
    }

    None
}

/// Check if a `computed(...)` call uses the `computed` export from `'vue'`,
/// `'@vue/composition-api'`, or `'#imports'` (Nuxt), including aliases.
fn is_vue_computed_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Expression::Identifier(ident) = call.callee.get_inner_expression() else {
        return false;
    };
    let scoping = ctx.scoping();
    let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    ctx.module_record().import_entries.iter().any(|entry| {
        if !matches!(entry.module_request.name(), "vue" | "@vue/composition-api" | "#imports") {
            return false;
        }
        let ImportImportName::Name(name_span) = &entry.import_name else { return false };
        if name_span.name() != "computed" {
            return false;
        }
        scoping.get_root_binding(entry.local_name.name().into()) == Some(symbol_id)
    })
}

const PROMISE_FUNCTIONS: &[&str] = &["then", "catch", "finally"];
const PROMISE_METHODS: &[&str] = &["all", "race", "reject", "resolve"];
const TIMED_FUNCTIONS: &[&str] =
    &["setTimeout", "setInterval", "setImmediate", "requestAnimationFrame"];

/// Static-member call info extracted from a `CallExpression.callee`, transparently
/// unwrapping parentheses and an optional outer `ChainExpression`. Only static
/// member-expressions are surfaced; computed/private members are reported as
/// `None` to match upstream `getStaticPropertyName`-based gating.
fn callee_static_member<'a, 'b>(
    callee: &'b Expression<'a>,
) -> Option<(&'b str, &'b Expression<'a>)> {
    let inner = callee.get_inner_expression();
    let member = match inner {
        Expression::StaticMemberExpression(m) => m.as_ref(),
        Expression::ChainExpression(c) => match &c.expression {
            ChainElement::StaticMemberExpression(m) => m.as_ref(),
            _ => return None,
        },
        _ => return None,
    };
    Some((member.property.name.as_str(), &member.object))
}

fn is_promise_method_call(
    call: &CallExpression<'_>,
    ignored_object_names: &FxHashSet<String>,
) -> bool {
    let Some((name, object)) = callee_static_member(&call.callee) else { return false };

    let is_promise_static = PROMISE_METHODS.contains(&name)
        && matches!(object.get_inner_expression(), Expression::Identifier(id) if id.name == "Promise");
    let is_thenable = PROMISE_FUNCTIONS.contains(&name);

    if !is_promise_static && !is_thenable {
        return false;
    }

    if let Some(root) = get_root_object_name(object)
        && ignored_object_names.contains(root)
    {
        return false;
    }
    true
}

fn is_timed_function_call(call: &CallExpression<'_>) -> bool {
    if call.arguments.is_empty() {
        return false;
    }
    let inner = call.callee.get_inner_expression();
    // `setTimeout(...)` / `setTimeout?.(...)` — bare identifier (the optional-chain
    // wrapper sits on the *outer* CallExpression).
    if let Expression::Identifier(id) = inner {
        return TIMED_FUNCTIONS.contains(&id.name.as_str());
    }
    // `window.setTimeout(...)` / `window?.setTimeout?.(...)` / `(window?.setTimeout)?.(...)`.
    if let Some((name, object)) = callee_static_member(&call.callee) {
        return TIMED_FUNCTIONS.contains(&name)
            && matches!(object.get_inner_expression(), Expression::Identifier(id) if id.name == "window");
    }
    false
}

fn is_next_tick_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Some((name, object)) = callee_static_member(&call.callee) else { return false };
    if name == "$nextTick" && is_this_object(object, ctx) {
        return true;
    }
    if name == "nextTick"
        && matches!(object.get_inner_expression(), Expression::Identifier(id) if id.name == "Vue")
    {
        return true;
    }
    false
}

/// Walk down the leftmost edge of a member/call chain and return the root
/// identifier name. Mirrors upstream `getRootObjectName`.
fn get_root_object_name<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    let inner = expr.get_inner_expression();
    if let Expression::ChainExpression(c) = inner {
        return root_from_chain_element(&c.expression);
    }
    root_from_expr(inner)
}

fn root_from_expr<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(m) => get_root_object_name(&m.object),
        Expression::ComputedMemberExpression(m) => get_root_object_name(&m.object),
        Expression::PrivateFieldExpression(m) => get_root_object_name(&m.object),
        Expression::CallExpression(call) => get_root_object_name(&call.callee),
        Expression::TSNonNullExpression(t) => get_root_object_name(&t.expression),
        _ => None,
    }
}

fn root_from_chain_element<'a>(elem: &ChainElement<'a>) -> Option<&'a str> {
    match elem {
        ChainElement::StaticMemberExpression(m) => get_root_object_name(&m.object),
        ChainElement::ComputedMemberExpression(m) => get_root_object_name(&m.object),
        ChainElement::PrivateFieldExpression(m) => get_root_object_name(&m.object),
        ChainElement::CallExpression(call) => get_root_object_name(&call.callee),
        ChainElement::TSNonNullExpression(t) => get_root_object_name(&t.expression),
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return;
                        },
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      ...foo,
                      computed: {
                        ...mapGetters({
                          test: 'getTest'
                        }),
                        foo: function () {
                          var bar = 0
                          try {
                            bar = bar / 0
                          } catch (e) {
                            return e
                          } finally {
                            return bar
                          }
                        },
                        bar: {
                          set () {
                            new Promise((resolve, reject) => {})
                          }
                        },
                        baz: {
                          ...mapGetters({ get: 'getBaz' }),
                          ...mapActions({ set: 'setBaz' })
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    async function resolveComponents(components) {
                      return await Promise.all(components.map(async (component) => {
                          if(typeof component === 'function') {
                                return await component()
                            }
                            return component;
                      }));
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo() {
                          return {
                            async bar() {
                              const data = await baz(this.a)
                              return data
                            }
                          }
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo() {
                          const a = 'test'
                          return [
                            async () => {
                              const baz = await bar(a)
                              return baz
                            },
                            'b',
                            {}
                          ]
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo() {
                          return function () {
                            return async () => await bar()
                          }
                        },
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo() {
                          return new Promise.resolve()
                        },
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo() {
                          return new Bar(async () => await baz())
                        },
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo() {
                          return someFunc.doSomething({
                            async bar() {
                              return await baz()
                            }
                          })
                        },
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                        computed: {
                            foo() {
                                return this.bar
                                  ? {
                                      baz:() => Promise.resolve(1)
                                    }
                                  : {}
                            }
                        }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                        computed: {
                            foo() {
                                return this.bar ? () => Promise.resolve(1) : null
                            }
                        }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                        computed: {
                            foo() {
                                return this.bar ? async () => 1 : null
                            }
                        }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    Vue.component('test',{
                      data1: new Promise(),
                      data2: Promise.resolve(),
                    })",
            None,
            None,
            None,
        ), // languageOptions,
        (
            "
                    import {computed} from 'vue'
                    export default {
                      setup() {
                        const test1 = computed(() => {})
                        const test2 = computed(function () {
                          var bar = 0
                          try {
                            bar = bar / 0
                          } catch (e) {
                            return e
                          } finally {
                            return bar
                          }
                        })
                        const test3 = computed({
                          set() {
                            new Promise((resolve, reject) => {})
                          }
                        })
                        const test4 = computed(() => {
                          return {
                            async bar() {
                              const data = await baz(this.a)
                              return data
                            }
                          }
                        })
                        const test5 = computed(() => {
                          const a = 'test'
                          return [
                            async () => {
                              const baz = await bar(a)
                              return baz
                            },
                            'b',
                            {}
                          ]
                        })
                        const test6 = computed(() => function () {
                          return async () => await bar()
                        })
                        const test7 = computed(() => new Promise.resolve())
                        const test8 = computed(() => {
                          return new Bar(async () => await baz())
                        })
                        const test9 = computed(() => {
                          return someFunc.doSomething({
                            async bar() {
                              return await baz()
                            }
                          })
                        })
                        const test10 = computed(() => {
                          return this.bar
                                  ? {
                                      baz:() => Promise.resolve(1)
                                    }
                                  : {}
                        })
                        const test11 = computed(() => {
                          return this.bar ? () => Promise.resolve(1) : null
                        })
                        const test12 = computed(() => {
                          return this.bar ? async () => 1 : null
                        })
                        const test13 = computed(() => {
                          bar()
                        })
                      }
                    }
                    ",
            None,
            None,
            None,
        ), // languageOptions,
        (
            r#"
                  <template>
                    <div class="f-c" style="height: 100%;">
                    </div>
                  </template>
                  <script setup>
                  import { ref, computed } from 'vue' // each time uncomment error will print. anything from 'vue'
                  import { useStore } from 'vuex' // others like this is ok
                  </script>
                  <script>
                  export default {
                    name: 'App',
                    components: {
                    },
                  }
                  </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, "sourceType": "module", "ecmaVersion": 2020 },
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return z.catch(
                            z.string().check(z.minLength(2)),
                            'default'
                          ).then(val => val).finally(() => {})
                        }
                      }
                    }
                  </script>
",
            Some(serde_json::json!([{ "ignoredObjectNames": ["z"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                  <script setup>
                  import { computed } from 'vue'
            
                  const numberWithCatch = computed(() => z.number().catch(42))
                  </script>",
            Some(serde_json::json!([{ "ignoredObjectNames": ["z"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, "sourceType": "module", "ecmaVersion": 2020 },
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return z.a?.['b']?.[c].d.method().catch(err => err).finally(() => {})
                        }
                      }
                    }
                  </script>
",
            Some(serde_json::json!([{ "ignoredObjectNames": ["z"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, "sourceType": "module", "ecmaVersion": 2020 },
        (
            r#"
                  <script setup lang="ts">
                  import { computed } from 'vue'
                  import { z } from 'zod'
            
                  const foo = computed(() => z.a?.['b'].c!.d.method().catch(err => err).finally(() => {}))
                  </script>"#,
            Some(serde_json::json!([{ "ignoredObjectNames": ["z"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
    ];

    let fail = vec![
        (
            "
<script>
                    export default {
                      computed: {
                        foo: async function () {
                          return await someFunc()
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: async function () {
                          return new Promise((resolve, reject) => {})
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return bar.then(response => {})
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return bar?.then?.(response => {})
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return (bar?.then)?.(response => {})
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return bar.catch(e => {})
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return Promise.all([])
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return bar.finally(res => {})
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: function () {
                        return Promise.race([])
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: function () {
                        return Promise.reject([])
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: function () {
                        return Promise.resolve([])
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo () {
                        return Promise.resolve([])
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: {
                        get () {
                          return Promise.resolve([])
                        }
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  new Vue({
                    computed: {
                      foo: {
                        get () {
                          return Promise.resolve([])
                        }
                      }
                    }
                  })
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 6 },
        (
            "
<script>
                  new Vue({
                    computed: {
                      foo: {
                        get () {
                          return test.blabla.then([])
                        }
                      }
                    }
                  })
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "ecmaVersion": 6 },
        (
            "
<script>
                  new Vue({
                    computed: {
                      foo () {
                        this.$nextTick(() => {})
                        Vue.nextTick(() => {})
                        return 'foo'
                      }
                    }
                  })
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  new Vue({
                    computed: {
                      async foo () {
                        await this.$nextTick()
                        await Vue.nextTick()
                        return 'foo'
                      }
                    }
                  })
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: function () {
                        setTimeout(() => { }, 0)
                        window.setTimeout(() => { }, 0)
                        setInterval(() => { }, 0)
                        window.setInterval(() => { }, 0)
                        setImmediate(() => { })
                        window.setImmediate(() => { })
                        requestAnimationFrame(() => {})
                        window.requestAnimationFrame(() => {})
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: function () {
                        setTimeout?.(() => { }, 0)
                        window?.setTimeout?.(() => { }, 0)
                        setInterval(() => { }, 0)
                        window?.setInterval?.(() => { }, 0)
                        setImmediate?.(() => { })
                        window?.setImmediate?.(() => { })
                        requestAnimationFrame?.(() => {})
                        window?.requestAnimationFrame?.(() => {})
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  export default {
                    computed: {
                      foo: function () {
                        setTimeout?.(() => { }, 0)
                        ;(window?.setTimeout)?.(() => { }, 0)
                        setInterval(() => { }, 0)
                        ;(window?.setInterval)?.(() => { }, 0)
                        setImmediate?.(() => { })
                        ;(window?.setImmediate)?.(() => { })
                        requestAnimationFrame?.(() => {})
                        ;(window?.requestAnimationFrame)?.(() => {})
                      }
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  import {computed} from 'vue'
                  export default {
                    setup() {
                      const test1 = computed(async () => {
                        return await someFunc()
                      })
                      const test2 = computed(async () => await someFunc())
                      const test3 = computed(async function () {
                        return await someFunc()
                      })
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  import {computed} from 'vue'
                  export default {
                    setup() {
                      const test = computed(async () => {
                        return new Promise((resolve, reject) => {})
                      })
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  import {computed} from 'vue'
                  export default {
                    setup() {
                      const test1 = computed(() => {
                        return bar.then(response => {})
                      })
                      const test2 = computed(() => {
                        return Promise.all([])
                      })
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  import {computed} from 'vue'
                  export default {
                    setup() {
                      const test1 = computed({
                        get: () => {
                          return Promise.resolve([])
                        }
                      })
                      const test2 = computed({
                        get() {
                          return Promise.resolve([])
                        }
                      })
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  import {computed} from 'vue'
                  export default {
                    setup() {
                      const test = computed(() => {
                        setTimeout(() => { }, 0)
                        window.setTimeout(() => { }, 0)
                        setInterval(() => { }, 0)
                        window.setInterval(() => { }, 0)
                        setImmediate(() => { })
                        window.setImmediate(() => { })
                        requestAnimationFrame(() => {})
                        window.requestAnimationFrame(() => {})
                      })
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                  import {computed} from 'vue'
                  export default {
                    setup() {
                      const test = computed(async () => {
                        bar()
                      })
                    }
                  }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                  <script setup>
                  import {computed} from 'vue'
                  const test1 = computed(async () => {
                    return await someFunc()
                  })
                  const test2 = computed(async () => await someFunc())
                  const test3 = computed(async function () {
                    return await someFunc()
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, ...languageOptions },
        (
            "
                  <script setup>
                  import {computed} from 'vue'
                  const test = computed(async () => {
                    return new Promise((resolve, reject) => {})
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, ...languageOptions },
        (
            "
                  <script setup>
                  import {computed} from 'vue'
                  const test1 = computed(() => {
                    return bar.then(response => {})
                  })
                  const test2 = computed(() => {
                    return Promise.all([])
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, ...languageOptions },
        (
            "
                  <script setup>
                  import {computed} from 'vue'
                  const test1 = computed({
                    get: () => {
                      return Promise.resolve([])
                    }
                  })
                  const test2 = computed({
                    get() {
                      return Promise.resolve([])
                    }
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, ...languageOptions },
        (
            "
                  <script setup>
                  import {computed} from 'vue'
                  const test = computed(() => {
                    setTimeout(() => { }, 0)
                    window.setTimeout(() => { }, 0)
                    setInterval(() => { }, 0)
                    window.setInterval(() => { }, 0)
                    setImmediate(() => { })
                    window.setImmediate(() => { })
                    requestAnimationFrame(() => {})
                    window.requestAnimationFrame(() => {})
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, ...languageOptions },
        (
            "
                  <script setup>
                  import {computed} from 'vue'
                  const test = computed(async () => {
                    bar()
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, ...languageOptions },
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return myFunc().catch('default')
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return z.number().catch(42)
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return someLib.string().catch(42)
                        }
                      }
                    }
                  </script>
",
            Some(serde_json::json!([{ "ignoredObjectNames": ["z"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
        (
            "
                    <script setup>
                    import {computed} from 'vue'
            
                    const deepCall = computed(() => z.a.b.c.d().e().f().catch())
                    </script>
                  ",
            Some(serde_json::json!([{ "ignoredObjectNames": ["a"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // { parser, "sourceType": "module", "ecmaVersion": 2020 },
    ];

    Tester::new(NoAsyncInComputedProperties::NAME, NoAsyncInComputedProperties::PLUGIN, pass, fail)
        .test_and_snapshot();
}
