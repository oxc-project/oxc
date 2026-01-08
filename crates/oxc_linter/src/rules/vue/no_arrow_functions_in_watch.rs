use oxc_ast::{
    AstKind,
    ast::{
        CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
        ObjectPropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn no_arrow_functions_in_watch_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("You should not use an arrow function to define a watcher.")
        .with_help("Use a regular function or method shorthand instead of an arrow function.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrowFunctionsInWatch;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows using arrow functions when defining a watcher.
    ///
    /// ### Why is this bad?
    ///
    /// Arrow functions bind `this` lexically, which means they don't have access to the Vue component instance.
    /// In Vue watchers, you often need access to `this` to interact with component data, methods, or other properties.
    /// Using regular functions or method shorthand ensures proper `this` binding.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   watch: {
    ///     foo: () => {},
    ///     bar: {
    ///       handler: () => {}
    ///     },
    ///     baz: [
    ///       (val) => {},
    ///       {
    ///         handler: () => {}
    ///       }
    ///     ]
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   watch: {
    ///     foo() {},
    ///     bar: function() {},
    ///     baz: {
    ///       handler: function() {}
    ///     },
    ///   }
    /// }
    /// </script>
    /// ```
    NoArrowFunctionsInWatch,
    vue,
    correctness
);

impl Rule for NoArrowFunctionsInWatch {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportDefaultDeclaration(export_default_decl) => {
                check_export_default_declaration(&export_default_decl.declaration, ctx);
            }
            AstKind::CallExpression(call_expr) => {
                check_define_component_function(call_expr, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.frameworks_options() != FrameworkOptions::VueSetup
    }
}

fn check_export_default_declaration<'a>(
    export_default_decl: &'a ExportDefaultDeclarationKind<'a>,
    ctx: &LintContext<'a>,
) {
    let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) = export_default_decl else {
        return;
    };

    let Some(watch_obj) = get_watch_object_expression(obj_expr) else {
        return;
    };

    for prop in &watch_obj.properties {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
            handle_watch_value(&obj_prop.value, ctx);
        }
    }
}

fn check_define_component_function<'a>(call_expr: &'a CallExpression<'a>, ctx: &LintContext<'a>) {
    let Some(ident) = call_expr.callee.get_identifier_reference() else {
        return;
    };
    if ident.name.as_str() == "defineComponent" && call_expr.arguments.len() == 1 {
        let arg = &call_expr.arguments[0];
        let Some(Expression::ObjectExpression(obj)) = arg.as_expression() else {
            return;
        };
        let Some(watch_obj) = get_watch_object_expression(obj) else {
            return;
        };
        for prop in &watch_obj.properties {
            handle_watch_inner_property(prop, ctx);
        }
    }
}

fn get_watch_object_expression<'a>(
    obj_expr: &'a ObjectExpression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    let watch_obj = obj_expr.properties.iter().find_map(|item| {
        if let ObjectPropertyKind::ObjectProperty(prop) = item
            && prop.key.static_name().is_some_and(|key| key == "watch")
        {
            Some(&prop.value)
        } else {
            None
        }
    })?;
    let Expression::ObjectExpression(watch_obj) = watch_obj.get_inner_expression() else {
        return None;
    };
    Some(watch_obj)
}

fn handle_watch_inner_property<'a>(inner_prop: &ObjectPropertyKind<'a>, ctx: &LintContext<'a>) {
    let ObjectPropertyKind::ObjectProperty(obj_prop) = inner_prop else {
        return;
    };

    if obj_prop.key.static_name().is_some_and(|key| key == "handler")
        && matches!(obj_prop.value.get_inner_expression(), Expression::ArrowFunctionExpression(_))
    {
        ctx.diagnostic(no_arrow_functions_in_watch_diagnostic(obj_prop.value.span()));
    }
}

fn handle_watch_value<'a>(value: &Expression<'a>, ctx: &LintContext<'a>) {
    match value.get_inner_expression() {
        // Direct arrow function: foo: () => {}
        Expression::ArrowFunctionExpression(arrow_func) => {
            ctx.diagnostic(no_arrow_functions_in_watch_diagnostic(arrow_func.span));
        }
        // Handler object: foo: { handler: () => {} }
        Expression::ObjectExpression(obj_expr) => obj_expr.properties.iter().for_each(|prop| {
            handle_watch_inner_property(prop, ctx);
        }),
        Expression::ArrayExpression(arr_expr) => arr_expr.elements.iter().for_each(|ele| {
            if let Some(expr) = ele.as_expression() {
                handle_watch_value(expr, ctx);
            }
        }),
        _ => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "   <script>
			        export default {}
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "   <script>
			        export default {
			          watch: {}
			        }
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "   <script>
			        export default {
			          watch: {
			            foo() {}
			          },
			        }
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
			        export default {
			          watch: {
			            foo: function() {}
			          },
			        }
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "   <script>
			        export default {
			          watch: {
			            foo() {},
			            bar() {}
			          },
			        }
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "   <script>
			        export default {
			          watch: {
			            foo: function() {},
			            bar: function() {}
			          },
			        }
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "   <script>
			        export default {
			          watch: {
			            ...obj,
			            foo: function() {},
			            bar: function() {}
			          },
			        }
			      </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "   <script>
			      export default {
			        data: {
			          a: 1,
			          b: 2,
			          c: 3,
			          d: 4,
			          e: {
			            f: {
			              g: 5
			            }
			          }
			        },
			        watch: {
			          a: function (val, oldVal) {
			            console.log('new: %s, old: %s', val, oldVal)
			          },
			          b: 'someMethod',
			          c: {
			            handler: function (val, oldVal) {},
			            deep: true
			          },
			          d: {
			            handler: 'someMethod',
			            immediate: true
			          },
			          e: [
			            'handle1',
			            function handle2 (val, oldVal) {},
			            {
			              handler: function handle3 (val, oldVal) {},
			              /* ... */
			            }
			          ],
			          'e.f': function (val, oldVal) { /* ... */ }
			        }
			      }</script>",
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
			        watch: {
			          foo: () => {}
			        },
			      }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
			      export default {
			        watch: {
			          foo() {},
			          bar: () => {}
			        }
			      }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
			      export default {
			        watch: {
			          foo: function() {},
			          bar: () => {}
			        }
			      }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
			      export default {
			        data: {
			          a: 1,
			          b: 2,
			          c: 3,
			          d: 4,
			          e: {
			            f: {
			              g: 5
			            }
			          }
			        },
			        watch: {
			          a: (val, oldVal) => {
			            console.log('new: %s, old: %s', val, oldVal)
			          },
			          b: 'someMethod',
			          c: {
			            handler: function (val, oldVal) {},
			            deep: true
			          },
			          d: {
			            handler: 'someMethod',
			            immediate: true
			          },
			          e: [
			            'handle1',
			            function handle2 (val, oldVal) {},
			            {
			              handler: function handle3 (val, oldVal) {},
			              /* ... */
			            }
			          ],
			          'e.f': function (val, oldVal) { /* ... */ }
			        }
			      }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
			      export default {
			        watch: {
			          foo:{
			            handler: function() {},
			          },
			          bar:{
			            handler: () => {}
			          }
			        }
			      }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
			      export default {
			        watch: {
			          e: [
			            'handle1',
			            (val, oldVal) => { /* ... */ },
			            {
			              handler: (val, oldVal) => { /* ... */ },
			            }
			          ],
			        }
			      }</script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoArrowFunctionsInWatch::NAME, NoArrowFunctionsInWatch::PLUGIN, pass, fail)
        .test_and_snapshot();
}
