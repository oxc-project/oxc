use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpression, ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind,
        TemplateLiteral,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::{find_property, is_vue_component_options_object_excluding_instance},
};

fn require_prop_type_constructor_diagnostic(prop_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("The \"{prop_name}\" property should be a constructor."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequirePropTypeConstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require `props` type values to be a constructor function (e.g. `String`,
    /// `Number`, `Boolean`) rather than a string, number, or other literal.
    ///
    /// ### Why is this bad?
    ///
    /// Vue uses the prop type for runtime validation and dev-time warnings. A
    /// string like `'String'` looks like the constructor but is never matched
    /// against an actual value, silently disabling the check.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     foo: 'String',
    ///     bar: { type: 'Number' },
    ///   },
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     foo: String,
    ///     bar: { type: Number },
    ///   },
    /// }
    /// </script>
    /// ```
    RequirePropTypeConstructor,
    vue,
    correctness,
    fix,
    version = "next",
);

impl Rule for RequirePropTypeConstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj) => {
                if !is_vue_component_options_object_excluding_instance(node, ctx) {
                    return;
                }
                let Some(props_prop) = find_property(obj, "props") else { return };
                let Expression::ObjectExpression(props_obj) =
                    props_prop.value.get_inner_expression()
                else {
                    return;
                };
                verify_props(props_obj, ctx);
            }
            AstKind::CallExpression(call) => {
                let Some(ident) = call.callee.get_identifier_reference() else { return };
                if ident.name != "defineProps" {
                    return;
                }
                let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) else {
                    return;
                };
                let Expression::ObjectExpression(props_obj) = arg.get_inner_expression() else {
                    return;
                };
                verify_props(props_obj, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

fn verify_props<'a>(props_obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
    for entry in &props_obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = entry else { continue };
        let Some(prop_name) = prop.key.static_name() else { continue };
        let value = prop.value.get_inner_expression();

        match value {
            Expression::ArrayExpression(arr) => check_array_elements(arr, prop_name.as_ref(), ctx),
            Expression::ObjectExpression(obj) => {
                let Some(type_prop) = find_property(obj, "type") else { continue };
                let type_value = type_prop.value.get_inner_expression();
                if let Expression::ArrayExpression(arr) = type_value {
                    check_array_elements(arr, prop_name.as_ref(), ctx);
                } else {
                    check_and_report(type_value, prop_name.as_ref(), ctx);
                }
            }
            _ => check_and_report(value, prop_name.as_ref(), ctx),
        }
    }
}

fn check_array_elements<'a>(arr: &ArrayExpression<'a>, prop_name: &str, ctx: &LintContext<'a>) {
    for elem in &arr.elements {
        if let ArrayExpressionElement::SpreadElement(_) = elem {
            continue;
        }
        if let Some(expr) = elem.as_expression() {
            check_and_report(expr, prop_name, ctx);
        }
    }
}

fn check_and_report<'a>(expr: &Expression<'a>, prop_name: &str, ctx: &LintContext<'a>) {
    let expr = expr.get_inner_expression();
    if !is_forbidden_type(expr) {
        return;
    }
    let span = expr.span();
    let fix_replacement = literal_identifier_replacement(expr);

    ctx.diagnostic_with_fix(
        require_prop_type_constructor_diagnostic(prop_name, span),
        |fixer: RuleFixer<'_, 'a>| -> RuleFix {
            if let Some(name) = fix_replacement { fixer.replace(span, name) } else { fixer.noop() }
        },
    );
}

fn is_forbidden_type(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::BinaryExpression(_)
            | Expression::UpdateExpression(_)
    )
}

fn literal_identifier_replacement(expr: &Expression) -> Option<String> {
    match expr {
        Expression::StringLiteral(lit) => {
            let value = lit.value.as_str();
            is_identifier_name(value).then(|| value.to_string())
        }
        Expression::TemplateLiteral(tpl) => single_quasi_identifier(tpl),
        _ => None,
    }
}

fn single_quasi_identifier(tpl: &TemplateLiteral) -> Option<String> {
    if !tpl.expressions.is_empty() || tpl.quasis.len() != 1 {
        return None;
    }
    let cooked = tpl.quasis[0].value.cooked.as_ref()?;
    is_identifier_name(cooked).then(|| cooked.to_string())
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "<script>
            export default {
              props: {
                ...props,
                myProp: Number,
                anotherType: [Number, String],
                extraProp: {
                  type: Number,
                  default: 10
                },
                lastProp: {
                  type: [Number, Boolean]
                },
                nullProp: null,
                nullTypeProp: { type: null }
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("SomeComponent.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                name: [String,,]
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("ExtraCommas.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                name: {
                  type: [String,,]
                }
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("ExtraCommas.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                name: [String,,Number]
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("ExtraCommas.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                name: [,,Number]
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("ExtraCommas.vue")),
        ),
        (
            "<script>
            export default {
              props: ['name',,,]
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("ExtraCommas.vue")),
        ),
        (
            r#"<script setup lang="ts">
            import {Props1 as Props} from './test01'
            defineProps<Props>()
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "<script>
            export default {
              props: {
                myProp: 'Number',
                anotherType: ['Number', 'String'],
                extraProp: {
                  type: 'Number',
                  default: 10
                },
                lastProp: {
                  type: ['Boolean']
                },
                nullProp: 'null'
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("SomeComponent.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                a: `String`,
                b: Foo + '',
                c: 1,
                d: true,
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("SomeComponent.vue")),
        ),
        (
            r#"<script lang="ts">
            export default {
              props: {
                a: {
                  type: 'String',
                  default: 10
                } as PropOptions<string>,
              }
            }
            </script>"#,
            None,
            None,
            Some(PathBuf::from("SomeComponent.vue")),
        ),
        (
            r#"<script lang="ts">
            export default {
              props: {
                name: ['String',,]
              }
            }
            </script>"#,
            None,
            None,
            Some(PathBuf::from("ExtraCommas.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                str: 'String',
                str2: 'a',
                emptyStr: '',
                number: 1000,
                binumber: 0b10000000000000000000000000000000,
                hexnumber: 0x123456789ABCDEF,
                exp1: 1E3,
                exp2: 2e6,
                exp3: 0.1e2,
                bigInt: 9007199254740991n,
                boolean: true,
                'null': null,
                regex: /a/,
                template: `String`,
                emptyTemplate: ``,
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("LiteralsComponent.vue")),
        ),
        (
            "<script setup>
            defineProps({
              a: {
                type: 'String',
                default: 'abc'
              },
            })
            </script>",
            None,
            None,
            Some(PathBuf::from("SomeComponent.vue")),
        ),
    ];

    let fix = vec![
        (
            "<script>
            export default {
              props: {
                myProp: 'Number',
                anotherType: ['Number', 'String'],
                extraProp: {
                  type: 'Number',
                  default: 10
                },
                lastProp: {
                  type: ['Boolean']
                },
                nullProp: 'null'
              }
            }
            </script>",
            "<script>
            export default {
              props: {
                myProp: Number,
                anotherType: [Number, String],
                extraProp: {
                  type: Number,
                  default: 10
                },
                lastProp: {
                  type: [Boolean]
                },
                nullProp: null
              }
            }
            </script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                a: `String`,
                b: Foo + '',
                c: 1,
                d: true,
              }
            }
            </script>",
            "<script>
            export default {
              props: {
                a: String,
                b: Foo + '',
                c: 1,
                d: true,
              }
            }
            </script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"<script lang="ts">
            export default {
              props: {
                a: {
                  type: 'String',
                  default: 10
                } as PropOptions<string>,
              }
            }
            </script>"#,
            r#"<script lang="ts">
            export default {
              props: {
                a: {
                  type: String,
                  default: 10
                } as PropOptions<string>,
              }
            }
            </script>"#,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"<script lang="ts">
            export default {
              props: {
                name: ['String',,]
              }
            }
            </script>"#,
            r#"<script lang="ts">
            export default {
              props: {
                name: [String,,]
              }
            }
            </script>"#,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
            export default {
              props: {
                str: 'String',
                str2: 'a',
                emptyStr: '',
                number: 1000,
                binumber: 0b10000000000000000000000000000000,
                hexnumber: 0x123456789ABCDEF,
                exp1: 1E3,
                exp2: 2e6,
                exp3: 0.1e2,
                bigInt: 9007199254740991n,
                boolean: true,
                'null': null,
                regex: /a/,
                template: `String`,
                emptyTemplate: ``,
              }
            }
            </script>",
            "<script>
            export default {
              props: {
                str: String,
                str2: a,
                emptyStr: '',
                number: 1000,
                binumber: 0b10000000000000000000000000000000,
                hexnumber: 0x123456789ABCDEF,
                exp1: 1E3,
                exp2: 2e6,
                exp3: 0.1e2,
                bigInt: 9007199254740991n,
                boolean: true,
                'null': null,
                regex: /a/,
                template: String,
                emptyTemplate: ``,
              }
            }
            </script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script setup>
            defineProps({
              a: {
                type: 'String',
                default: 'abc'
              },
            })
            </script>",
            "<script setup>
            defineProps({
              a: {
                type: String,
                default: 'abc'
              },
            })
            </script>",
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(RequirePropTypeConstructor::NAME, RequirePropTypeConstructor::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
