use oxc_ast::{ast::*, AstBuilder, AstKind};
use oxc_semantic::AstNodeId;
use oxc_span::SPAN;
use std::rc::Rc;

use oxc_syntax::operator::BinaryOperator;

use crate::context::TransformerCtx;
use crate::options::TransformOptions;
use crate::TransformTarget;

/// ES2015: instanceof
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-instanceof>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-instanceof>
pub struct Instanceof<'a> {
    ast: Rc<AstBuilder<'a>>,
    ctx: TransformerCtx<'a>,
}

impl<'a> Instanceof<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.instanceof).then(|| Self { ast, ctx })
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        // if instanceof syntax is under a helper, it should not be transformed
        if let Expression::BinaryExpression(be) = expr {
            if let BinaryExpression { operator: BinaryOperator::Instanceof, left, right, .. } =
                &**be
            {
                let is_under_helper = self
                    .ctx
                    .semantic()
                    .nodes()
                    // FIXME: how to get the AstNodeId of the expression?
                    .iter_parents(AstNodeId::new(1)) // expr.get_id or sth
                    .any(|parent| {
                        if let AstKind::CallExpression(CallExpression { callee, .. }) =
                            parent.kind()
                        {
                            return callee.is_specific_member_access("babelHelpers", "instanceof");
                        };
                        false
                    });

                if is_under_helper {
                    return;
                }

                let object = self.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    "babelHelpers".into(),
                ));

                let property = IdentifierName::new(SPAN, "instanceof".into());
                let helper = self.ast.member_expression(MemberExpression::StaticMemberExpression(
                    StaticMemberExpression { span: SPAN, object, property, optional: false },
                ));

                let left = self.ast.copy(left);
                let right = self.ast.copy(right);
                let mut args = self.ast.new_vec_with_capacity(2);
                args.push(Argument::Expression(left));
                args.push(Argument::Expression(right));

                *expr = self.ast.call_expression(SPAN, helper, args, false, None);
            }
        }
    }
}
