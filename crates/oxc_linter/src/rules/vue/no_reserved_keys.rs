use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectPropertyKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

/// Reserved instance properties of the Vue instance (Vue 2/3 common).
/// Source: <https://github.com/vuejs/eslint-plugin-vue/blob/master/lib/utils/vue-reserved.json>
pub(super) const RESERVED_KEYS: &[&str] = &[
    "$data",
    "$props",
    "$el",
    "$options",
    "$parent",
    "$root",
    "$children",
    "$slots",
    "$scopedSlots",
    "$refs",
    "$isServer",
    "$attrs",
    "$listeners",
    "$watch",
    "$set",
    "$delete",
    "$on",
    "$once",
    "$off",
    "$emit",
    "$mount",
    "$forceUpdate",
    "$nextTick",
    "$destroy",
];

fn reserved_key_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Key `{name}` is reserved.")).with_label(span)
}

fn starts_with_underscore_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Keys starting with `_` are reserved in `{name}` group."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoReservedKeysConfig {
    /// Extra reserved key names to disallow, on top of the built-in list.
    reserved: Vec<CompactStr>,
    /// Extra component option groups to inspect, on top of the built-in
    /// `props` / `computed` / `data` / `asyncData` / `methods` / `setup`.
    groups: Vec<CompactStr>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NoReservedKeys(Box<NoReservedKeysConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow overwriting reserved Vue instance keys (e.g. `$data`, `$emit`)
    /// or using `_`-prefixed keys inside `data` / `asyncData`.
    ///
    /// ### Why is this bad?
    ///
    /// Vue exposes a number of instance properties (`$emit`, `$data`, `$props`,
    /// etc.). Defining a prop, computed, data, method or setup return key with
    /// the same name overwrites the underlying Vue API and silently breaks the
    /// component (e.g. `methods: { $emit() {} }` clobbers event emission).
    /// Vue also reserves `_`-prefixed names inside its reactivity system, so
    /// `data() { return { _foo: 1 } }` may collide with internal state.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: ['$data'],
    ///   methods: {
    ///     $emit() {}
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: ['fooData'],
    ///   methods: {
    ///     send() {}
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// ### Options
    ///
    /// This rule has an options object with the following defaults:
    ///
    /// ```json
    /// {
    ///   "reserved": [],
    ///   "groups": []
    /// }
    /// ```
    ///
    /// #### `reserved`
    ///
    /// An array of extra key names to treat as reserved, in addition to the
    /// built-in Vue instance properties.
    ///
    /// #### `groups`
    ///
    /// An array of extra component option groups to inspect, in addition to the
    /// built-in `props` / `computed` / `data` / `asyncData` / `methods` / `setup`.
    NoReservedKeys,
    vue,
    correctness,
    config = NoReservedKeys,
    version = "next",
);

impl Rule for NoReservedKeys {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectProperty(prop) = node.kind() else { return };
        let Some(group_name) = prop.key.static_name() else { return };
        let group = group_name.as_ref();
        if !self.is_target_group(group) {
            return;
        }

        let mut ancestors = ctx.nodes().ancestors(node.id());
        let Some(parent) = ancestors.next() else { return };
        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }
        let Some(grand) = ancestors.next() else { return };
        let in_vue = matches!(grand.kind(), AstKind::ExportDefaultDeclaration(_))
            || matches!(grand.kind(), AstKind::CallExpression(c)
                if c.callee.get_identifier_reference().is_some_and(|i| i.name == "defineComponent"))
            || matches!(grand.kind(), AstKind::NewExpression(n)
                if n.callee.get_identifier_reference().is_some_and(|i| i.name == "Vue"));
        if !in_vue {
            return;
        }

        match prop.value.get_inner_expression() {
            Expression::ArrayExpression(arr) => {
                for elem in &arr.elements {
                    let Some(Expression::StringLiteral(lit)) = elem.as_expression() else {
                        continue;
                    };
                    if self.is_reserved(lit.value.as_str()) {
                        ctx.diagnostic(reserved_key_diagnostic(lit.value.as_str(), lit.span));
                    }
                }
            }
            Expression::ObjectExpression(obj) => {
                self.check_keys(group, obj, ctx);
            }
            Expression::FunctionExpression(func) => {
                let Some(body) = &func.body else { return };
                for stmt in &body.statements {
                    if let Statement::ReturnStatement(ret) = stmt
                        && let Some(arg) = &ret.argument
                        && let Expression::ObjectExpression(obj) = arg.get_inner_expression()
                    {
                        self.check_keys(group, obj, ctx);
                    }
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                if arrow.expression {
                    // `() => ({foo})` expression body
                    if let Some(Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                        && let Expression::ObjectExpression(obj) =
                            es.expression.get_inner_expression()
                    {
                        self.check_keys(group, obj, ctx);
                    }
                } else {
                    // `() => { return {foo} }` block body
                    for stmt in &arrow.body.statements {
                        if let Statement::ReturnStatement(ret) = stmt
                            && let Some(arg) = &ret.argument
                            && let Expression::ObjectExpression(obj) = arg.get_inner_expression()
                        {
                            self.check_keys(group, obj, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }
}

impl NoReservedKeys {
    /// Built-in groups plus any added via the `groups` option.
    fn is_target_group(&self, group: &str) -> bool {
        matches!(group, "props" | "data" | "asyncData" | "computed" | "methods" | "setup")
            || self.0.groups.iter().any(|g| g.as_str() == group)
    }

    /// Built-in reserved keys plus any added via the `reserved` option.
    fn is_reserved(&self, name: &str) -> bool {
        RESERVED_KEYS.contains(&name) || self.0.reserved.iter().any(|r| r.as_str() == name)
    }

    fn check_keys<'a>(&self, group: &str, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        for prop_kind in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop_kind else { continue };
            let Some(name) = p.key.static_name() else { continue };
            let span = p.key.span();
            let n = name.as_ref();
            if self.is_reserved(n) {
                ctx.diagnostic(reserved_key_diagnostic(n, span));
            } else if matches!(group, "data" | "asyncData") && n.starts_with('_') {
                ctx.diagnostic(starts_with_underscore_diagnostic(group, span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                <script>
                export default {
                  props: ['foo'],
                  computed: { bar() {} },
                  data() { return { dat: null } },
                  methods: { test() {} }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  methods: {
                    _foo() {},
                    test() {}
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  props: { foo: { type: String } }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  data: () => ({ dat: 1 })
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                defineComponent({
                  props: ['foo'],
                  methods: { test() {} }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `groups` option: a custom group with no reserved key is fine
        (
            "
                <script>
                new Vue({
                  foo: { baz: String }
                })
                </script>
            ",
            Some(serde_json::json!([{ "groups": ["foo"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
                <script>
                export default {
                  props: ['$data']
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  props: { $data: { type: String } }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  methods: { $emit() {} }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  computed: { $forceUpdate() {} }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  data() { return { $el: 1 } }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  data() { return { _foo: 1 } }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                defineComponent({
                  methods: { $emit() {} }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `new Vue({...})` constructor (Vue 2)
        (
            "
                <script>
                new Vue({
                  props: { $el: String }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  setup () { return { $el: '' } }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  asyncData () { return { $el: '' } }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  data: { _foo: String }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  data: () => { return { _foo: String } }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  data: () => ({ _foo: String })
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                new Vue({
                  asyncData: () => ({ _foo: String })
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `reserved` + `groups` options (eslint-plugin-vue parity)
        (
            "
                <script>
                new Vue({
                  foo: { bar: String }
                })
                </script>
            ",
            Some(serde_json::json!([{ "reserved": ["bar"], "groups": ["foo"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `reserved` option: an extra key flagged inside a built-in group
        (
            "
                <script>
                export default {
                  methods: { myKey() {} }
                }
                </script>
            ",
            Some(serde_json::json!([{ "reserved": ["myKey"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // `groups` option: an extra group inspected for built-in reserved keys
        (
            "
                <script>
                export default {
                  extraGroup: { $emit() {} }
                }
                </script>
            ",
            Some(serde_json::json!([{ "groups": ["extraGroup"] }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoReservedKeys::NAME, NoReservedKeys::PLUGIN, pass, fail).test_and_snapshot();
}
