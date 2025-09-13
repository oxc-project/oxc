use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, Expression, IdentifierReference, MemberExpression,
        VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, frameworks::FrameworkOptions, rule::Rule};

fn multiple_arguments_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected multiple arguments.")
        .with_help("Pass only one argument to the slot function.")
        .with_label(span)
}

fn spread_argument_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected spread argument.")
        .with_help("Do not use spread arguments when calling slot functions.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMultipleSlotArgs;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow passing multiple arguments to scoped slots.
    ///
    /// ### Why is this bad?
    ///
    /// Users have to use the arguments in fixed order and cannot omit the ones they don't need.
    /// e.g. if you have a slot that passes in 5 arguments but the user actually only need the last 2 of them,
    /// they will have to declare all 5 just to use the last 2.
    ///
    /// More information can be found in [vuejs/vue#9468](https://github.com/vuejs/vue/issues/9468#issuecomment-462210146)
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render(h) {
    ///     var children = this.$scopedSlots.default(foo, bar)
    ///     var children = this.$scopedSlots.default(...foo)
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render(h) {
    ///     var children = this.$scopedSlots.default()
    ///     var children = this.$scopedSlots.default(foo)
    ///     var children = this.$scopedSlots.default({ foo, bar })
    ///   }
    /// }
    /// </script>
    /// ```
    NoMultipleSlotArgs,
    vue,
    restriction,
    pending  // TODO: Remove second argument, Spread argument is possible not supported
);

impl Rule for NoMultipleSlotArgs {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.arguments.is_empty() {
            return;
        }

        let member_expr = match call_expr.callee.get_inner_expression() {
            Expression::StaticMemberExpression(member_expr) => member_expr.as_ref(),
            Expression::ChainExpression(chain_expr) => {
                if let Some(MemberExpression::StaticMemberExpression(member_expr)) =
                    chain_expr.expression.as_member_expression()
                {
                    member_expr.as_ref()
                } else {
                    return;
                }
            }
            Expression::Identifier(identifier) => {
                let Some(member_expr) = get_identifier_resolved_reference(identifier, ctx) else {
                    return;
                };
                if let Expression::StaticMemberExpression(member_expr) = member_expr {
                    member_expr.as_ref()
                } else {
                    return;
                }
            }
            _ => return,
        };

        let inner = match member_expr.object.get_inner_expression() {
            Expression::StaticMemberExpression(inner) => inner.as_ref(),
            Expression::ChainExpression(chain_expr) => {
                if let Some(MemberExpression::StaticMemberExpression(inner)) =
                    chain_expr.expression.as_member_expression()
                {
                    inner.as_ref()
                } else {
                    return;
                }
            }
            _ => return,
        };

        match inner.object.get_inner_expression() {
            Expression::ThisExpression(_) => {}
            Expression::Identifier(identifier) => {
                let Some(expression) = get_identifier_resolved_reference(identifier, ctx) else {
                    return;
                };
                if !matches!(expression, Expression::ThisExpression(_)) {
                    return;
                }
            }
            _ => return,
        }

        if inner.property.name != "$slots" && inner.property.name != "$scopedSlots" {
            return;
        }

        if call_expr.arguments.len() > 1 {
            ctx.diagnostic(multiple_arguments_diagnostic(call_expr.arguments[1].span()));
        } else if call_expr.arguments[0].is_spread() {
            ctx.diagnostic(spread_argument_diagnostic(call_expr.arguments[0].span()));
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_path().extension().is_some_and(|ext| ext == "vue")
            && ctx.frameworks_options() != FrameworkOptions::VueSetup
    }
}

fn get_identifier_resolved_reference<'a>(
    identifier: &IdentifierReference,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let reference = ctx.scoping().get_reference(identifier.reference_id());
    let symbol_id = reference.symbol_id()?;
    let declaration = ctx.scoping().symbol_declaration(symbol_id);
    let node = ctx.nodes().get_node(declaration);

    let AstKind::VariableDeclarator(declarator) = node.kind() else {
        return None;
    };

    // `const` variable can not be overridden
    if declarator.kind == VariableDeclarationKind::Const {
        return declarator.init.as_ref();
    }

    find_latest_assignment(&identifier.name, declarator.span.end, identifier.span.start, ctx)
}

fn find_latest_assignment<'a>(
    identifier_name: &str,
    start_index: u32,
    end_index: u32,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let mut result = None;
    for node in ctx.nodes() {
        // The node is after the call expression, no need to continue searching
        if node.span().start > end_index {
            break;
        }

        // The node is before the variable declaration, skip it
        if node.span().start > start_index {
            if let AstKind::AssignmentExpression(assign_expr) = node.kind() {
                if let AssignmentTarget::AssignmentTargetIdentifier(assigned_id) = &assign_expr.left
                {
                    if assigned_id.name == identifier_name {
                        result = Some(&assign_expr.right);
                    }
                }
            }
        }
    }
    result
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
			        render (h) {
			          var children = this.$scopedSlots.default()
			          var children = this.$scopedSlots.foo(foo)
			          const bar = this.$scopedSlots.bar
			          bar(foo)
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
			        render (h) {
			          unknown.$scopedSlots.default(foo, bar)
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
			        render (h) {
			          // for Vue3
			          var children = this.$slots.default()
			          var children = this.$slots.foo(foo)
			          const bar = this.$slots.bar
			          bar(foo)
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
			        render (h) {
			          this.$foo.default(foo, bar)
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
			        render (h) {
			          this.$scopedSlots.default(foo, bar)
			          this.$scopedSlots.foo(foo, bar)
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
			        render (h) {
			          this?.$scopedSlots?.default?.(foo, bar)
			          this?.$scopedSlots?.foo?.(foo, bar)
			          const vm = this
			          vm?.$scopedSlots?.default?.(foo, bar)
			          vm?.$scopedSlots?.foo?.(foo, bar)
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
			        render (h) {
			          this.$scopedSlots.default?.(foo, bar)
			          this.$scopedSlots.foo?.(foo, bar)
			          const vm = this
			          vm.$scopedSlots.default?.(foo, bar)
			          vm.$scopedSlots.foo?.(foo, bar)
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
			        render (h) {
			          ;(this?.$scopedSlots)?.default?.(foo, bar)
			          ;(this?.$scopedSlots?.foo)?.(foo, bar)
			          const vm = this
			          ;(vm?.$scopedSlots)?.default?.(foo, bar)
			          ;(vm?.$scopedSlots?.foo)?.(foo, bar)
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
			        render (h) {
			          ;(this?.$scopedSlots).default(foo, bar)
			          ;(this?.$scopedSlots?.foo)(foo, bar)
			          const vm = this
			          ;(vm?.$scopedSlots).default(foo, bar)
			          ;(vm?.$scopedSlots?.foo)(foo, bar)
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
			        render (h) {
			          let children

			          this.$scopedSlots.default(foo, { bar })

			          children = this.$scopedSlots.foo
			          if (children) children(...foo)
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
			        render (h) {
			          // for Vue3
			          this.$slots.default(foo, bar)
			          this.$slots.foo(foo, bar)
			        }
			      }
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoMultipleSlotArgs::NAME, NoMultipleSlotArgs::PLUGIN, pass, fail)
        .test_and_snapshot();
}
