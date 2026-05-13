use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpression, CallExpression, ExportDefaultDeclarationKind, Expression, NewExpression,
        ObjectExpression, ObjectPropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn require_type_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prop \"{name}\" should define at least its type."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequirePropTypes;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces that a props statement contains type definition.
    ///
    /// ### Why is this bad?
    ///
    /// In committed code, prop definitions should always be as detailed as
    /// possible, specifying at least type(s).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup>
    /// const props = defineProps({
    ///   name: String
    /// })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup>
    /// const props = defineProps({
    ///   name: { type: String }
    /// })
    /// </script>
    ///
    /// // Or with validator
    /// <script setup>
    /// const props = defineProps({
    ///   name: {
    ///     validator: (value) => value.length > 0
    ///   }
    /// })
    /// </script>
    /// ```
    RequirePropTypes,
    vue,
    style,
    version = "next",
);

impl Rule for RequirePropTypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if ctx.frameworks_options() == FrameworkOptions::VueSetup {
            Self::run_on_setup(node, ctx);
        } else {
            Self::run_on_options(node, ctx);
        }
    }
}

impl RequirePropTypes {
    fn run_on_setup<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Some(ident) = call_expr.callee.get_identifier_reference() else { return };

        if call_expr.type_arguments.is_some() {
            return;
        }

        match ident.name.as_str() {
            "defineProps" => {
                let Some(expr) = call_expr.arguments.first().and_then(|arg| arg.as_expression())
                else {
                    return;
                };
                match expr {
                    // foo: {
                    Expression::ObjectExpression(obj) => Self::check_props(obj, ctx),
                    // foo: [
                    Expression::ArrayExpression(arr) => Self::check_array_props(arr, ctx),
                    _ => {}
                }
            }
            "defineModel" => Self::check_define_model(call_expr, ctx),
            _ => {}
        }
    }

    fn check_define_model<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
        let Some(first_arg) = call_expr.arguments.first() else {
            ctx.diagnostic(require_type_diagnostic(call_expr.span, "modelValue"));
            return;
        };

        let Some(expr) = first_arg.as_expression() else { return };

        if matches!(expr, Expression::Identifier(_)) {
            return;
        }

        if let Expression::StringLiteral(lit) = expr {
            if let Some(second_arg) = call_expr.arguments.get(1)
                && let Some(second_expr) = second_arg.as_expression()
            {
                if matches!(second_expr, Expression::Identifier(_)) {
                    return;
                }
                if let Expression::ObjectExpression(obj) = second_expr
                    && prop_value_has_type(obj)
                {
                    return;
                }
            }
            ctx.diagnostic(require_type_diagnostic(call_expr.span, lit.value.as_str()));
            return;
        }

        let Expression::ObjectExpression(obj) = expr else { return };

        if !prop_value_has_type(obj) {
            ctx.diagnostic(require_type_diagnostic(call_expr.span, "modelValue"));
        }
    }

    fn run_on_options<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let obj = match node.kind() {
            AstKind::ExportDefaultDeclaration(export_decl) => match &export_decl.declaration {
                ExportDefaultDeclarationKind::ObjectExpression(obj) => Some(obj.as_ref()),
                ExportDefaultDeclarationKind::CallExpression(call_expr) => {
                    Self::find_props_in_call(call_expr)
                }
                _ => None,
            },
            AstKind::NewExpression(new_expr) => Self::find_props_in_new_expr(new_expr),
            _ => None,
        };

        let Some(obj) = obj else { return };

        let Some(props_prop) = obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(p) = prop else { return None };
            p.key.static_name().is_some_and(|k| k == "props").then_some(p)
        }) else {
            return;
        };

        match props_prop.value.get_inner_expression() {
            // foo: {
            Expression::ObjectExpression(o) => Self::check_props(o, ctx),
            // foo: [
            Expression::ArrayExpression(a) => Self::check_array_props(a, ctx),
            _ => {}
        }
    }

    fn find_props_in_call<'a>(
        call_expr: &'a CallExpression<'a>,
    ) -> Option<&'a ObjectExpression<'a>> {
        let member_expr = call_expr.callee.get_member_expr()?;
        if member_expr.static_property_name() != Some("extend") {
            return None;
        }

        let arg = call_expr.arguments.first()?.as_expression()?;
        match arg.get_inner_expression() {
            Expression::ObjectExpression(obj) => Some(obj),
            _ => None,
        }
    }

    fn find_props_in_new_expr<'a>(
        new_expr: &'a NewExpression<'a>,
    ) -> Option<&'a ObjectExpression<'a>> {
        let Expression::Identifier(ident) = new_expr.callee.get_inner_expression() else {
            return None;
        };
        if ident.name != "Vue" {
            return None;
        }

        let arg = new_expr.arguments.first()?.as_expression()?;
        match arg.get_inner_expression() {
            Expression::ObjectExpression(obj) => Some(obj),
            _ => None,
        }
    }

    fn check_props(obj: &ObjectExpression, ctx: &LintContext) {
        for prop in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop else { continue };
            let Some(key) = p.key.static_name() else {
                ctx.diagnostic(require_type_diagnostic(p.key.span(), "Unknown prop"));
                continue;
            };

            let is_invalid = match p.value.get_inner_expression() {
                Expression::ObjectExpression(o) => !prop_value_has_type(o),
                Expression::ArrayExpression(a) => a.elements.is_empty(),
                Expression::FunctionExpression(_) => true,
                _ => false,
            };

            if is_invalid {
                ctx.diagnostic(require_type_diagnostic(p.span(), key.as_ref()));
            }
        }
    }

    fn check_array_props(arr: &ArrayExpression, ctx: &LintContext) {
        for elem in &arr.elements {
            let Some(expr) = elem.as_expression() else { continue };
            let name = match expr {
                Expression::StringLiteral(lit) => Some(lit.value.as_str()),
                Expression::Identifier(id) => Some(id.name.as_str()),
                Expression::TemplateLiteral(lit) if lit.expressions.is_empty() => {
                    lit.quasis.first().and_then(|q| q.value.cooked.as_deref())
                }
                _ => None,
            }
            .unwrap_or("Unknown prop");

            ctx.diagnostic(require_type_diagnostic(expr.span(), name));
        }
    }
}

fn prop_value_has_type(obj: &ObjectExpression) -> bool {
    obj.properties.iter().any(|prop| {
        let ObjectPropertyKind::ObjectProperty(p) = prop else { return false };
        let Some(key) = p.key.static_name() else { return false };
        if key == "type" {
            if let Expression::ArrayExpression(a) = p.value.get_inner_expression() {
                return !a.elements.is_empty();
            }
            return true;
        }
        key == "validator"
    })
}

#[test]
#[expect(clippy::literal_string_with_formatting_args)]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;
    // ref: https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/require-prop-types.test.ts

    let pass = vec![
        (
            "
            <script>
            export default {
                ...foo,
                props: {
                    ...test(),
                    foo: String
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
                props: {
                    foo: [String, Number]
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
                props: {
                    foo: {
                        type: String
                    }
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
                props: {
                    foo: {
                        ['type']: String
                    }
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
                props: {
                    foo: {
                        validator: v => v
                    }
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
                props: {
                    foo: {
                        ['validator']: v => v
                    }
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
            export default { props }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default { props: externalProps }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default { props: [] }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default { props: {} }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default Vue.extend({
                props: {
                    foo: {
                        type: String
                    }
                }
            });
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script lang="ts">
            export default (Vue as VueConstructor<Vue>).extend({
                props: {
                    foo: {
                        type: String
                    }
                }
            });
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script lang="ts">
            export default Vue.extend({
                props: {
                    foo: {
                        type: String
                    } as PropOptions<string>
                }
            });
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script lang="ts">
            import {Props1 as Props} from './test01'
            defineProps<Props>()
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            defineProps({
                foo: String
            })
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script setup lang="ts">
            defineProps<{foo:string}>()
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            const m = defineModel({type:String})
            const foo = defineModel('foo', {type:String})
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            const m = defineModel(String)
            const foo = defineModel('foo', String)
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script setup lang="ts">
            const m = defineModel<string>()
            const foo = defineModel<string>('foo')
            </script>
            "#,
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
                props: ['foo', bar, `baz`, foo()]
            }
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            new Vue({
                props: ['foo', bar, `baz`, foo()]
            })
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
        (
            "
            <script>
            export default {
                props: {
                    foo: {
                    }
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
                props: {
                    foo: {
                        type: []
                    }
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
                props: {
                    foo() {}
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
            export default Vue.extend({
                props: {
                    foo: {}
                }
            });
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script lang="ts">
            export default Vue.extend({
                props: {
                    foo: {} as PropOptions<string>
                }
            });
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script lang="ts">
            export default (Vue as VueConstructor<Vue>).extend({
                props: {
                    foo: {} as PropOptions<string>
                }
            });
            </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            defineProps({
                foo: {}
            })
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            defineProps(['foo'])
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            const m = defineModel()
            const foo = defineModel('foo')
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup>
            const m = defineModel({})
            const foo = defineModel('foo',{})
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(RequirePropTypes::NAME, RequirePropTypes::PLUGIN, pass, fail).test_and_snapshot();
}
