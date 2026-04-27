use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_deprecated_data_object_declaration_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Object declaration on `data` property is deprecated.")
        .with_help("Use a function declaration instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDeprecatedDataObjectDeclaration;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow object declaration on `data` (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// In Vue 3, declaring `data` as an object causes the same object to be
    /// shared between every instance of the component, which leads to cross-
    /// instance state pollution. `data` must be a function that returns a
    /// fresh object per instance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   data: {
    ///     foo: 'bar'
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
    ///     return { foo: 'bar' }
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedDataObjectDeclaration,
    vue,
    correctness,
    pending, // fixer will be implemented later
    version = "next",
);

impl Rule for NoDeprecatedDataObjectDeclaration {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectProperty(prop) = node.kind() else { return };

        if !prop.key.is_specific_static_name("data") {
            return;
        }

        if !matches!(prop.value.get_inner_expression(), Expression::ObjectExpression(_)) {
            return;
        }

        let mut ancestors = ctx.nodes().ancestors(node.id());
        let Some(parent) = ancestors.next() else { return };
        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        let Some(grand) = ancestors.next() else { return };
        let in_vue = match grand.kind() {
            AstKind::ExportDefaultDeclaration(_) => true,
            AstKind::CallExpression(call) => call
                .callee
                .get_identifier_reference()
                .is_some_and(|id| id.name == "defineComponent"),
            _ => false,
        };
        if !in_vue {
            return;
        }

        ctx.diagnostic(no_deprecated_data_object_declaration_diagnostic(prop.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                    <script>
                    export default {
                      data: function () {
                        return {
                          foo: 'bar'
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
                      ...foo
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
                      data
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
                      data: () => {
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
                      data () {
                      },
                      methods: {
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
                      data () {
                      },
                      computed: {
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
                    defineComponent({
                      data () {
                        return { foo: 'bar' }
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
                    export default {
                      data () {
                        return {}
                      },
                      methods: {
                        data: { foo: 'bar' }
                      }
                    }
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
                      data: {
                        foo: 'bar'
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
                      data: /*a*/ (/*b*/{
                        foo: 'bar'
                      })
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
                    defineComponent({
                      data: {
                        foo: 'bar'
                      }
                    })
                    </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(
        NoDeprecatedDataObjectDeclaration::NAME,
        NoDeprecatedDataObjectDeclaration::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
