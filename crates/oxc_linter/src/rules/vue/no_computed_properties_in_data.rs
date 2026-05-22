use std::borrow::Cow;

use rustc_hash::FxHashSet;

use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{is_this_object, is_vue_component_options_object},
};

fn no_computed_properties_in_data_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The computed property cannot be used in `data()` because it is before initialization.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoComputedPropertiesInData;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow accessing computed properties inside `data()`.
    ///
    /// ### Why is this bad?
    ///
    /// `data()` runs **before** computed properties are initialized, so
    /// `this.<computedName>` evaluates to `undefined` and leaves silently
    /// broken state in the component instance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   data() {
    ///     const foo = this.foo // `foo` is a computed property
    ///     return {}
    ///   },
    ///   computed: {
    ///     foo() {}
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   data() {
    ///     const foo = this.foo // `foo` is a prop, not a computed
    ///     return {}
    ///   },
    ///   props: ['foo']
    /// }
    /// </script>
    /// ```
    NoComputedPropertiesInData,
    vue,
    correctness,
    version = "next",
);

impl Rule for NoComputedPropertiesInData {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(member) = node.kind() else { return };
        if !is_this_object(&member.object, ctx) {
            return;
        }

        let Some(options) = enclosing_data_options(node, ctx) else { return };

        if !collect_computed_names(options).contains(member.property.name.as_str()) {
            return;
        }

        ctx.diagnostic(no_computed_properties_in_data_diagnostic(member.span()));
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

/// Return the Vue component options object when `node` sits inside the body of a
/// `data` function (`data() {}` / `data: function() {}` / `data: () => {}`)
/// that is itself declared on a Vue component options object.
fn enclosing_data_options<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let nodes = ctx.nodes();

    let function_node = nodes.ancestors(node.id()).find(|ancestor| {
        matches!(ancestor.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_))
    })?;

    let prop_node = nodes.parent_node(function_node.id());
    let AstKind::ObjectProperty(prop) = prop_node.kind() else { return None };
    if !prop.key.is_specific_static_name("data") {
        return None;
    }

    let options_node = nodes.parent_node(prop_node.id());
    let AstKind::ObjectExpression(options) = options_node.kind() else { return None };
    if !is_vue_component_options_object(options_node, ctx) {
        return None;
    }
    Some(options)
}

fn collect_computed_names<'a>(options: &ObjectExpression<'a>) -> FxHashSet<&'a str> {
    let Some(computed_obj) = options.properties.iter().find_map(|p| match p {
        ObjectPropertyKind::ObjectProperty(prop)
            if prop.key.is_specific_static_name("computed") =>
        {
            match prop.value.get_inner_expression() {
                Expression::ObjectExpression(obj) => Some(obj),
                _ => None,
            }
        }
        _ => None,
    }) else {
        return FxHashSet::default();
    };

    computed_obj
        .properties
        .iter()
        .filter_map(|entry| match entry {
            ObjectPropertyKind::ObjectProperty(prop) if !prop.computed => Some(prop),
            _ => None,
        })
        .filter_map(|prop| match prop.key.static_name()? {
            Cow::Borrowed(name) => Some(name),
            // Computed keys backed by literals (e.g. `0: foo`) produce an owned
            // string; they are not valid identifiers to reference via `this`, so skip.
            Cow::Owned(_) => None,
        })
        .collect()
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;

    let pass = vec![
        (
            "<script>
            export default {
              data() {
                const foo = this.foo
                return {}
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
            export default {
              data() {
                const foo = this.foo()
                return {}
              },
              methods: {
                foo() {}
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
            export default {
              props: ['foo'],
              data() {
                const foo = this.foo
                return {}
              },
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
            export default {
              data: {
                foo: this.foo
              },
              computed: {
                foo () {}
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "<script>
            export default {
              data() {
                const foo = this.foo
                return  {}
              },
              computed: {
                foo () {}
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>
            export default {
              data() {
                const vm = this
                const foo = vm.foo
                return  {}
              },
              computed: {
                foo () {}
              }
            }
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoComputedPropertiesInData::NAME, NoComputedPropertiesInData::PLUGIN, pass, fail)
        .test_and_snapshot();
}
