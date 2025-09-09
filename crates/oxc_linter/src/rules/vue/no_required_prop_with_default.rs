use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Expression, ObjectPropertyKind, TSMethodSignatureKind, TSSignature, TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use rustc_hash::FxHashSet;
use oxc_span::{Span, GetSpan};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_required_prop_with_default_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRequiredPropWithDefault;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoRequiredPropWithDefault,
    vue,
    style,
    pending
);

impl Rule for NoRequiredPropWithDefault {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };
                if ident.name != "withDefaults" || call_expr.arguments.len() != 2 {
                    return;
                }
                let [first_arg, second_arg] = call_expr.arguments.as_slice() else {
                    return;
                };
                if let Some(first_arg_expr) = first_arg.as_expression()
                    && let Some(second_arg_expr) = second_arg.as_expression()
                {
                    let Expression::ObjectExpression(second_obj_expr) =
                    second_arg_expr.get_inner_expression()
                    else {
                        return;
                    };
                    if second_obj_expr.properties.is_empty() {
                        return;
                    }
                    let mut key_hash = FxHashSet::<String>::default();

                    for prop in second_obj_expr.properties.iter() {
                        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                        && let Some(key) = obj_prop.key.static_name() {
                            key_hash.insert(key.to_string());
                        }
                    }

                    let Expression::CallExpression(first_call_expr) =
                        first_arg_expr.get_inner_expression()
                    else {
                        return;
                    };
                    let Expression::Identifier(first_call_ident) = &first_call_expr.callee else {
                        return;
                    };
                    if first_call_ident.name != "defineProps" {
                        return;
                    }
                    let Some(type_arguments) = first_call_expr.type_arguments.as_ref() else {
                        return;
                    };
                    let Some(first_type_argument) = type_arguments.params.first() else {
                        return;
                    };
                    match first_type_argument {
                        TSType::TSTypeReference(type_ref) => {
                            let TSTypeName::IdentifierReference(ident_ref) = &type_ref.type_name
                            else {
                                return;
                            };
                            let reference = ctx.scoping().get_reference(ident_ref.reference_id());
                            if !reference.is_type() {
                                return;
                            }
                            let reference_node =
                                ctx.symbol_declaration(reference.symbol_id().unwrap());
                            let AstKind::TSInterfaceDeclaration(interface_decl) = reference_node.kind() else {
                                return;
                            };
                            let body = &interface_decl.body;
                            body.body.iter().for_each(|item| {
                                let (key_name, optional) = match item {
                                    TSSignature::TSPropertySignature(prop_sign) => (prop_sign.key.static_name(), prop_sign.optional),
                                    TSSignature::TSMethodSignature(method_sign) if method_sign.kind == TSMethodSignatureKind::Method => (method_sign.key.static_name(), method_sign.optional),
                                    _ => (None, false),
                                };
                                if let Some(key_name) = key_name && !optional {
                                    if key_hash.contains(key_name.as_ref()) {
                                        ctx.diagnostic(no_required_prop_with_default_diagnostic(
                                            item.span()
                                        ));
                                    }
                                }
                            });
                        }
                        TSType::TSTypeLiteral(type_literal) => {
                            type_literal.members.iter().for_each(|item| {
                                let (key_name, optional) = match item {
                                    TSSignature::TSPropertySignature(prop_sign) => (prop_sign.key.static_name(), prop_sign.optional),
                                    TSSignature::TSMethodSignature(method_sign) if method_sign.kind == TSMethodSignatureKind::Method => (method_sign.key.static_name(), method_sign.optional),
                                    _ => (None, false),
                                };
                                if let Some(key_name) = key_name && !optional {
                                    if key_hash.contains(key_name.as_ref()) {
                                        ctx.diagnostic(no_required_prop_with_default_diagnostic(
                                            item.span()
                                        ));
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
            AstKind::ExportDefaultDeclaration(export_default_decl) => {
                let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) = &export_default_decl.declaration else {
                    return;
                };
                // find prop
                let Some(prop) = obj_expr.properties.iter().find(|item| {
                    if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
                    && let Some(key) = obj_prop.key.static_name() {
                        key == "props"
                    } else {
                        false
                    }
                }) else {
                    return;
                };
                if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                && let Expression::ObjectExpression(obj_expr) = obj_prop.value.get_inner_expression() {
                    obj_expr.properties.iter().for_each(|item| {
                        if let ObjectPropertyKind::ObjectProperty(p) = item
                        && let Some(key) = p.key.static_name()
                        && let Expression::ObjectExpression(inner_obj_expr) = p.value.get_inner_expression() {
                            // check inner_obj_expr.properties has 'default' key
                            let mut has_default_key = false;
                            let mut has_required_key = false;
                            for property in inner_obj_expr.properties.iter() {
                                if let ObjectPropertyKind::ObjectProperty(inner_p) = property
                                && let Some(inner_key) = inner_p.key.static_name() {
                                    if inner_key == "default" {
                                        has_default_key = true;
                                    }
                                    if inner_key == "required" {
                                        let Expression::BooleanLiteral(inner_value) = &inner_p.value else {
                                            continue;
                                        };
                                        if inner_value.value {
                                            has_required_key = true;
                                        } else {
                                            break;
                                        }
                                    }

                                    if has_default_key && has_required_key {
                                        ctx.diagnostic(no_required_prop_with_default_diagnostic(
                                            p.span
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                    })
                }

            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_path().extension().is_some_and(|ext| ext == "vue")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    // let pass = vec![(
    //     r#"
	// 						<script setup lang="ts">
	// 							interface A {
	// 								name?: string;
	// 							}
	// 							withDefaults(
	// 								defineProps<A>(),
	// 								{
	// 									name: 'Foo',
	// 								}
	// 							);
	// 						</script>
	// 					"#,
    //     None,
    //     None,
    //     Some(PathBuf::from("test2.vue")),
    // )];

    // let pass = vec![
    //     (
    //         r#"
    //                 <script setup lang="ts">
    //                     interface TestPropType {
    //                         get name(): string
    //                         set name(a: string)
    //                         age?: number
    //                     }
    //                 const props = withDefaults(
    //                 defineProps<TestPropType>(),
    //                     {
    //                         'name': 'world',
    //                     }
    //                 );
    //                 </script>
    //               "#,
    //         None,
    //         None,
    //         Some(PathBuf::from("test.vue")),
    //     ),
    // ];

    // let fail = vec![
    // ];
        
    let pass = vec![
    (
        r#"
                <script setup lang="ts">
                    interface TestPropType {
                    name?: string
                    age?: number
                    }
                    const props = withDefaults(
                    defineProps<TestPropType>(),
                    {
                        name: "World",
                    }
                    );
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        r#"
                <script setup lang="ts">
                    type TestPropType = {
                    name?: string
                    age?: number
                    }
                    const props = withDefaults(
                    defineProps<TestPropType>(),
                    {
                        name: "World",
                    }
                    );
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        r#"
                <script setup lang="ts">
                    interface TestPropType {
                    name?
                    }
                    const props = withDefaults(
                    defineProps<TestPropType>(),
                    {
                        name: "World",
                    }
                    );
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        r#"
                <script setup lang="ts">
                    interface TestPropType {
                    get name(): string
                    set name(a: string)
                    age?: number
                    }
                    const props = withDefaults(
                    defineProps<TestPropType>(),
                    {
                        'name': 'World',
                    }
                    );
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        r#"
                <script setup lang="ts">
                    interface TestPropType {
                    get name(): void
                    age?: number
                    }
                    const props = withDefaults(
                    defineProps<TestPropType>(),
                    {
                        'name': 'World',
                    }
                    );
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        r#"
                <script setup lang="ts">
                    const [name] = 'test'

                    interface TestPropType {
                    [name]: string
                    age?: number
                    }
                    const props = withDefaults(
                    defineProps<TestPropType>(),
                    {
                        [name]: 'World'
                    }
                    );
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        "
                <script>
                export default {
                    props: {
                    name: {
                        required: false,
                        default: 'Hello'
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
                    props: ['name']
                }
                </script>
                ",
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ),
    (
        "
                <script setup>
                    const props = defineProps({
                    name: {
                        required: false,
                        default: 'Hello'
                    }
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
                    interface TestPropType {
                    name?: string
                    }
                    const {name="World"} = defineProps<TestPropType>();
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        r#"
                <script setup lang="ts">
                    const {name="World"} = defineProps<{
                    name?: string
                    }>();
                </script>
                "#,
        None,
        None,
        Some(PathBuf::from("test.vue")),
    ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
    (
        "
                <script setup>
                    const {name='Hello'} = defineProps({
                    name: {
                        required: false
                    }
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
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            name: string
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              name: "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            name: string | number
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              name: "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            'na::me': string
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              'na::me': "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          import nameType from 'name.ts';
    		          interface TestPropType {
    		            name: nameType
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              name: "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            name
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              name: "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            name
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              name: "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            'na\\"me2'
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              'na\\"me2': "World",
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            foo(): void
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              foo() {console.log(123)},
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            readonly name
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              name: 'World',
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            readonly 'name'
    		            age?: number
    		          }
    		          const props = withDefaults(
    		            defineProps<TestPropType>(),
    		            {
    		              'name': 'World',
    		            }
    		          );
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            "
    		        <script>
    		        export default {
    		          props: {
    		            name: {
    		              required: true,
    		              default: 'Hello'
    		            }
    		          }
    		        }
    		        </script>
    		      ",
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
    		        <script>
    		        export default {
    		          props: {
    		            'name': {
    		              required: true,
    		              default: 'Hello'
    		            }
    		          }
    		        }
    		        </script>
    		      ",
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
    		        <script>
    		        import { defineComponent } from 'vue'
    		        export default defineComponent({
    		          props: {
    		            'name': {
    		              required: true,
    		              default: 'Hello'
    		            }
    		          }
    		        })
    		        </script>
    		      ",
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
    		        <script>
    		        import { defineComponent } from 'vue'
    		        export default defineComponent({
    		          props: {
    		            name: {
    		              required: true,
    		              default: 'Hello'
    		            }
    		          }
    		        })
    		        </script>
    		      ",
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
    		        <script>
    		        import { defineComponent } from 'vue'
    		        export default defineComponent({
    		          props: {
    		            name: {
    		              required: true,
    		              default: 'Hello'
    		            }
    		          }
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
    		          const props = defineProps({
    		            name: {
    		              required: true,
    		              default: 'Hello'
    		            }
    		          })
    		        </script>
    		      ",
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
    		        <script setup lang="ts">
    		          interface TestPropType {
    		            name: string
    		          }
    		          const {name="World"} = defineProps<TestPropType>();
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          const {name="World"} = defineProps<{
    		            name: string
    		          }>();
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
    		        <script setup lang="ts">
    		          const {name="World"} = defineProps({
    		            name: {
    		              required: true,
    		            }
    		          });
    		        </script>
    		      "#,
            Some(serde_json::json!([{ "autofix": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let _fix = vec![
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name: string
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name?: string
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name: string | number
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name?: string | number
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            'na::me': string
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'na::me': "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            'na::me'?: string
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'na::me': "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          import nameType from 'name.ts';
			          interface TestPropType {
			            name: nameType
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          import nameType from 'name.ts';
			          interface TestPropType {
			            name?: nameType
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name?
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name?
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            'na\\"me2'
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'na\\"me2': "World",
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            'na\\"me2'?
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'na\\"me2': "World",
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            foo(): void
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              foo() {console.log(123)},
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            foo?(): void
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              foo() {console.log(123)},
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly name
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: 'World',
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly name?
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              name: 'World',
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly 'name'
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'name': 'World',
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly 'name'?
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'name': 'World',
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly 'a'
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              '\\u0061': 'World',
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly 'a'?
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              '\\u0061': 'World',
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly '\\u0061'
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'a': 'World',
			            }
			          );
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            readonly '\\u0061'?
			            age?: number
			          }
			          const props = withDefaults(
			            defineProps<TestPropType>(),
			            {
			              'a': 'World',
			            }
			          );
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            "
			        <script>
			        export default {
			          props: {
			            name: {
			              required: true,
			              default: 'Hello'
			            }
			          }
			        }
			        </script>
			      ",
            "
			        <script>
			        export default {
			          props: {
			            name: {
			              required: false,
			              default: 'Hello'
			            }
			          }
			        }
			        </script>
			      ",
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            "
			        <script>
			        export default {
			          props: {
			            'name': {
			              required: true,
			              default: 'Hello'
			            }
			          }
			        }
			        </script>
			      ",
            "
			        <script>
			        export default {
			          props: {
			            'name': {
			              required: false,
			              default: 'Hello'
			            }
			          }
			        }
			        </script>
			      ",
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            "
			        <script>
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          props: {
			            'name': {
			              required: true,
			              default: 'Hello'
			            }
			          }
			        })
			        </script>
			      ",
            "
			        <script>
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          props: {
			            'name': {
			              required: false,
			              default: 'Hello'
			            }
			          }
			        })
			        </script>
			      ",
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            "
			        <script>
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          props: {
			            name: {
			              required: true,
			              default: 'Hello'
			            }
			          }
			        })
			        </script>
			      ",
            "
			        <script>
			        import { defineComponent } from 'vue'
			        export default defineComponent({
			          props: {
			            name: {
			              required: false,
			              default: 'Hello'
			            }
			          }
			        })
			        </script>
			      ",
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            "
			        <script setup>
			          const props = defineProps({
			            name: {
			              required: true,
			              default: 'Hello'
			            }
			          })
			        </script>
			      ",
            "
			        <script setup>
			          const props = defineProps({
			            name: {
			              required: false,
			              default: 'Hello'
			            }
			          })
			        </script>
			      ",
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name: string
			          }
			          const {name="World"} = defineProps<TestPropType>();
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          interface TestPropType {
			            name?: string
			          }
			          const {name="World"} = defineProps<TestPropType>();
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          const {name="World"} = defineProps<{
			            name: string
			          }>();
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          const {name="World"} = defineProps<{
			            name?: string
			          }>();
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
        (
            r#"
			        <script setup lang="ts">
			          const {name="World"} = defineProps({
			            name: {
			              required: true,
			            }
			          });
			        </script>
			      "#,
            r#"
			        <script setup lang="ts">
			          const {name="World"} = defineProps({
			            name: {
			              required: false,
			            }
			          });
			        </script>
			      "#,
            Some(serde_json::json!([{ "autofix": true }])),
        ),
    ];

    Tester::new(NoRequiredPropWithDefault::NAME, NoRequiredPropWithDefault::PLUGIN, pass, fail)
        .test_and_snapshot();
}
