use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_this_object, is_vue_next_tick_import},
};

fn use_promise_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Use the Promise returned by `nextTick` instead of passing a callback function.",
    )
    .with_label(span)
}

fn use_callback_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Pass a callback function to `nextTick` instead of using the returned Promise.",
    )
    .with_label(span)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum NextTickOption {
    /// Require using the Promise returned by `nextTick`.
    #[default]
    #[serde(rename = "promise")]
    Promise,
    /// Require passing a callback function to `nextTick`.
    #[serde(rename = "callback")]
    Callback,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NextTickStyle(NextTickOption);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce Promise or callback style in `nextTick`.
    ///
    /// ### Why is this bad?
    ///
    /// In Vue.js, `nextTick` can be used either by passing a callback or
    /// by using the returned Promise. Mixing these styles makes the code
    /// harder to read and inconsistent. Choose one style consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with default `"promise"` option:
    /// ```js
    /// this.$nextTick(() => {})
    /// Vue.nextTick(() => {})
    /// import { nextTick } from 'vue'
    /// nextTick(() => {})
    /// ```
    ///
    /// Examples of **correct** code for this rule with default `"promise"` option:
    /// ```js
    /// this.$nextTick().then(() => {})
    /// await Vue.nextTick()
    /// import { nextTick } from 'vue'
    /// await nextTick()
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `"callback"` option:
    /// ```js
    /// await this.$nextTick()
    /// Vue.nextTick().then(() => {})
    /// ```
    ///
    /// Examples of **correct** code for this rule with `"callback"` option:
    /// ```js
    /// this.$nextTick(() => {})
    /// Vue.nextTick(() => {})
    /// ```
    NextTickStyle,
    vue,
    style,
    fix,
    config = NextTickOption,
    version = "next",
);

impl Rule for NextTickStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let report_span = match node.kind() {
            AstKind::StaticMemberExpression(m) => {
                let valid = match m.property.name.as_str() {
                    "$nextTick" => is_this_object(&m.object, ctx),
                    "nextTick" => m.object.is_specific_id("Vue"),
                    _ => false,
                };
                if !valid {
                    return;
                }
                m.property.span
            }
            AstKind::IdentifierReference(ident) => {
                if !is_vue_next_tick_import(ident, ctx) {
                    return;
                }
                ident.span
            }
            _ => return,
        };

        let parent = ctx.nodes().parent_node(node.id());
        let AstKind::CallExpression(call) = parent.kind() else {
            return;
        };
        if call.callee.span() != node.kind().span() {
            return;
        }

        match self.0 {
            NextTickOption::Callback => {
                if call.arguments.len() != 1 || is_awaited_promise(parent, ctx) {
                    ctx.diagnostic(use_callback_diagnostic(report_span));
                }
            }
            NextTickOption::Promise => {
                if !call.arguments.is_empty() || !is_awaited_promise(parent, ctx) {
                    ctx.diagnostic_with_fix(use_promise_diagnostic(report_span), |fixer| {
                        fixer.insert_text_after_range(
                            Span::new(report_span.end, report_span.end),
                            "().then",
                        )
                    });
                }
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

fn is_awaited_promise(call_node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    match ctx.nodes().parent_kind(call_node.id()) {
        AstKind::AwaitExpression(_) => true,
        AstKind::StaticMemberExpression(m) => m.property.name == "then",
        _ => false,
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;
    // https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/next-tick-style.test.ts

    let pass = vec![
        (
            "r
            <script>
            import { nextTick as nt } from 'vue';
            export default {
                async mounted() {
                    this.$nextTick().then(() => callback());
                    Vue.nextTick().then(() => callback());
                    nt().then(() => callback());
                    await this.$nextTick(); callback();
                    await Vue.nextTick(); callback();
                    await nt(); callback();
                }
            }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "r
            <script>
            import { nextTick as nt } from 'vue';
            export default {
                async mounted() {
                    this.$nextTick().then(() => callback());
                    Vue.nextTick().then(() => callback());
                    nt().then(() => callback());
                    await this.$nextTick(); callback();
                    await Vue.nextTick(); callback();
                    await nt(); callback();
                }
            }
            </script>
            ",
            Some(serde_json::json!(["promise"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    this.$nextTick(() => callback());
                    Vue.nextTick(() => callback());
                    nt(() => callback());
                    this.$nextTick(callback);
                    Vue.nextTick(callback);
                    nt(callback);
                }
            }
            </script>
            ",
            Some(serde_json::json!(["callback"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    foo.then(this.$nextTick);
                    foo.then(Vue.nextTick);
                    foo.then(nt);
                    foo.then(nt, catchHandler);
                    foo.then(Vue.nextTick, catchHandler);
                    foo.then(this.$nextTick, catchHandler);
                }
            }
            </script>
            ",
            Some(serde_json::json!(["promise"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    foo.then(this.$nextTick);
                    foo.then(Vue.nextTick);
                    foo.then(nt);
                    foo.then(nt, catchHandler);
                    foo.then(Vue.nextTick, catchHandler);
                    foo.then(this.$nextTick, catchHandler);
                }
            }
            </script>
            ",
            Some(serde_json::json!(["callback"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    this.$nextTick(() => callback());
                    Vue.nextTick(() => callback());
                    nt(() => callback());
                    this.$nextTick(callback);
                    Vue.nextTick(callback);
                    nt(callback);
                }
            }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    this.$nextTick(() => callback());
                    Vue.nextTick(() => callback());
                    nt(() => callback());
                    this.$nextTick(callback);
                    Vue.nextTick(callback);
                    nt(callback);
                }
            }
            </script>
            ",
            Some(serde_json::json!(["promise"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                async mounted() {
                    this.$nextTick().then(() => callback());
                    Vue.nextTick().then(() => callback());
                    nt().then(() => callback());
                    await this.$nextTick(); callback();
                    await Vue.nextTick(); callback();
                    await nt(); callback();
                }
            }
            </script>
            ",
            Some(serde_json::json!(["callback"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix = vec![(
        "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    this.$nextTick(() => callback());
                    Vue.nextTick(() => callback());
                    nt(() => callback());
                    this.$nextTick(callback);
                    Vue.nextTick(callback);
                    nt(callback);
                }
            }
            </script>
            ",
        "r
            <script>import { nextTick as nt } from 'vue';
            export default {
                mounted() {
                    this.$nextTick().then(() => callback());
                    Vue.nextTick().then(() => callback());
                    nt().then(() => callback());
                    this.$nextTick().then(callback);
                    Vue.nextTick().then(callback);
                    nt().then(callback);
                }
            }
            </script>
            ",
        None,
        Some(PathBuf::from("test.vue")),
    )];

    Tester::new(NextTickStyle::NAME, NextTickStyle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
