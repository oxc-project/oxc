use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, IdentifierReference, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

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
    version = "1.62.0",
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

        let member_expr = match call_expr.callee.get_inner_expression() {
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

        match member_expr.object.get_inner_expression() {
            Expression::ThisExpression(_) if is_in_vue_component_instance_method(node, ctx) => {
                ctx.diagnostic(no_deprecated_events_api_diagnostic(member_expr.property.span));
            }
            Expression::Identifier(ident)
                if is_this(ident, ctx) && is_in_vue_component_instance_method(node, ctx) =>
            {
                ctx.diagnostic(no_deprecated_events_api_diagnostic(member_expr.property.span));
            }
            _ => {}
        }
    }
}

#[inline]
fn is_this(ident: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    get_declaration_from_reference_id(ident.reference_id(), ctx.semantic())
        .and_then(|node| match node.kind() {
            AstKind::VariableDeclarator(var) => var.init.as_ref(),
            _ => None,
        })
        .is_some_and(|init| matches!(init, Expression::ThisExpression(_)))
}

fn is_in_vue_component_instance_method(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let Some(function_node) = ctx
        .nodes()
        .ancestors(node.id())
        .find(|ancestor| matches!(ancestor.kind(), AstKind::Function(_)))
    else {
        return false;
    };

    let property_node = ctx.nodes().parent_node(function_node.id());
    let AstKind::ObjectProperty(_) = property_node.kind() else {
        return false;
    };

    let object_node = ctx.nodes().parent_node(property_node.id());
    if is_vue_component_options_object(object_node, ctx) {
        return true;
    }

    let container_property_node = ctx.nodes().parent_node(object_node.id());
    if !matches!(container_property_node.kind(), AstKind::ObjectProperty(_)) {
        return false;
    }

    let Some(container_name) = container_property_node
        .kind()
        .as_object_property()
        .and_then(|prop| if prop.computed { None } else { prop.key.static_name() })
    else {
        return false;
    };

    matches!(container_name.as_ref(), "computed" | "methods" | "watch")
        && is_vue_component_options_object(
            ctx.nodes().parent_node(container_property_node.id()),
            ctx,
        )
}

fn is_vue_component_options_object(object_node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let AstKind::ObjectExpression(object_expr) = object_node.kind() else {
        return false;
    };

    ctx.nodes().ancestors(object_node.id()).any(|ancestor| match ancestor.kind() {
        AstKind::ExportDefaultDeclaration(export_default_decl) => {
            export_default_decl.declaration.span() == object_expr.span
        }
        AstKind::CallExpression(call_expr) => {
            call_expr
                .arguments
                .iter()
                .any(|arg| arg.as_expression().is_some_and(|expr| expr.span() == object_expr.span))
                && is_vue_component_options_call(call_expr)
        }
        _ => false,
    })
}

fn is_vue_component_options_call(call_expr: &CallExpression<'_>) -> bool {
    if call_expr
        .callee
        .get_identifier_reference()
        .is_some_and(|ident| matches!(ident.name.as_str(), "createApp" | "defineComponent"))
    {
        return true;
    }

    call_expr.callee.get_member_expr().is_some_and(|member_expr| {
        member_expr.static_property_name().is_some_and(|name| name == "component")
    })
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;
    // ref: https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/no-deprecated-events-api.test.ts

    let pass = vec![
        (
            r"
            createApp({
              mounted() {
                this.$emit('start')
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            createApp({
              methods: {
                click() {
                  this.$emit('click')
                }
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            const another = function () {
          this.$on('start', args => {
            console.log('start')
          })
        }

        createApp({
          mounted () {
            this.$emit('start')
          }
        })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            app.component('some-comp', {
              mounted() {
                this.$emit('start')
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            <script>
            export default {
              mounted() {
                this.$emit('start')
              }
            }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
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
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
            <script>
            export default {
              mounted() {
                a(this.$on)
              }
            }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
            app.component('some-comp', {
              mounted() {
                this.$on?.('start', foo)
                this.$off?.('start', foo)
                this.$once?.('start', foo)
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    let fail = vec![
        (
            r"
            app.component('some-comp', {
              mounted() {
                this.$on('start', function(args) {
                  console.log('start', args)
                })
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            app.component('some-comp', {
              mounted() {
                this.$off('start')
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            <script>
            export default {
              mounted() {
                this.$once('start', function() {
                  console.log('start')
                })
              }
            }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
            app.component('some-comp', {
              mounted() {
                const vm = this
                vm.$on('start', function(args) {
                  console.log('start', args)
                })
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            app.component('some-comp', {
              mounted() {
                this?.$on('start')
                this?.$off('start')
                this?.$once('start')
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            r"
            app.component('some-comp', {
              mounted() {
                ;(this?.$on)('start')
                ;(this?.$off)('start')
                ;(this?.$once)('start')
              }
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    Tester::new(NoDeprecatedEventsApi::NAME, NoDeprecatedEventsApi::PLUGIN, pass, fail)
        .test_and_snapshot();
}
