use oxc_ast::{
    AstKind,
    ast::{
        BindingPatternKind, ExportDefaultDeclarationKind, Expression, ObjectExpression,
        ObjectPropertyKind, TSMethodSignatureKind, TSSignature, TSType, TSTypeName,
        VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn no_required_prop_with_default_diagnostic(span: Span, prop_name: &str) -> OxcDiagnostic {
    let msg = format!("Prop \"{prop_name}\" should be optional.");
    OxcDiagnostic::warn(msg)
        .with_help("Remove the `required: true` option, or drop the `required` key entirely to make this prop optional.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRequiredPropWithDefault;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce props with default values to be optional.
    ///
    /// ### Why is this bad?
    ///
    /// If a prop is declared with a default value, whether it is required or not,
    /// we can always skip it in actual use. In that situation, the default value would be applied.
    /// So, a required prop with a default value is essentially the same as an optional prop.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup lang="ts">
    /// const props = withDefaults(
    ///   defineProps<{
    ///     name: string | number
    ///     age?: number
    ///   }>(),
    ///   {
    ///     name: 'Foo',
    ///   }
    /// );
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup lang="ts">
    /// const props = withDefaults(
    ///   defineProps<{
    ///     name?: string | number
    ///     age?: number
    ///   }>(),
    ///   {
    ///     name: 'Foo',
    ///   }
    /// );
    /// </script>
    /// ```
    NoRequiredPropWithDefault,
    vue,
    suspicious,
    pending
);

impl Rule for NoRequiredPropWithDefault {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let is_vue = ctx.file_extension().is_some_and(|ext| ext == "vue");
        if is_vue {
            self.run_on_vue(node, ctx);
        } else {
            self.check_define_component(node, ctx);
        }
    }
}

impl NoRequiredPropWithDefault {
    fn run_on_vue<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if ctx.frameworks_options() == FrameworkOptions::VueSetup {
            self.run_on_setup(node, ctx);
        } else {
            self.run_on_composition(node, ctx);
        }
    }

    #[expect(clippy::unused_self)]
    fn check_define_component(&self, node: &AstNode<'_>, ctx: &LintContext<'_>) {
        // only check `defineComponent` method
        // e.g. `let component = defineComponent({ props: { name: { required: true, default: 'a' } } })`
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };
        if ident.name.as_str() == "defineComponent" && call_expr.arguments.len() == 1 {
            let arg = &call_expr.arguments[0];
            let Some(Expression::ObjectExpression(obj)) = arg.as_expression() else {
                return;
            };
            handle_object_expression(ctx, obj);
        }
    }

    fn run_on_setup<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };

        match ident.name.as_str() {
            "defineProps" => {
                if let Some(arge) = call_expr.arguments.first() {
                    let Some(Expression::ObjectExpression(obj)) = arge.as_expression() else {
                        return;
                    };
                    // Here we need to consider the following two examples
                    // 1. const props = defineProps({ name: { required: true, default: 'a' } })
                    // 2. const { name = 'a' } =  defineProps({ name: { required: true } })
                    let key_hash =
                        collect_hash_from_variable_declarator(ctx, node).unwrap_or_default();
                    handle_prop_object(ctx, obj, Some(&key_hash));
                }
                if call_expr.arguments.is_empty() {
                    // if `defineProps` is used without arguments, we need to check the type arguments
                    // e.g. `const { name = 'a' } = defineProps<IProp>()`
                    let Some(type_args) = &call_expr.type_arguments else {
                        return;
                    };
                    let Some(first_type_argument) = type_args.params.first() else {
                        return;
                    };
                    if let Some(key_hash) = collect_hash_from_variable_declarator(ctx, node) {
                        handle_type_argument(ctx, first_type_argument, &key_hash);
                    }
                }
            }
            "withDefaults" if call_expr.arguments.len() == 2 => {
                let [first_arg, second_arg] = call_expr.arguments.as_slice() else {
                    return;
                };
                if let (Some(first_arg_expr), Some(second_arg_expr)) =
                    (first_arg.as_expression(), second_arg.as_expression())
                {
                    let Expression::ObjectExpression(second_obj_expr) =
                        second_arg_expr.get_inner_expression()
                    else {
                        return;
                    };
                    let Some(key_hash) = collect_hash_from_object_expr(second_obj_expr) else {
                        return;
                    };
                    process_define_props_call(ctx, first_arg_expr, &key_hash);
                }
            }
            _ => {
                self.check_define_component(node, ctx);
            }
        }
    }

    fn run_on_composition<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportDefaultDeclaration(export_default_decl) => {
                let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) =
                    &export_default_decl.declaration
                else {
                    return;
                };
                handle_object_expression(ctx, obj_expr);
            }
            _ => {
                self.check_define_component(node, ctx);
            }
        }
    }
}

fn collect_hash_from_object_expr(obj: &ObjectExpression) -> Option<FxHashSet<String>> {
    if obj.properties.is_empty() {
        return None;
    }
    let key_hash = obj
        .properties
        .iter()
        .filter_map(|item| {
            if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
                && let Some(key) = obj_prop.key.static_name()
            {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect();
    Some(key_hash)
}

fn collect_hash_from_variable_declarator(
    ctx: &LintContext<'_>,
    node: &AstNode,
) -> Option<FxHashSet<String>> {
    let var_decl = get_first_variable_decl_ancestor(ctx, node)?;
    let BindingPatternKind::ObjectPattern(obj_pattern) = &var_decl.id.kind else {
        return None;
    };
    let key_hash: FxHashSet<String> = obj_pattern
        .properties
        .iter()
        .filter_map(|prop| {
            if matches!(prop.value.kind, BindingPatternKind::AssignmentPattern(_)) {
                prop.key.static_name()
            } else {
                None
            }
        })
        .map(|key| key.to_string())
        .collect();
    Some(key_hash)
}

fn get_first_variable_decl_ancestor<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode,
) -> Option<&'a VariableDeclarator<'a>> {
    ctx.nodes().ancestors(node.id()).find_map(|ancestor| {
        if let AstKind::VariableDeclarator(var_decl) = ancestor.kind() {
            Some(var_decl)
        } else {
            None
        }
    })
}

fn process_define_props_call(
    ctx: &LintContext,
    first_arg_expr: &Expression,
    key_hash: &FxHashSet<String>,
) {
    let Expression::CallExpression(first_call_expr) = first_arg_expr.get_inner_expression() else {
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

    handle_type_argument(ctx, first_type_argument, key_hash);
}

fn handle_type_argument(ctx: &LintContext, ts_type: &TSType, key_hash: &FxHashSet<String>) {
    match ts_type {
        // e.g. `const props = defineProps<IProps>()`
        TSType::TSTypeReference(type_ref) => {
            let TSTypeName::IdentifierReference(ident_ref) = &type_ref.type_name else {
                return;
            };
            // we need to find the reference of type_ref
            let reference = ctx.scoping().get_reference(ident_ref.reference_id());
            if !reference.is_type() {
                return;
            }
            let Some(symbol_id) = reference.symbol_id() else {
                return;
            };
            let reference_node = ctx.symbol_declaration(symbol_id);
            let AstKind::TSInterfaceDeclaration(interface_decl) = reference_node.kind() else {
                return;
            };
            let body = &interface_decl.body;
            body.body.iter().for_each(|item| {
                let (key_name, optional) = match item {
                    TSSignature::TSPropertySignature(prop_sign) => {
                        (prop_sign.key.static_name(), prop_sign.optional)
                    }
                    TSSignature::TSMethodSignature(method_sign)
                        if method_sign.kind == TSMethodSignatureKind::Method =>
                    {
                        (method_sign.key.static_name(), method_sign.optional)
                    }
                    _ => (None, false),
                };
                if let Some(key_name) = key_name
                    && !optional
                    && key_hash.contains(key_name.as_ref())
                {
                    ctx.diagnostic(no_required_prop_with_default_diagnostic(
                        item.span(),
                        key_name.as_ref(),
                    ));
                }
            });
        }
        // e.g. `const props = defineProps<{ name: string }>()`
        TSType::TSTypeLiteral(type_literal) => {
            type_literal.members.iter().for_each(|item| {
                let (key_name, optional) = match item {
                    TSSignature::TSPropertySignature(prop_sign) => {
                        (prop_sign.key.static_name(), prop_sign.optional)
                    }
                    TSSignature::TSMethodSignature(method_sign)
                        if method_sign.kind == TSMethodSignatureKind::Method =>
                    {
                        (method_sign.key.static_name(), method_sign.optional)
                    }
                    _ => (None, false),
                };
                if let Some(key_name) = key_name
                    && !optional
                    && key_hash.contains(key_name.as_ref())
                {
                    ctx.diagnostic(no_required_prop_with_default_diagnostic(
                        item.span(),
                        key_name.as_ref(),
                    ));
                }
            });
        }
        _ => {}
    }
}

fn handle_object_expression(ctx: &LintContext, obj: &ObjectExpression) {
    let Some(prop) = obj.properties.iter().find(|item| {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
            && let Some(key) = obj_prop.key.static_name()
        {
            key == "props"
        } else {
            false
        }
    }) else {
        return;
    };
    let ObjectPropertyKind::ObjectProperty(prop_obj) = prop else {
        return;
    };
    let Expression::ObjectExpression(prop_obj_expr) = prop_obj.value.get_inner_expression() else {
        return;
    };
    handle_prop_object(ctx, prop_obj_expr, None);
}

fn handle_prop_object(
    ctx: &LintContext,
    obj: &ObjectExpression,
    key_hash: Option<&FxHashSet<String>>,
) {
    obj.properties.iter().for_each(|v| {
        if let ObjectPropertyKind::ObjectProperty(inner_prop) = v
            && let Some(inner_key) = inner_prop.key.static_name()
            && let Expression::ObjectExpression(inner_prop_value_expr) =
                inner_prop.value.get_inner_expression()
        {
            let mut has_default_key = false;
            let mut has_required_key = false;

            // Sometimes the default value comes from the `ObjectPattern` of a `VariableDeclarator`,
            // e.g. `const { name = 2 } = defineProps()`
            if key_hash.is_some_and(|hash| hash.contains(inner_key.as_ref())) {
                has_default_key = true;
            }

            for property in &inner_prop_value_expr.properties {
                if let ObjectPropertyKind::ObjectProperty(item_obj) = property
                    && let Some(item_key) = item_obj.key.static_name()
                {
                    if item_key == "default" {
                        has_default_key = true;
                    }
                    if item_key == "required" {
                        let Expression::BooleanLiteral(inner_value) = &item_obj.value else {
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
                            inner_prop.span(),
                            inner_key.as_ref(),
                        ));
                        break;
                    }
                }
            }
        }
    });
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            r#"
            <script setup lang="ts">
                const { laps } = defineProps<{
                    laps: unknown[];
                }>();
            </script>
                "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
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
        (
            "   <script lang='ts'>
                export interface ComponentProps {
                    name?: string;
                }
                </script>
                <script setup lang='ts'>
                    const {name='Hello'} = defineProps<ComponentProps>()
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
                const a = defineComponent({
                    props: {
                        'name': {
                        required: true,
                        default: 'Hello'
                        }
                    }
                    })
            ",
            None,
            None,
            Some(PathBuf::from("test.ts")),
        ),
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

    // let _fix = vec![
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name: string
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name?: string
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name: string | number
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name?: string | number
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            'na::me': string
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'na::me': "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            'na::me'?: string
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'na::me': "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          import nameType from 'name.ts';
    // 		          interface TestPropType {
    // 		            name: nameType
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          import nameType from 'name.ts';
    // 		          interface TestPropType {
    // 		            name?: nameType
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name?
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name?
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            'na\\"me2'
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'na\\"me2': "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            'na\\"me2'?
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'na\\"me2': "World",
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            foo(): void
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              foo() {console.log(123)},
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            foo?(): void
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              foo() {console.log(123)},
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly name
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly name?
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              name: 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly 'name'
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'name': 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly 'name'?
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'name': 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly 'a'
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              '\\u0061': 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly 'a'?
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              '\\u0061': 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly '\\u0061'
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'a': 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            readonly '\\u0061'?
    // 		            age?: number
    // 		          }
    // 		          const props = withDefaults(
    // 		            defineProps<TestPropType>(),
    // 		            {
    // 		              'a': 'World',
    // 		            }
    // 		          );
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         "
    // 		        <script>
    // 		        export default {
    // 		          props: {
    // 		            name: {
    // 		              required: true,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        }
    // 		        </script>
    // 		      ",
    //         "
    // 		        <script>
    // 		        export default {
    // 		          props: {
    // 		            name: {
    // 		              required: false,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        }
    // 		        </script>
    // 		      ",
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         "
    // 		        <script>
    // 		        export default {
    // 		          props: {
    // 		            'name': {
    // 		              required: true,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        }
    // 		        </script>
    // 		      ",
    //         "
    // 		        <script>
    // 		        export default {
    // 		          props: {
    // 		            'name': {
    // 		              required: false,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        }
    // 		        </script>
    // 		      ",
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         "
    // 		        <script>
    // 		        import { defineComponent } from 'vue'
    // 		        export default defineComponent({
    // 		          props: {
    // 		            'name': {
    // 		              required: true,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        })
    // 		        </script>
    // 		      ",
    //         "
    // 		        <script>
    // 		        import { defineComponent } from 'vue'
    // 		        export default defineComponent({
    // 		          props: {
    // 		            'name': {
    // 		              required: false,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        })
    // 		        </script>
    // 		      ",
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         "
    // 		        <script>
    // 		        import { defineComponent } from 'vue'
    // 		        export default defineComponent({
    // 		          props: {
    // 		            name: {
    // 		              required: true,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        })
    // 		        </script>
    // 		      ",
    //         "
    // 		        <script>
    // 		        import { defineComponent } from 'vue'
    // 		        export default defineComponent({
    // 		          props: {
    // 		            name: {
    // 		              required: false,
    // 		              default: 'Hello'
    // 		            }
    // 		          }
    // 		        })
    // 		        </script>
    // 		      ",
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         "
    // 		        <script setup>
    // 		          const props = defineProps({
    // 		            name: {
    // 		              required: true,
    // 		              default: 'Hello'
    // 		            }
    // 		          })
    // 		        </script>
    // 		      ",
    //         "
    // 		        <script setup>
    // 		          const props = defineProps({
    // 		            name: {
    // 		              required: false,
    // 		              default: 'Hello'
    // 		            }
    // 		          })
    // 		        </script>
    // 		      ",
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name: string
    // 		          }
    // 		          const {name="World"} = defineProps<TestPropType>();
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          interface TestPropType {
    // 		            name?: string
    // 		          }
    // 		          const {name="World"} = defineProps<TestPropType>();
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          const {name="World"} = defineProps<{
    // 		            name: string
    // 		          }>();
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          const {name="World"} = defineProps<{
    // 		            name?: string
    // 		          }>();
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    //     (
    //         r#"
    // 		        <script setup lang="ts">
    // 		          const {name="World"} = defineProps({
    // 		            name: {
    // 		              required: true,
    // 		            }
    // 		          });
    // 		        </script>
    // 		      "#,
    //         r#"
    // 		        <script setup lang="ts">
    // 		          const {name="World"} = defineProps({
    // 		            name: {
    // 		              required: false,
    // 		            }
    // 		          });
    // 		        </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "autofix": true }])),
    //     ),
    // ];

    Tester::new(NoRequiredPropWithDefault::NAME, NoRequiredPropWithDefault::PLUGIN, pass, fail)
        .test_and_snapshot();
}
