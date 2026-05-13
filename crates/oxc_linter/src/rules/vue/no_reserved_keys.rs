use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectPropertyKind, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

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

#[derive(Debug, Default, Clone)]
pub struct NoReservedKeys;

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
    NoReservedKeys,
    vue,
    correctness,
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
        if !matches!(group, "props" | "data" | "asyncData" | "computed" | "methods" | "setup") {
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
                    if RESERVED_KEYS.contains(&lit.value.as_str()) {
                        ctx.diagnostic(reserved_key_diagnostic(lit.value.as_str(), lit.span));
                    }
                }
            }
            Expression::ObjectExpression(obj) => {
                check_keys(group, obj, ctx);
            }
            Expression::FunctionExpression(func) => {
                let Some(body) = &func.body else { return };
                for stmt in &body.statements {
                    if let Statement::ReturnStatement(ret) = stmt
                        && let Some(arg) = &ret.argument
                        && let Expression::ObjectExpression(obj) = arg.get_inner_expression()
                    {
                        check_keys(group, obj, ctx);
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
                        check_keys(group, obj, ctx);
                    }
                } else {
                    // `() => { return {foo} }` block body
                    for stmt in &arrow.body.statements {
                        if let Statement::ReturnStatement(ret) = stmt
                            && let Some(arg) = &ret.argument
                            && let Expression::ObjectExpression(obj) = arg.get_inner_expression()
                        {
                            check_keys(group, obj, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_keys<'a>(group: &str, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
    for prop_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = prop_kind else { continue };
        let Some(name) = p.key.static_name() else { continue };
        let span = p.key.span();
        let n = name.as_ref();
        if RESERVED_KEYS.contains(&n) {
            ctx.diagnostic(reserved_key_diagnostic(n, span));
        } else if matches!(group, "data" | "asyncData") && n.starts_with('_') {
            ctx.diagnostic(starts_with_underscore_diagnostic(group, span));
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
    ];

    Tester::new(NoReservedKeys::NAME, NoReservedKeys::PLUGIN, pass, fail).test_and_snapshot();
}
