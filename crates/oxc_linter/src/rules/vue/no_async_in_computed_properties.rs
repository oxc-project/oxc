use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{CallExpression, ChainElement, Expression, ExpressionKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{ComputedContext, find_computed_context, get_computed_getter_context, is_this_object},
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
                if let ExpressionKind::Identifier(id) =
                    new_expr.callee.get_inner_expression().kind()
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
        // The shared `ComputedContext::CompositionApi` carries a getter span used by
        // `no-side-effects-in-computed-properties` for setup-variable detection; this rule
        // doesn't need it.
        ComputedContext::CompositionApi(_) => {
            ctx.diagnostic(unexpected_in_function(span, kind));
        }
    }
}

const PROMISE_FUNCTIONS: &[&str] = &["then", "catch", "finally"];
const PROMISE_METHODS: &[&str] =
    &["all", "allSettled", "any", "race", "reject", "resolve", "try", "withResolvers"];
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
    let member = match inner.kind() {
        ExpressionKind::StaticMemberExpression(m) => m,
        ExpressionKind::ChainExpression(c) => match &c.expression {
            ChainElement::MemberExpression(m) => match m.as_static_member_expression() {
                Some(m) => m,
                None => return None,
            },
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
        && matches!(object.get_inner_expression().kind(), ExpressionKind::Identifier(id) if id.name == "Promise");
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
    if let ExpressionKind::Identifier(id) = inner.kind() {
        return TIMED_FUNCTIONS.contains(&id.name.as_str());
    }
    // `window.setTimeout(...)` / `window?.setTimeout?.(...)` / `(window?.setTimeout)?.(...)`.
    if let Some((name, object)) = callee_static_member(&call.callee) {
        return TIMED_FUNCTIONS.contains(&name)
            && matches!(object.get_inner_expression().kind(), ExpressionKind::Identifier(id) if id.name == "window");
    }
    false
}

fn is_next_tick_call(call: &CallExpression<'_>, ctx: &LintContext<'_>) -> bool {
    let Some((name, object)) = callee_static_member(&call.callee) else { return false };
    if name == "$nextTick" && is_this_object(object, ctx) {
        return true;
    }
    if name == "nextTick"
        && matches!(object.get_inner_expression().kind(), ExpressionKind::Identifier(id) if id.name == "Vue")
    {
        return true;
    }
    false
}

/// Walk down the leftmost edge of a member/call chain and return the root
/// identifier name. Mirrors upstream `getRootObjectName`.
fn get_root_object_name<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    let inner = expr.get_inner_expression();
    if let ExpressionKind::ChainExpression(c) = inner.kind() {
        return root_from_chain_element(&c.expression);
    }
    root_from_expr(inner)
}

fn root_from_expr<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    match expr.kind() {
        ExpressionKind::Identifier(id) => Some(id.name.as_str()),
        ExpressionKind::StaticMemberExpression(m) => get_root_object_name(&m.object),
        ExpressionKind::ComputedMemberExpression(m) => get_root_object_name(&m.object),
        ExpressionKind::PrivateFieldExpression(m) => get_root_object_name(&m.object),
        ExpressionKind::CallExpression(call) => get_root_object_name(&call.callee),
        ExpressionKind::TSNonNullExpression(t) => get_root_object_name(&t.expression),
        _ => None,
    }
}

fn root_from_chain_element<'a>(elem: &ChainElement<'a>) -> Option<&'a str> {
    match elem {
        ChainElement::MemberExpression(m) => get_root_object_name(m.object()),
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
        (
            "
<script>
                    export default {
                      computed: {
                        foo: function () {
                          return Promise.allSettled([])
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
                          return Promise.any([])
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
                          return Promise.try([])
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
                          return Promise.withResolvers([])
                        }
                      }
                    }
                  </script>
",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // languageOptions,
    ];

    Tester::new(NoAsyncInComputedProperties::NAME, NoAsyncInComputedProperties::PLUGIN, pass, fail)
        .test_and_snapshot();
}
