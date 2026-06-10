use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpression, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
        TSSignature, TSType, TSTypeName,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::get_declaration_from_reference_id,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::{is_vue_component_options_object, vue_casing::kebab_case},
};

/// Reserved attribute names that cannot be used as prop names, by Vue version.
const RESERVED_VUE3: &[&str] = &["key", "ref"];
const RESERVED_VUE2: &[&str] =
    &["key", "ref", "is", "slot", "slot-scope", "slotScope", "class", "style"];

fn no_reserved_props_diagnostic(prop_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "'{prop_name}' is a reserved attribute and cannot be used as props."
    ))
    .with_label(span)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoReservedPropsConfig {
    /// Vue major version whose reserved attribute set is applied. Vue 2 reserves
    /// more names (`is`, `slot`, `class`, `style`, ...) than Vue 3.
    #[serde(deserialize_with = "deserialize_vue_version")]
    vue_version: u8,
}

impl Default for NoReservedPropsConfig {
    fn default() -> Self {
        Self { vue_version: 3 }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NoReservedProps(Box<NoReservedPropsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow reserved attribute names (e.g. `key`, `ref`) from being used as
    /// prop names.
    ///
    /// ### Why is this bad?
    ///
    /// Vue treats a number of attributes specially (`key` and `ref` in Vue 3;
    /// additionally `is`, `slot`, `slot-scope`, `class` and `style` in Vue 2).
    /// Declaring a prop with one of these names collides with the framework's
    /// own handling and breaks the component.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     ref: String,
    ///     key: String,
    ///   }
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
    ///   }
    /// }
    /// </script>
    /// ```
    NoReservedProps,
    vue,
    correctness,
    config = NoReservedProps,
    version = "1.69.0",
);

impl Rule for NoReservedProps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => self.check_define_props(call, ctx),
            AstKind::ObjectProperty(prop) => {
                if !prop.key.is_specific_static_name("props") {
                    return;
                }
                let parent = ctx.nodes().parent_node(node.id());
                if !is_vue_component_options_object(parent, ctx) {
                    return;
                }
                match prop.value.get_inner_expression() {
                    Expression::ArrayExpression(arr) => self.check_array_props(arr, ctx),
                    Expression::ObjectExpression(obj) => self.check_object_props(obj, ctx),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

impl NoReservedProps {
    fn is_reserved(&self, name: &str) -> bool {
        let reserved = if self.0.vue_version == 2 { RESERVED_VUE2 } else { RESERVED_VUE3 };
        reserved.contains(&name)
    }

    fn report(&self, name: &str, span: Span, ctx: &LintContext) {
        if self.is_reserved(name) {
            ctx.diagnostic(no_reserved_props_diagnostic(&kebab_case(name), span));
        }
    }

    /// `props: ['ref', `key`]` — array declaration.
    fn check_array_props<'a>(&self, arr: &ArrayExpression<'a>, ctx: &LintContext<'a>) {
        for elem in &arr.elements {
            let Some(expr) = elem.as_expression() else { continue };
            let (name, span): (&str, Span) = match expr.get_inner_expression() {
                Expression::StringLiteral(lit) => (lit.value.as_str(), lit.span),
                Expression::TemplateLiteral(tpl) => match tpl.single_quasi() {
                    Some(quasi) => (quasi.as_str(), tpl.span),
                    None => continue,
                },
                _ => continue,
            };
            self.report(name, span, ctx);
        }
    }

    /// `props: { ref: String }` — object declaration.
    fn check_object_props<'a>(&self, obj: &ObjectExpression<'a>, ctx: &LintContext<'a>) {
        for prop_kind in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop_kind else { continue };
            let Some(name) = p.key.static_name() else { continue };
            self.report(name.as_ref(), p.key.span(), ctx);
        }
    }

    /// `<script setup>` `defineProps(...)` — counterpart of upstream `onDefinePropsEnter`.
    fn check_define_props<'a>(&self, call: &CallExpression<'a>, ctx: &LintContext<'a>) {
        if ctx.frameworks_options() != FrameworkOptions::VueSetup {
            return;
        }
        if !call.callee.get_identifier_reference().is_some_and(|id| id.name == "defineProps") {
            return;
        }

        // Runtime declaration: `defineProps([...])` / `defineProps({...})`.
        if let Some(arg) = call.arguments.first().and_then(|arg| arg.as_expression()) {
            match arg.get_inner_expression() {
                Expression::ArrayExpression(arr) => self.check_array_props(arr, ctx),
                Expression::ObjectExpression(obj) => self.check_object_props(obj, ctx),
                _ => {}
            }
            return;
        }

        // Type-only declaration: `defineProps<T>()`.
        if let Some(type_args) = call.type_arguments.as_ref()
            && let Some(first) = type_args.params.first()
        {
            self.check_type_props(first, ctx);
        }
    }

    fn check_type_props<'a>(&self, ty: &TSType<'a>, ctx: &LintContext<'a>) {
        match ty {
            TSType::TSTypeLiteral(literal) => {
                for sig in &literal.members {
                    self.check_signature(sig, ctx);
                }
            }
            TSType::TSUnionType(union) => {
                for member in &union.types {
                    self.check_type_props(member, ctx);
                }
            }
            TSType::TSIntersectionType(intersection) => {
                for member in &intersection.types {
                    self.check_type_props(member, ctx);
                }
            }
            TSType::TSTypeReference(type_ref) => {
                let TSTypeName::IdentifierReference(ident) = &type_ref.type_name else { return };
                let Some(decl) =
                    get_declaration_from_reference_id(ident.reference_id(), ctx.semantic())
                else {
                    return;
                };
                match decl.kind() {
                    AstKind::TSInterfaceDeclaration(interface) => {
                        for sig in &interface.body.body {
                            self.check_signature(sig, ctx);
                        }
                    }
                    AstKind::TSTypeAliasDeclaration(alias) => {
                        self.check_type_props(&alias.type_annotation, ctx);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn check_signature<'a>(&self, sig: &TSSignature<'a>, ctx: &LintContext<'a>) {
        let key = match sig {
            TSSignature::TSPropertySignature(prop) => &prop.key,
            TSSignature::TSMethodSignature(method) => &method.key,
            _ => return,
        };
        let Some(name) = key.static_name() else { return };
        self.report(name.as_ref(), key.span(), ctx);
    }
}

fn deserialize_vue_version<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u8::deserialize(deserializer)?;
    if matches!(version, 2 | 3) {
        Ok(version)
    } else {
        Err(serde::de::Error::custom("vueVersion must be either 2 or 3"))
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
              props: {
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
              props: ['foo']
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
            defineProps({ foo: String })
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
            <script setup lang=\"ts\">
            interface Props {
              foo: String
            }
            defineProps<Props>()
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
              data() {
                return {
                  ref: ''
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
            <script setup lang=\"ts\">
            interface Props {
              is: string,
              slot: string,
              \"slot-scope\": string,
            }
            defineProps<Props>()
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
              props: {
                ref: String,
                key: String,
                is: String,
                slot: String,
                \"slot-scope\": String,
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
                ref: String,
                key: String,
                is: String,
                slot: String,
                \"slot-scope\": String,
              }
            }
            </script>
            ",
            Some(serde_json::json!([{ "vueVersion": 3 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default {
              props: [
                'ref',
                'key',
                'is',
                'slot',
                \"slot-scope\",
                \"slotScope\",
                'class',
                `style`
              ]
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
            defineProps({
              ref: String,
              key: String,
              is: String,
              slot: String,
              \"slot-scope\": String,
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
            defineProps([
              'ref',
              'key',
              'is',
              'slot',
              \"slot-scope\",
            ])
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script setup lang=\"ts\">
            interface Props {
              ref: string,
              key: string,
              is: string,
              slot: string,
              \"slot-scope\": string,
            }
            defineProps<Props>()
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
                ref: String,
                key: String,
                is: String,
                slot: String,
                \"slot-scope\": String,
              }
            }
            </script>
            ",
            Some(serde_json::json!([{ "vueVersion": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default {
              props: [
                'ref',
                'key',
                'is',
                'slot',
                \"slot-scope\",
              ]
            }
            </script>
            ",
            Some(serde_json::json!([{ "vueVersion": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
            <script>
            export default {
              props: [
                \"slotScope\",
                'class',
                `style`
              ]
            }
            </script>
            ",
            Some(serde_json::json!([{ "vueVersion": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // oxc-specific: component forms other than `export default` are also
        // detected via the shared `is_vue_component_options_object` helper.
        (
            "
            <script>
            defineComponent({
              props: {
                ref: String,
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
            <script>
            Vue.component('foo', {
              props: ['key']
            })
            </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoReservedProps::NAME, NoReservedProps::PLUGIN, pass, fail).test_and_snapshot();
}
