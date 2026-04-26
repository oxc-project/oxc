use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode, ast_util::get_declaration_from_reference_id, context::LintContext, rule::Rule,
};

const DEPRECATED_EVENTS_API_METHODS: [&str; 3] = ["$on", "$off", "$once"];

fn no_deprecated_events_api_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The Events api `{}` is deprecated.",
        DEPRECATED_EVENTS_API_METHODS.join("`, `")
    ))
    .with_help("Using external library instead, for example mitt.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedEventsApi;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow using deprecated Events API (`$on`, `$off`, `$once`) in Vue.js 3.0.0+.
    ///
    /// ### Why is this bad?
    ///
    /// In Vue.js 3.0.0+, the internal event APIs `$on`, `$off`, and `$once` have been removed.
    /// These methods were used for event handling between components but are no longer available.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   mounted() {
    ///     this.$on('event', () => {})
    ///     this.$off('event')
    ///     this.$once('event', () => {})
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// import mitt from 'mitt'
    ///
    /// const emitter = mitt()
    ///
    /// export default {
    ///   mounted() {
    ///     emitter.on('event', () => {})
    ///     emitter.off('event')
    ///     emitter.once('event', () => {})
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedEventsApi,
    vue,
    correctness,
    version = "next",
);

impl Rule for NoDeprecatedEventsApi {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            // It is OK because checking whether it is deprecated, e.g. `this.$on?.()`
            return;
        }

        let callee = call_expr.callee.get_inner_expression();

        let member_expr = match callee {
            Expression::StaticMemberExpression(member_expr) => member_expr.as_ref(),
            Expression::ChainExpression(chain_expr) => {
                let Some(MemberExpression::StaticMemberExpression(member_expr)) =
                    chain_expr.expression.as_member_expression()
                else {
                    return;
                };
                member_expr.as_ref()
            }
            _ => return,
        };

        let prop_name = member_expr.property.name.as_str();
        if !DEPRECATED_EVENTS_API_METHODS.contains(&prop_name) {
            return;
        }

        let mut current = member_expr.object.get_inner_expression();

        loop {
            match current {
                Expression::ThisExpression(_) => {
                    return ctx.diagnostic(no_deprecated_events_api_diagnostic(
                        member_expr.property.span,
                    ));
                }
                Expression::Identifier(ident) => {
                    if is_this(&ident, ctx) {
                        ctx.diagnostic(no_deprecated_events_api_diagnostic(
                            member_expr.property.span,
                        ));
                    }
                    return;
                }
                Expression::ParenthesizedExpression(paren) => {
                    current = paren.expression.get_inner_expression();
                }
                _ => return,
            }
        }
    }
}

#[inline]
fn is_this(ident: &oxc_ast::ast::IdentifierReference, ctx: &LintContext<'_>) -> bool {
    get_declaration_from_reference_id(ident.reference_id(), ctx.semantic())
        .and_then(|node| match node.kind() {
            AstKind::VariableDeclarator(var) => var.init.as_ref(),
            _ => None,
        })
        .is_some_and(|init| matches!(init, Expression::ThisExpression(_)))
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;
    // ref: https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/no-deprecated-events-api.test.ts

    let pass = vec![
        (
            r#"
            createApp({
              mounted() {
                this.$emit('start')
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            createApp({
              methods: {
                click() {
                  this.$emit('click')
                }
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            app.component('some-comp', {
              mounted() {
                this.$emit('start')
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            <script>
            export default {
              mounted() {
                this.$emit('start')
              }
            }
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            import mitt from 'mitt'
            const emitter = mitt()

            export default {
              setup() {
                emitter.on('foo', e => console.log('foo', e))
                emitter.emit('foo', { a: 'b' })
                emitter.off('foo', onFoo)
              }
            }
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              mounted() {
                a(this.$on)
              }
            }
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            app.component('some-comp', {
              mounted() {
                this.$on?.('start', foo)
                this.$off?.('start', foo)
                this.$once?.('start', foo)
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    let fail = vec![
        (
            r#"
            app.component('some-comp', {
              mounted() {
                this.$on('start', function(args) {
                  console.log('start', args)
                })
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            app.component('some-comp', {
              mounted() {
                this.$off('start')
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            <script>
            export default {
              mounted() {
                this.$once('start', function() {
                  console.log('start')
                })
              }
            }
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            app.component('some-comp', {
              mounted() {
                const vm = this
                vm.$on('start', function(args) {
                  console.log('start', args)
                })
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            app.component('some-comp', {
              mounted() {
                this?.$on('start')
                this?.$off('start')
                this?.$once('start')
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r#"
            app.component('some-comp', {
              mounted() {
                ;(this?.$on)('start')
                ;(this?.$off)('start')
                ;(this?.$once)('start')
              }
            })
            "#,
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    Tester::new(NoDeprecatedEventsApi::NAME, NoDeprecatedEventsApi::PLUGIN, pass, fail)
        .test_and_snapshot();
}
