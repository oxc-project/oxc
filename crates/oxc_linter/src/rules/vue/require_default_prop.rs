use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpressionElement, BindingPattern, CallExpression, Expression, ObjectExpression,
        ObjectPattern, ObjectPropertyKind, PropertyKey, TSSignature, TSType,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::Rule,
    utils::{
        find_property, for_each_define_props_type_signature,
        is_vue_component_options_object_excluding_instance,
    },
};

const NATIVE_TYPES: [&str; 7] =
    ["String", "Number", "Boolean", "Function", "Object", "Array", "Symbol"];

fn require_default_prop_diagnostic(span: Span, prop_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prop '{prop_name}' requires default value to be set."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireDefaultProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires default value to be set for props that are not marked as
    /// `required`.
    ///
    /// ### Why is this bad?
    ///
    /// A prop that is neither required nor given a default is implicitly
    /// `undefined` when omitted. Forcing a default keeps the component's
    /// behavior explicit and avoids `undefined` leaking into the template and
    /// logic. `Boolean` props are exempt because they already default to
    /// `false`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   props: {
    ///     name: String,
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
    ///     name: {
    ///       type: String,
    ///       default: '',
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    RequireDefaultProp,
    vue,
    style,
    version = "next",
    short_description = "Requires a default value to be set for props that are not marked as `required`.",
);

/// Tracks how default values may be supplied for `<script setup>` props, so a
/// prop covered by `withDefaults` or a destructure default is not flagged.
#[derive(Default)]
struct PropsContext<'a> {
    destructure: Option<&'a ObjectPattern<'a>>,
    has_with_defaults: bool,
    with_defaults: Option<&'a ObjectExpression<'a>>,
}

impl Rule for RequireDefaultProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj) => {
                if !is_vue_component_options_object_excluding_instance(node, ctx) {
                    return;
                }
                let Some(props_prop) = find_property(obj, "props") else { return };
                if let Expression::ObjectExpression(props_obj) =
                    props_prop.value.get_inner_expression()
                {
                    check_object_props(props_obj, ctx, &PropsContext::default());
                }
            }
            AstKind::CallExpression(call) => {
                if ctx.frameworks_options() != FrameworkOptions::VueSetup {
                    return;
                }
                let Some(ident) = call.callee.get_identifier_reference() else { return };
                match ident.name.as_str() {
                    // A `defineProps` wrapped by `withDefaults` is handled by the
                    // `withDefaults` branch, which also knows the default values.
                    "defineProps" if !is_wrapped_by_with_defaults(node, ctx) => {
                        let props_ctx = destructure_context(node, ctx, false, None);
                        handle_define_props(call, ctx, &props_ctx);
                    }
                    "withDefaults" if call.arguments.len() == 2 => {
                        handle_with_defaults(node, call, ctx);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn handle_with_defaults<'a>(
    node: &AstNode<'a>,
    call: &'a CallExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(first) = call.arguments.first().and_then(|a| a.as_expression()) else { return };
    let Some(second) = call.arguments.get(1).and_then(|a| a.as_expression()) else { return };
    let Expression::CallExpression(define_props) = first.get_inner_expression() else { return };
    if !define_props.callee.get_identifier_reference().is_some_and(|i| i.name == "defineProps") {
        return;
    }
    let with_defaults = match second.get_inner_expression() {
        Expression::ObjectExpression(obj) => Some(obj.as_ref()),
        _ => None,
    };
    let props_ctx = destructure_context(node, ctx, true, with_defaults);
    handle_define_props(define_props, ctx, &props_ctx);
}

fn handle_define_props<'a>(
    call: &CallExpression<'a>,
    ctx: &LintContext<'a>,
    pc: &PropsContext<'a>,
) {
    if let Some(arg) = call.arguments.first().and_then(|a| a.as_expression()) {
        if let Expression::ObjectExpression(obj) = arg.get_inner_expression() {
            check_object_props(obj, ctx, pc);
        }
        return;
    }
    if let Some(type_args) = call.type_arguments.as_ref()
        && let Some(first) = type_args.params.first()
    {
        for_each_define_props_type_signature(first, ctx, &mut |sig| {
            check_type_signature(sig, ctx, pc);
        });
    }
}

/// Resolves the enclosing `const { foo = 1 } = …` destructure (if any) into the
/// set of prop names that already carry a default.
fn destructure_context<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
    has_with_defaults: bool,
    with_defaults: Option<&'a ObjectExpression<'a>>,
) -> PropsContext<'a> {
    let mut pc = PropsContext { has_with_defaults, with_defaults, ..PropsContext::default() };
    let declarator = ctx.nodes().ancestors(node.id()).find_map(|ancestor| {
        if let AstKind::VariableDeclarator(decl) = ancestor.kind() { Some(decl) } else { None }
    });
    if let Some(decl) = declarator
        && let BindingPattern::ObjectPattern(pattern) = &decl.id
    {
        pc.destructure = Some(pattern);
    }
    pc
}

fn is_wrapped_by_with_defaults(node: &AstNode, ctx: &LintContext) -> bool {
    matches!(
        ctx.nodes().parent_node(node.id()).kind(),
        AstKind::CallExpression(call)
            if call.callee.get_identifier_reference().is_some_and(|i| i.name == "withDefaults")
    )
}

fn check_object_props<'a>(
    obj: &ObjectExpression<'a>,
    ctx: &LintContext<'a>,
    pc: &PropsContext<'a>,
) {
    for prop in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else { continue };
        if prop.shorthand {
            continue;
        }
        let value = prop.value.get_inner_expression();
        if !is_without_default_value(value) || is_boolean_prop(value) {
            continue;
        }
        let name = prop.key.static_name();
        if let Some(destructure) = pc.destructure {
            match &name {
                // A computed key whose name is unknown is ignored under a destructure.
                None => continue,
                Some(name) if has_destructure_default(destructure, name.as_ref()) => continue,
                _ => {}
            }
        }
        if let Some(name) = &name {
            ctx.diagnostic(require_default_prop_diagnostic(prop.span(), name.as_ref()));
        } else {
            let display = format!("[{}]", prop.key.span().source_text(ctx.source_text()));
            ctx.diagnostic(require_default_prop_diagnostic(prop.span(), &display));
        }
    }
}

fn has_destructure_default(pattern: &ObjectPattern, name: &str) -> bool {
    pattern.properties.iter().any(|prop| {
        matches!(prop.value, BindingPattern::AssignmentPattern(_))
            && prop.key.static_name().as_deref() == Some(name)
    })
}

fn object_has_key(obj: &ObjectExpression, name: &str) -> bool {
    obj.properties.iter().any(|prop| {
        matches!(prop, ObjectPropertyKind::ObjectProperty(prop)
            if prop.key.static_name().as_deref() == Some(name))
    })
}

fn check_type_signature<'a>(
    signature: &TSSignature<'a>,
    ctx: &LintContext<'a>,
    pc: &PropsContext<'a>,
) {
    let (key, optional) = match signature {
        TSSignature::TSPropertySignature(sig) => (&sig.key, sig.optional),
        TSSignature::TSMethodSignature(sig) => (&sig.key, sig.optional),
        _ => return,
    };
    // A required (non-optional) prop never needs a default.
    if !optional {
        return;
    }
    let Some(name) = key.static_name() else { return };
    if is_single_boolean_type(signature) {
        return;
    }
    // Without `withDefaults`/destructure there is nowhere to attach a default,
    // so a bare `defineProps<T>()` type prop is never reported.
    if !pc.has_with_defaults && pc.destructure.is_none() {
        return;
    }
    if pc.with_defaults.is_some_and(|defaults| object_has_key(defaults, name.as_ref())) {
        return;
    }
    if pc.destructure.is_some_and(|destructure| has_destructure_default(destructure, name.as_ref()))
    {
        return;
    }
    ctx.diagnostic(require_default_prop_diagnostic(signature.span(), name.as_ref()));
}

/// Mirrors upstream `isWithoutDefaultValue`. The value is already unwrapped of
/// `as`/parenthesized expressions by the caller.
fn is_without_default_value(value: &Expression) -> bool {
    match value {
        Expression::ObjectExpression(obj) => !prop_is_required(obj) && !prop_has_default(obj),
        Expression::Identifier(ident) => NATIVE_TYPES.contains(&ident.name.as_str()),
        // A call/member expression is assumed to produce the default value.
        Expression::CallExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => false,
        _ => true,
    }
}

fn prop_is_required(obj: &ObjectExpression) -> bool {
    obj.properties.iter().any(|prop| {
        matches!(prop, ObjectPropertyKind::ObjectProperty(prop)
            if prop.key.static_name().as_deref() == Some("required")
                && matches!(prop.value.get_inner_expression(), Expression::BooleanLiteral(lit) if lit.value))
    })
}

fn prop_has_default(obj: &ObjectExpression) -> bool {
    obj.properties.iter().any(|prop| {
        matches!(prop, ObjectPropertyKind::ObjectProperty(prop)
            if prop.key.static_name().as_deref() == Some("default"))
    })
}

fn is_boolean_prop(value: &Expression) -> bool {
    if is_value_node_of_boolean_type(value) {
        return true;
    }
    let Expression::ObjectExpression(obj) = value else { return false };
    obj.properties.iter().any(|prop| {
        matches!(prop, ObjectPropertyKind::ObjectProperty(prop)
            if matches!(&prop.key, PropertyKey::StaticIdentifier(key) if key.name == "type")
                && is_value_node_of_boolean_type(prop.value.get_inner_expression()))
    })
}

fn is_value_node_of_boolean_type(value: &Expression) -> bool {
    match value {
        Expression::Identifier(ident) => ident.name == "Boolean",
        Expression::ArrayExpression(arr) => {
            let mut elements = arr
                .elements
                .iter()
                .filter(|element| !matches!(element, ArrayExpressionElement::Elision(_)));
            match (elements.next(), elements.next()) {
                (Some(first), None) => first.as_expression().is_some_and(|element| {
                    matches!(element.get_inner_expression(), Expression::Identifier(ident) if ident.name == "Boolean")
                }),
                _ => false,
            }
        }
        _ => false,
    }
}

fn is_single_boolean_type(signature: &TSSignature) -> bool {
    let TSSignature::TSPropertySignature(sig) = signature else { return false };
    sig.type_annotation
        .as_ref()
        .is_some_and(|annotation| matches!(annotation.type_annotation, TSType::TSBooleanKeyword(_)))
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        ("
                  <script>
                    export default {
                      props: {
                        a: {
                          type: Number,
                          required: true
                        },
                        b: {
                          type: Number,
                          default: 0
                        },
                        c: {
                          type: Number,
                          required: false,
                          default: 0
                        },
                        d: {
                          type: String,
                          required: false,
                          'default': 'lorem'
                        },
                        e: {
                          type: Boolean
                        },
                        f: {
                          type: Boolean,
                          required: false
                        },
                        g: {
                          type: Boolean,
                          default: true
                        },
                        h: {
                          type: [Boolean]
                        },
                        i: Boolean,
                        j: [Boolean],
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script>
                    export default {
                      props: {
                        ...x,
                        a: {
                          ...y,
                          type: Number,
                          required: true
                        },
                        b: {
                          type: Number,
                          default: 0
                        }
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script>
                    const x = {
                      type: Object,
                      default() {
                        return {
                          foo: 1,
                          bar: 2
                        }
                      }
                    }
                    export default {
                      props: {
                        a: {
                          ...x,
                          default() {
                            return {
                              ...x.default(),
                              baz: 3
                            }
                          }
                        }
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        (r#"
                  <script lang="ts">
                    export default (Vue as VueConstructor<Vue>).extend({
                      props: {
                        a: {
                          type: String,
                          required: true
                        } as PropOptions<string>
                      }
                    });
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
        (r#"
                  <script lang="ts">
                    export default Vue.extend({
                      props: {
                        a: {
                          type: String,
                          required: true
                        } as PropOptions<string>
                      }
                    });
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script>
                    export default {
                      props: {
                        bar,
                        baz: prop,
                        baz1: prop.foo,
                        bar2: foo()
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script>
                    export default {
                      props: {
                        foo: {
                          ...foo,
                          default: 0
                        },
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("destructuring-test.vue"))),
        ("
                  <script>
                    export default {
                      props: {
                        foo: {
                          [bar]: true,
                          default: 0
                        },
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("unknown-prop-details-test.vue"))),
        ("
                  <script>
                  export default {
                    props: ['foo']
                  }
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script>
                    export default {
                      props: {
                        a: {
                          type: [,Boolean]
                        },
                        b: [,Boolean],
                      }
                    }
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script setup>
                  defineProps({
                    foo: {
                      type: String,
                      default: ''
                    }
                  })
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        ("
                  <script setup>
                  defineProps(['foo'])
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))),
        (r#"
                  <script setup lang="ts">
                  interface Props {
                    foo?: number
                  }
                  defineProps<Props>()
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (r#"
                  <script setup lang="ts">
                  interface Props {
                    foo?: number
                  }
                  withDefaults(defineProps<Props>(), {foo:42})
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (r#"
                  <script setup lang="ts">
                  interface Props {
                    foo?: number
                  }
                  defineProps<Props>({
                    foo:{
                      default: 42
                    }
                  })
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (r#"
                  <template>
                    <div>
                      {{ required }}
                      {{ optional }}
                    </div>
                  </template>
            
                  <script setup lang="ts">
                  import { defineProps, withDefaults } from 'vue';
            
                  interface Props {
                    required: boolean;
                    optional?: boolean;
                  }
            
                  const props = withDefaults(defineProps<Props>(), {
                    optional: false,
                  });
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (r#"
                  <script setup lang="ts">
                  interface Props {
                    optional?: boolean;
                  }
            
                  const props = defineProps<Props>();
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (r#"
                  <script setup lang="ts">
                  const defaultProps = {
                    foo: 'foo',
                  }
                  withDefaults(defineProps<{
                    foo: string;
                    bar?: number;
                  }>(), {
                    ...defaultProps,
                    bar: 42,
                  })
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        ("
                  <script setup>
                  const {foo=42,bar=42} = defineProps({foo: Number, bar: {type: Number}})
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions },
        ("
                  <script setup>
                  const {foo,bar} = defineProps({foo: Boolean, bar: {type: Boolean}})
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions },
        ("
                  <script setup>
                  // ignore
                  const {bar = 42, foo = 42} = defineProps({[x]: Number, bar: {type: Number}})
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions },
        ("
                  <script setup>
                  const {bar=42} = defineProps({foo: {type: Number, required: true}, bar: {type: Number, required: false}})
                  </script>
                  ", None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions },
        (r#"
                  <script setup lang="ts">
                  const {foo = 42, bar} = defineProps<{foo?: number; bar: number}>()
                  </script>
                  "#, None, None, Some(PathBuf::from("test.vue"))), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
    ];

    let fail = vec![
        (
            r#"
                  <script setup lang="ts">
                  type Props = { foo?: number }
                  withDefaults(defineProps<Props>(), {})
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("type-alias.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  interface A { foo?: number }
                  interface B { bar?: number }
                  withDefaults(defineProps<A & B>(), {})
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("intersection.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  type Props = { foo?: number } | { bar?: number }
                  withDefaults(defineProps<Props>(), {})
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("union.vue")),
        ),
        (
            "
                  <script>
                    export default {
                      props: {
                        a: Number,
                        b: [Number, String],
                        c: {
                          type: Number
                        },
                        d: {
                          type: Number,
                          required: false
                        },
                        e: [Boolean, String],
                        f: {
                          type: [Boolean, String],
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
            r#"
                  <script lang="ts">
                    export default (Vue as VueConstructor<Vue>).extend({
                      props: {
                        a: {
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
                    export default Vue.extend({
                      props: {
                        a: {
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
            "
                  <script>
                    export default {
                      props: {
                        a: String,
                        'b': String,
                        ['c']: String,
                        [`d`]: String,
                      }
                    };
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
                        [foo]: String,
                        [bar()]: String,
                        [baz.baz]: String,
                      }
                    };
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
                          ...foo
                        },
                      }
                    }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("destructuring-test.vue")),
        ),
        (
            "
                  <script>
                    export default {
                      props: {
                        foo: {
                          [bar]: true
                        },
                      }
                    }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("unknown-prop-details-test.vue")),
        ),
        (
            "
                  <script>
                    export default {
                      props: {
                        bar,
                        baz: prop?.foo,
                        bar1: foo?.(),
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
                  <script setup>
                  defineProps({
                    foo: String
                  })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            r#"
                  <script setup lang="ts">
                  const defaultProps = {
                    foo: 'foo',
                  }
                  withDefaults(defineProps<{
                    foo: string;
                    bar?: number;
                  }>(), {
                    ...defaultProps,
                  })
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            r#"
                  <script setup lang="ts">
                  interface Props {
                    foo?: number
                  }
                  withDefaults(defineProps<Props>(), {bar:42})
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            "
                  <script setup>
                  const {foo,bar} = defineProps({foo: Boolean, bar: {type: String}})
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            "
                  <script setup>
                  const {foo,bar} = defineProps({foo: Number, bar: {type: Number}})
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions },
        (
            r#"
                  <script setup lang="ts">
                  const {foo, bar} = defineProps<{foo?: number; bar: number}>()
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("type-with-props-destructure.vue")),
        ), // { "parser": vueEslintParser, ...languageOptions, "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
    ];

    Tester::new(RequireDefaultProp::NAME, RequireDefaultProp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
