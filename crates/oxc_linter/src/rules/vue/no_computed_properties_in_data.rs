use std::borrow::Cow;

use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    AstNode, ast_util::get_declaration_from_reference_id, context::LintContext, rule::Rule,
    utils::is_vue_component_options_object,
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
    fn run_once(&self, ctx: &LintContext) {
        ctx.nodes()
            .iter()
            .filter(|node| matches!(node.kind(), AstKind::ObjectExpression(_)))
            .filter(|node| is_vue_component_options_object(node, ctx))
            .filter_map(|node| {
                let AstKind::ObjectExpression(options) = node.kind() else { return None };
                let data_fn_span = data_function_span(options)?;
                let computed_names = collect_computed_names(options);
                (!computed_names.is_empty()).then_some((data_fn_span, computed_names))
            })
            .for_each(|(data_fn_span, computed_names)| {
                check_data_body(ctx, data_fn_span, &computed_names);
            });
    }
}

/// Return the span of the `data` property's function body when `data` is declared
/// as a function (`data() {}` or `data: () => {}`). Object-literal `data: {}`
/// is intentionally not covered — there is no execution context to confuse.
fn data_function_span(options: &ObjectExpression<'_>) -> Option<Span> {
    options
        .properties
        .iter()
        .filter_map(|p| match p {
            ObjectPropertyKind::ObjectProperty(prop)
                if prop.key.is_specific_static_name("data") =>
            {
                Some(prop)
            }
            _ => None,
        })
        .find_map(|prop| match prop.value.get_inner_expression() {
            Expression::FunctionExpression(func) => Some(func.span),
            Expression::ArrowFunctionExpression(arrow) => Some(arrow.span),
            _ => None,
        })
}

fn collect_computed_names(options: &ObjectExpression<'_>) -> FxHashSet<String> {
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
        .filter_map(|prop| prop.key.static_name().map(Cow::into_owned))
        .collect()
}

fn check_data_body(ctx: &LintContext<'_>, data_fn_span: Span, computed_names: &FxHashSet<String>) {
    for node in ctx.nodes().iter() {
        let AstKind::StaticMemberExpression(member) = node.kind() else { continue };
        if !is_this_object(&member.object, ctx) {
            continue;
        }
        if !computed_names.contains(member.property.name.as_str()) {
            continue;
        }
        // The `this.<computed>` access must sit inside the data function body,
        // and the closest enclosing function must be that data function (so
        // nested helpers / inline callbacks aren't reported).
        if !closest_function_is(node, data_fn_span, ctx) {
            continue;
        }
        ctx.diagnostic(no_computed_properties_in_data_diagnostic(member.span()));
    }
}

fn is_this_object(expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    match expr.get_inner_expression() {
        Expression::ThisExpression(_) => true,
        Expression::Identifier(ident) => is_this_alias(ident, ctx),
        _ => false,
    }
}

fn is_this_alias(ident: &IdentifierReference, ctx: &LintContext<'_>) -> bool {
    get_declaration_from_reference_id(ident.reference_id(), ctx.semantic())
        .and_then(|node| match node.kind() {
            AstKind::VariableDeclarator(var) => var.init.as_ref(),
            _ => None,
        })
        .is_some_and(|init| matches!(init.get_inner_expression(), Expression::ThisExpression(_)))
}

fn closest_function_is(node: &AstNode<'_>, data_fn_span: Span, ctx: &LintContext<'_>) -> bool {
    ctx.nodes()
        .ancestors(node.id())
        .find_map(|ancestor| match ancestor.kind() {
            AstKind::Function(func) => Some(func.span == data_fn_span),
            AstKind::ArrowFunctionExpression(arrow) => Some(arrow.span == data_fn_span),
            _ => None,
        })
        .unwrap_or(false)
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
