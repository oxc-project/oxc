use oxc_ast::{
    AstKind,
    ast::{ArrowFunctionExpression, Expression, Function, ReturnStatement, Statement},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

use crate::{
    AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule,
    utils::is_vue_component_options_object,
};

fn expected_boolean_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Expected to return a boolean value in \"{name}\" emits validator."
    ))
    .with_label(span)
}

fn expected_true_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Expected to return a true value in \"{name}\" emits validator."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ReturnInEmitsValidator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that a `return` statement is present in `emits` validators
    /// (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// An `emits` validator must return a boolean indicating whether the
    /// emitted payload is valid. Forgetting to return a value (or returning
    /// only falsy values) makes the validator effectively reject every emit,
    /// breaking the component contract silently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   emits: {
    ///     foo() {
    ///       // missing return
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   emits: {
    ///     foo(payload) {
    ///       return typeof payload === 'string'
    ///     }
    ///   }
    /// }
    /// </script>
    /// ```
    ReturnInEmitsValidator,
    vue,
    correctness,
    version = "next",
);

impl Rule for ReturnInEmitsValidator {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let report_span = match node.kind() {
            AstKind::Function(func) => func.span,
            AstKind::ArrowFunctionExpression(arrow) => arrow.span,
            _ => return,
        };

        let Some(emit_name) = get_emit_validator_name(node, ctx) else { return };

        let (has_return_value, possible_of_return_true) = analyze_return_value(node.kind());
        if possible_of_return_true {
            return;
        }

        let diagnostic = if has_return_value {
            expected_true_diagnostic(report_span, &emit_name)
        } else {
            expected_boolean_diagnostic(report_span, &emit_name)
        };
        ctx.diagnostic(diagnostic);
    }
}

/// Returns the emit name (`foo` in `emits: { foo() {} }`) when `node` is a
/// function/arrow function used as an emits validator. Otherwise returns
/// `None`. Covers both Options API (`emits: {...}` inside a Vue component
/// options object) and Composition API (`defineEmits({...})` in `<script setup>`).
fn get_emit_validator_name(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<String> {
    let nodes = ctx.nodes();
    let parent = nodes.parent_node(node.id());
    let AstKind::ObjectProperty(prop) = parent.kind() else { return None };
    let emit_name = prop.key.static_name()?;

    let obj_node = nodes.parent_node(parent.id());
    if !matches!(obj_node.kind(), AstKind::ObjectExpression(_)) {
        return None;
    }
    let outer = nodes.parent_node(obj_node.id());

    // Options API: function -> ObjectProperty(emit name) -> ObjectExpression(emits body)
    //              -> ObjectProperty("emits") -> ObjectExpression(component options)
    if let AstKind::ObjectProperty(emits_prop) = outer.kind() {
        if !emits_prop.key.is_specific_static_name("emits") {
            return None;
        }
        let component_obj = nodes.parent_node(outer.id());
        if is_vue_component_options_object(component_obj, ctx) {
            return Some(emit_name.into_owned());
        }
        return None;
    }

    // Composition API: function -> ObjectProperty(emit name) -> ObjectExpression
    //                  -> CallExpression(`defineEmits(...)`)
    if ctx.frameworks_options() == FrameworkOptions::VueSetup
        && let AstKind::CallExpression(call) = outer.kind()
        && call.callee.get_identifier_reference().is_some_and(|ident| ident.name == "defineEmits")
    {
        return Some(emit_name.into_owned());
    }

    None
}

/// Walks the body of an emits validator and reports whether at least one
/// return statement carries a value, and whether at least one of those values
/// can be truthy. Mirrors the upstream eslint-plugin-vue logic, with nested
/// functions treated as opaque (their return statements don't count).
fn analyze_return_value(kind: AstKind<'_>) -> (bool, bool) {
    let mut visitor = ReturnVisitor::default();

    match kind {
        AstKind::Function(func) => {
            if let Some(body) = &func.body {
                visitor.visit_function_body(body);
            }
        }
        AstKind::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // Concise body `() => expr`: implicit return of `expr`.
                visitor.has_return_value = true;
                if let Some(expr) = arrow_expression_body(arrow)
                    && !is_falsy(expr)
                {
                    visitor.possible_of_return_true = true;
                }
            } else {
                visitor.visit_function_body(&arrow.body);
            }
        }
        _ => {}
    }

    (visitor.has_return_value, visitor.possible_of_return_true)
}

fn arrow_expression_body<'a>(arrow: &'a ArrowFunctionExpression<'a>) -> Option<&'a Expression<'a>> {
    let stmt = arrow.body.statements.first()?;
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => Some(&expr_stmt.expression),
        _ => None,
    }
}

#[derive(Default)]
struct ReturnVisitor {
    nested_depth: u32,
    has_return_value: bool,
    possible_of_return_true: bool,
}

impl<'a> Visit<'a> for ReturnVisitor {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.nested_depth += 1;
        walk::walk_function(self, func, flags);
        self.nested_depth -= 1;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.nested_depth += 1;
        walk::walk_arrow_function_expression(self, arrow);
        self.nested_depth -= 1;
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        if self.nested_depth == 0
            && let Some(arg) = &stmt.argument
        {
            self.has_return_value = true;
            if !is_falsy(arg) {
                self.possible_of_return_true = true;
            }
        }
        walk::walk_return_statement(self, stmt);
    }
}

fn is_falsy(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::BooleanLiteral(b) => !b.value,
        Expression::NumericLiteral(n) => n.value == 0.0,
        Expression::NullLiteral(_) => true,
        Expression::StringLiteral(s) => s.value.is_empty(),
        Expression::BigIntLiteral(big) => big.is_zero(),
        Expression::Identifier(ident) => matches!(ident.name.as_str(), "undefined" | "NaN"),
        _ => false,
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
                  emits: {
                    foo () {
                      return true
                    },
                    bar: function (e) {
                      return true
                    },
                    baz: (e) => {
                      return e
                    },
                    baz2: (e) => e,
                    qux () {
                      if (foo) {
                        return true
                      } else {
                        return false
                      }
                    },
                    quux: null,
                    corge (evt) {
                      return evt
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
                  emits: {
                    foo () {
                      const options = []
                      this.matches.forEach((match) => {
                        options.push(match)
                      })
                      return options
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
                  emits: ['foo']
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
                  emits: {
                    foo () {
                      const options = []
                      this.matches.forEach(function (match) {
                        options.push(match)
                      })
                      return options
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
                  emits: {
                    a () {
                      return 1n
                    },
                    b: function (e) {
                      return 1
                    },
                    c: (e) => {
                      return 'a'
                    },
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
                defineEmits({
                  foo () {
                    return true
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
                import {Emits1 as Emits} from './test01'
                const emit = defineEmits<Emits>()
                </script>
            "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                defineEmits({
                  foo () {
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
            "
                <script>
                export default {
                  emits: {
                    foo () {
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
                  emits: {
                    foo: function () {
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
                  emits: {
                    foo: () => {
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
                  emits: {
                    foo: () => false
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
                  emits: {
                    foo: function () {
                      function bar () {
                        return this.baz * 2
                      }
                      bar()
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
                  emits: {
                    foo () {
                    },
                    bar () {
                      return
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
                  emits: {
                    foo () {
                      return
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
                  emits: {
                    foo: function () {
                      if (a) {
                        return false
                      } else if (b) {
                        return 0
                      } else if (c) {
                        return null
                      } else if (d) {
                        return ''
                      } else if (e) {
                        return undefined
                      } else if (f) {
                        return NaN
                      } else if (g) {
                        return 0n
                      } else if (h) {
                        return 0x0n
                      }
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
                <script setup>
                defineEmits({
                  foo () {
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(ReturnInEmitsValidator::NAME, ReturnInEmitsValidator::PLUGIN, pass, fail)
        .test_and_snapshot();
}
