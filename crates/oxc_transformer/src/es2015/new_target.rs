use crate::{context::TransformerCtx, TransformOptions, TransformTarget};
use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder};
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
    kinds: Vec<'a, NewTargetKind<'a>>,
}

#[derive(Debug)]
enum NewTargetKind<'a> {
    Method,
    Constructor,
    Function(Option<Atom<'a>>),
}

impl<'a> NewTarget<'a> {
    pub(crate) fn enter_method_definition(&mut self, def: &MethodDefinition<'a>) {
        let kind = match def.kind {
            MethodDefinitionKind::Get
            | MethodDefinitionKind::Set
            | MethodDefinitionKind::Method => NewTargetKind::Method,
            MethodDefinitionKind::Constructor => NewTargetKind::Constructor,
        };
        self.push(kind);
    }

    pub(crate) fn leave_method_definition(&mut self, _: &MethodDefinition) {
        self.pop();
    }

    pub(crate) fn enter_object_property(&mut self, prop: &ObjectProperty<'a>) {
        if prop.method {
            self.push(NewTargetKind::Method);
        }
    }

    pub(crate) fn leave_object_property(&mut self, prop: &ObjectProperty<'a>) {
        if prop.method {
            self.pop();
        }
    }

    pub(crate) fn enter_function(&mut self, func: &Function<'a>) {
        if let Some(kind) = self.function_new_target_kind(func) {
            self.push(kind);
        }
    }

    pub(crate) fn leave_function(&mut self, func: &Function<'a>) {
        if let Some(_) = self.function_new_target_kind(func) {
            self.pop();
        }
    }

    fn function_new_target_kind(&self, func: &Function<'a>) -> Option<NewTargetKind<'a>> {
        // oxc visitor `MethodDefinitionKind` will enter `Function` node, here need to exclude it
        if let Some(kind) = self.kinds.last() {
            if !matches!(kind, NewTargetKind::Function(_)) {
                return None;
            }
        }
        func.id.as_ref().map(|id| NewTargetKind::Function(Some(id.name.clone())))
    }
}

impl<'a> NewTarget<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        let kinds = ast.new_vec();
        (options.target < TransformTarget::ES2015 || options.new_target).then_some(Self {
            ast,
            ctx,
            kinds,
        })
    }

    fn push(&mut self, kind: NewTargetKind<'a>) {
        self.kinds.push(kind);
    }

    fn pop(&mut self) {
        self.kinds.pop();
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
                            let id = name.clone().unwrap_or_else(|| {
                                self.ast.new_atom(self.ctx.scopes().generate_uid("target").as_str())
                            });
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
