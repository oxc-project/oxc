//! Integration of standard decorators with class bindings and private-element lowering.

use oxc_ast::ast::*;
use oxc_str::Ident;
use oxc_traverse::BoundIdentifier;

use crate::{
    context::TraverseCtx,
    decorator::standard::{ClassDecoratorBindings, DecoratorElementKind, PrivateDecoratorAccess},
};

use super::ClassProperties;

impl<'a> ClassProperties<'a> {
    pub(super) fn push_decorate_element(
        &mut self,
        kind: DecoratorElementKind,
        is_static: bool,
        key: Expression<'a>,
        decorators: &BoundIdentifier<'a>,
        private_name: Option<Ident<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let details = self.classes_stack.last();
        let private = private_name.map(|name| {
            let prop = details.private_props.lowered().unwrap().get(&name).unwrap();
            let brand = if kind == DecoratorElementKind::Field {
                prop.binding.clone()
            } else if is_static {
                details.bindings.decorator_static_brand.as_ref().unwrap().clone()
            } else {
                details.bindings.brand().clone()
            };
            let (getter, setter) = if matches!(prop.method_kind, Some(MethodDefinitionKind::Set)) {
                (prop.binding2.clone(), Some(prop.binding.clone()))
            } else {
                (Some(prop.binding.clone()), prop.binding2.clone())
            };
            PrivateDecoratorAccess { brand, value: prop.binding.clone(), getter, setter }
        });
        let target = if details.is_declaration {
            details.bindings.name.as_ref().or(details.bindings.temp.as_ref())
        } else {
            details.bindings.temp.as_ref()
        }
        .expect("decorated class always has a binding");
        self.decorator_state.push_decorate_element(
            kind,
            is_static,
            key,
            decorators,
            private.as_ref(),
            details.decorators.as_ref().unwrap(),
            target,
            ctx,
        );
    }

    pub(super) fn build_decorator_sequence(
        &mut self,
        class: &Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> (Vec<Expression<'a>>, Option<Expression<'a>>) {
        let details = self.classes_stack.last();
        let bindings = &details.bindings;
        let target = if details.is_declaration {
            bindings.name.as_ref().or(bindings.temp.as_ref())
        } else {
            bindings.temp.as_ref()
        }
        .expect("decorated class always has a binding");
        self.decorator_state.build_decorator_sequence(
            class,
            details.decorators.as_ref().unwrap(),
            &ClassDecoratorBindings {
                target,
                declaration: if details.is_declaration { bindings.name.as_ref() } else { None },
                temporary: bindings.temp.as_ref(),
                static_brand: bindings.decorator_static_brand.as_ref(),
            },
            ctx,
        )
    }
}
