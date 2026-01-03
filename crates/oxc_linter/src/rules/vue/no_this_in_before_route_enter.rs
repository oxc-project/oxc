use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Expression, Function, ObjectPropertyKind, ThisExpression},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_this_in_before_route_enter_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`beforeRouteEnter` does NOT have access to `this` component instance.")
        .with_help("Use the callback's `vm` parameter instead of `this` in `beforeRouteEnter`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThisInBeforeRouteEnter;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `this` usage in a `beforeRouteEnter` method.
    ///
    /// This rule is only relevant when using `vue-router`.
    ///
    /// ### Why is this bad?
    ///
    /// Inside a `beforeRouteEnter` method, there is no access to `this`.
    /// See [the vue-router docs](https://router.vuejs.org/guide/advanced/navigation-guards.html#in-component-guards).
    /// This behavior isn't obvious, and so this lint rule can help prevent runtime errors in some cases.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```js
    /// export default {
    ///   beforeRouteEnter(to, from, next) {
    ///     this.a; // Error: 'this' is not available
    ///     next();
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// export default {
    ///   beforeRouteEnter(to, from, next) {
    ///     // anything without `this`
    ///   }
    /// }
    /// ```
    NoThisInBeforeRouteEnter,
    vue,
    correctness,
);

impl Rule for NoThisInBeforeRouteEnter {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_default_decl) = node.kind() else { return };
        let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) =
            &export_default_decl.declaration
        else {
            return;
        };

        let before_route_enter_prop = obj_expr.properties.iter().find_map(|prop| {
            if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                && let Some(key_name) = obj_prop.key.static_name()
                && key_name == "beforeRouteEnter"
            {
                Some(obj_prop)
            } else {
                None
            }
        });

        if let Some(before_route_enter_prop) = before_route_enter_prop {
            let function_body = match &before_route_enter_prop.value {
                Expression::FunctionExpression(func_expr) => func_expr.body.as_ref(),
                _ => return,
            };

            let Some(function_body) = function_body else {
                return;
            };

            let mut finder = ThisFinder::new();
            finder.visit_function_body(function_body);
            for span in finder.found_this_expressions {
                ctx.diagnostic(no_this_in_before_route_enter_diagnostic(span));
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

struct ThisFinder {
    found_this_expressions: Vec<Span>,
}

impl ThisFinder {
    fn new() -> Self {
        Self { found_this_expressions: Vec::new() }
    }
}

impl<'a> Visit<'a> for ThisFinder {
    fn visit_this_expression(&mut self, expr: &ThisExpression) {
        self.found_this_expressions.push(expr.span);
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter() {
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter() {
                const variable = 42;
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter() {
                someFunction(42)
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
            <script>
            export default {
              beforeRouteEnter(to, from, next) {
                function test() {
                  this.a;
                }
              }
            };
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
            <script>
            let a = {
              beforeRouteEnter() {
                this.a;
              }
            };
            export default a;
            </script>",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r"
            export default {
              beforeRouteEnter(to, from, next) {
                this.a;
              }
            };
            ",
            None,
            None,
            Some(PathBuf::from("test.js")),
        ),
    ];

    let fail = vec![
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter() {
                this.xxx();
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter: function() {
                this.method();
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter() {
                this.attr = this.method();
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter: function() {
                this.attr = this.method();
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        // this inside if condition
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter() {
                if (this.method()) {}
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
            <script>
            export default {
              data () {
                return {
                  greeting: "Hello"
                };
              },
              beforeRouteEnter: function() {
                if (true) { this.method(); }
              }
            };
            </script>"#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoThisInBeforeRouteEnter::NAME, NoThisInBeforeRouteEnter::PLUGIN, pass, fail)
        .test_and_snapshot();
}
