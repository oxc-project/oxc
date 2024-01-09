use oxc_ast::{ast::*, AstBuilder, VisitMut, AstKind};
use oxc_diagnostics::miette;
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::operator::BinaryOperator;
use std::{rc::Rc};
use crate::{TransformOptions, TransformTarget, context::TransformerCtx};
use oxc_allocator::Vec;

/// ES2015: New Target
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-template-new-target>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-new-target>
pub struct NewTarget<'a> {
    ast: Rc<AstBuilder<'a>>,
    ctx: TransformerCtx<'a>,
    kinds: Vec<'a, NewTargetKind>,
}
enum NewTargetKind {
    Method,
    Constructor,
    Function(Atom)
}

impl<'a> VisitMut<'a> for NewTarget<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        if let Some(kind) = match kind {
            AstKind::MethodDefinition(def) => {
                match def.kind {
                    MethodDefinitionKind::Get | MethodDefinitionKind::Set |MethodDefinitionKind::Method => Some(NewTargetKind::Method),
                    MethodDefinitionKind::Constructor => Some( NewTargetKind::Constructor),
                }
            }
            AstKind::Function(function) => {
                Some(function.id.as_ref().map(|id| NewTargetKind::Function(id.name.clone())).unwrap_or_else(|| NewTargetKind::Function(self.ctx.scopes().generate_uid("target"))))
            }
            _ => None
        } {
            self.kinds.push(kind);
        }
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.kinds.pop();
    }
}

impl<'a> NewTarget<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, ctx:  TransformerCtx<'a>, options: &TransformOptions) -> Option<Self> {
        let kinds = ast.new_vec();
        (options.target < TransformTarget::ES2015 || options.new_target)
            .then(|| Self { ast, ctx, kinds })
    }

    fn create_constructor_expr(&self, span: Span) -> Expression<'a> {
        self.ast.static_member_expression(span, self.ast.this_expression(span), IdentifierName { span: span, name: "constructor".into() }, false)
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        if let Expression::MetaProperty(meta) = expr {
            if meta.meta.name == "new" && meta.property.name == "target" {
                if let Some(kind) = self.kinds.last() {
                    match kind {
                        NewTargetKind::Constructor => {
                            *expr = self.create_constructor_expr(meta.span);
                        }
                        NewTargetKind::Method => {
                            *expr = self.ast.void_0();
                        }
                        NewTargetKind::Function(name) => {
                            let test = self.ast.binary_expression(SPAN, self.ast.this_expression(SPAN), BinaryOperator::Instanceof, self.ast.identifier_reference_expression( IdentifierReference::new(SPAN, name.clone())));
                            let consequent = self.ast.static_member_expression(SPAN, self.ast.this_expression(SPAN), IdentifierName { span: SPAN, name: "constructor".into() }, false);
                            *expr = self.ast.conditional_expression(meta.span, test, consequent, self.ast.void_0());
                        }
                    } 
                } else {
                    self.ctx.error(miette::Error::msg("new.target must be under a (non-arrow) function or a class."))
                }
            }
        }
    }
}

