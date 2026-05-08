use oxc_ast::{
    AstKind,
    ast::{ChainElement, Expression, IdentifierReference, StaticMemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::get_declaration_from_reference_id, context::LintContext, rule::Rule,
};

fn require_slots_as_functions_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Property in `$slots` should be used as function.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireSlotsAsFunctions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce properties of `$slots` to be used as a function.
    ///
    /// ### Why is this bad?
    ///
    /// In Vue.js 3, `this.$slots.<name>` is a function (slot render function),
    /// not an array of vnodes like in Vue.js 2. Treating slot properties as
    /// values (e.g. `this.$slots.default.filter(...)`) breaks at runtime.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   render(h) {
    ///     var children = this.$slots.default
    ///     return h('div', children.filter(...))
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
    ///     var children = this.$slots.default()
    ///     return h('div', children)
    ///   }
    /// }
    /// </script>
    /// ```
    RequireSlotsAsFunctions,
    vue,
    correctness,
    version = "next",
);

impl Rule for RequireSlotsAsFunctions {
    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::StaticMemberExpression(slot_prop_member) = node.kind() else { return };

        // Inner must be `this.$slots` (`StaticMemberExpression` with property `$slots`).
        let Some(slots_static) = inner_static_member_expression(&slot_prop_member.object) else {
            return;
        };
        if slots_static.property.name != "$slots" {
            return;
        }
        if !is_this_object(&slots_static.object, ctx) {
            return;
        }

        verify(node, slot_prop_member.property.span, ctx);
    }
}

fn inner_static_member_expression<'a, 'b>(
    expr: &'b Expression<'a>,
) -> Option<&'b StaticMemberExpression<'a>> {
    match expr.get_inner_expression() {
        Expression::StaticMemberExpression(m) => Some(m),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(m) => Some(m),
            _ => None,
        },
        _ => None,
    }
}

fn is_this_object(expr: &Expression, ctx: &LintContext<'_>) -> bool {
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

fn verify(node: &AstNode<'_>, report_span: Span, ctx: &LintContext<'_>) {
    let parent = ctx.nodes().parent_node(node.id());
    match parent.kind() {
        // `this.$slots.foo()` — used as a function, OK
        AstKind::CallExpression(call) if call.callee.span() == node.kind().span() => {}
        // `(this.$slots.foo)?.bar` etc. — keep walking up
        AstKind::ChainExpression(_)
        | AstKind::ParenthesizedExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSNonNullExpression(_)
        | AstKind::TSSatisfiesExpression(_) => {
            verify(parent, report_span, ctx);
        }
        // `var children = this.$slots.foo` — follow references of `children`
        AstKind::VariableDeclarator(var)
            if var.init.as_ref().is_some_and(|init| init.span() == node.kind().span()) =>
        {
            if let Some(binding_ident) = var.id.get_binding_identifier()
                && let Some(symbol_id) = binding_ident.symbol_id.get()
            {
                follow_references(symbol_id, report_span, ctx);
            }
        }
        // `children = this.$slots.foo` — follow references of `children`
        AstKind::AssignmentExpression(assign) if assign.right.span() == node.kind().span() => {
            if let oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(target) =
                &assign.left
                && let Some(symbol_id) =
                    ctx.scoping().get_reference(target.reference_id()).symbol_id()
            {
                follow_references(symbol_id, report_span, ctx);
            }
        }
        // `.bar` access / spread / array element / non-init declarator → NG
        AstKind::StaticMemberExpression(_)
        | AstKind::ComputedMemberExpression(_)
        | AstKind::PrivateFieldExpression(_)
        | AstKind::SpreadElement(_)
        | AstKind::ArrayExpression(_)
        | AstKind::VariableDeclarator(_) => {
            ctx.diagnostic(require_slots_as_functions_diagnostic(report_span));
        }
        _ => {}
    }
}

fn follow_references(
    symbol_id: oxc_semantic::SymbolId,
    report_span: Span,
    ctx: &LintContext<'_>,
) {
    for reference in ctx.scoping().get_resolved_references(symbol_id) {
        if !reference.flags().is_read() {
            continue;
        }
        let ref_node = ctx.nodes().get_node(reference.node_id());
        verify(ref_node, report_span, ctx);
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
                    render (h) {
                      var children = this.$slots.default()
                      var children = this.$slots.default && this.$slots.default()
            
                      return h('div', this.$slots.default)
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
                      var children = unknown.$slots.default
                      var children = unknown.$slots.default.filter(test)
            
                      return h('div', [...children])
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
                      var children = this.$slots.default
                      var children = this.$slots.default.filter(test)
            
                      return h('div', [...children])
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
                      var bar = this.$slots.foo?.bar // NG
                      var bar = this.$slots.foo?.() // OK
                      var bar = (this.$slots?.foo)?.bar // NG
                      var bar = (this.$slots?.foo)?.() // OK
                      var bar = (this?.$slots)?.foo?.bar // NG
                      var bar = (this?.$slots)?.foo?.() // OK
                      return h('div', bar)
                    }
                  }
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(RequireSlotsAsFunctions::NAME, RequireSlotsAsFunctions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
