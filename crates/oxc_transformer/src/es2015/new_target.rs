use crate::{context::TransformerCtx, TransformOptions, TransformTarget};
use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder, AstKind, VisitMut};
use oxc_diagnostics::miette;
use oxc_span::{Atom, Span, SPAN};
use oxc_syntax::operator::BinaryOperator;
use std::rc::Rc;

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

#[derive(Debug)]
enum NewTargetKind {
    Method,
    Constructor,
    Function(Option<Atom>),
}

impl<'a> VisitMut<'a> for NewTarget<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        if let Some(kind) = self.get_kind(kind) {
            self.kinds.push(kind);
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if self.get_kind(kind).is_some() {
            self.kinds.pop();
        }
    }
}

impl<'a> NewTarget<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        let kinds = ast.new_vec();
        (options.target < TransformTarget::ES2015 || options.new_target).then(|| Self {
            ast,
            ctx,
            kinds,
        })
    }

    fn get_kind(&self, kind: AstKind<'a>) -> Option<NewTargetKind> {
        match kind {
            AstKind::MethodDefinition(def) => match def.kind {
                MethodDefinitionKind::Get
                | MethodDefinitionKind::Set
                | MethodDefinitionKind::Method => Some(NewTargetKind::Method),
                MethodDefinitionKind::Constructor => Some(NewTargetKind::Constructor),
            },
            AstKind::ObjectProperty(property) => property.method.then_some(NewTargetKind::Method),
            AstKind::Function(function) => {
                // oxc visitor `MethodDefinitionKind` will enter `Function` node, here need to exclude it
                if let Some(kind) = self.kinds.last() {
                    if !matches!(kind, NewTargetKind::Function(_)) {
                        return None;
                    }
                }
                function.id.as_ref().map(|id| NewTargetKind::Function(Some(id.name.clone())))
            }
            _ => None,
        }
    }

    fn create_constructor_expr(&self, span: Span) -> Expression<'a> {
        self.ast.static_member_expression(
            span,
            self.ast.this_expression(span),
            IdentifierName { span, name: "constructor".into() },
            false,
        )
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
                            // TODO packages/babel-helper-create-class-features-plugin/src/fields.ts#L192 unshadow
                            // It will mutate previous ast node, it is difficult at now.
                            let id = name
                                .clone()
                                .unwrap_or_else(|| self.ctx.scopes().generate_uid("target"));
                            let test = self.ast.binary_expression(
                                SPAN,
                                self.ast.this_expression(SPAN),
                                BinaryOperator::Instanceof,
                                self.ast.identifier_reference_expression(IdentifierReference::new(
                                    SPAN, id,
                                )),
                            );
                            let consequent = self.ast.static_member_expression(
                                SPAN,
                                self.ast.this_expression(SPAN),
                                IdentifierName { span: SPAN, name: "constructor".into() },
                                false,
                            );
                            *expr = self.ast.conditional_expression(
                                meta.span,
                                test,
                                consequent,
                                self.ast.void_0(),
                            );
                        }
                    }
                } else {
                    self.ctx.error(miette::Error::msg(
                        "new.target must be under a (non-arrow) function or a class.",
                    ));
                }
            }
        }
    }
}
