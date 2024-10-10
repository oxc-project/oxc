//! Utility import / inline / external helper function transform.
//!
//! TODO: add more details
pub mod options;

use std::{borrow::Cow, cell::RefCell, rc::Rc};

use options::{HelperLoaderMode, HelperLoaderOptions};
use oxc_allocator::Vec;
use oxc_ast::ast::{Argument, CallExpression, Expression, Program, TSTypeParameterInstantiation};
use oxc_semantic::{ReferenceFlags, SymbolFlags, SymbolId};
use oxc_span::{Atom, SPAN};
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};
use rustc_hash::FxHashMap;

use super::module_imports::ImportKind;
use crate::TransformCtx;

pub struct HelperLoader<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> Traverse<'a> for HelperLoader<'a, 'ctx> {
    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.insert_into_program();
    }
}

impl<'a, 'ctx> HelperLoader<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }

    fn insert_into_program(&self) {
        self.ctx
            .helper_loader
            .imports
            .borrow_mut()
            .drain(..)
            .for_each(|(source, import)| self.ctx.module_imports.add_import(source, import, false));
    }
}

type LoadedHelper<'a> = FxHashMap<Atom<'a>, (Atom<'a>, Option<SymbolId>)>;
pub struct HelperLoaderStore<'a> {
    mode: HelperLoaderMode,
    module_name: Cow<'static, str>,
    /// (helper_name, (import_name, symbol_id))
    loaded_helper: Rc<RefCell<LoadedHelper<'a>>>,
    imports: Rc<RefCell<std::vec::Vec<(Atom<'a>, ImportKind<'a>)>>>,
}

impl<'a> HelperLoaderStore<'a> {
    pub fn new(options: &HelperLoaderOptions) -> Self {
        Self {
            mode: options.mode,
            module_name: options.module_name.clone(),
            loaded_helper: Rc::new(RefCell::new(FxHashMap::default())),
            imports: Rc::new(RefCell::new(std::vec::Vec::new())),
        }
    }

    fn add_default_import(
        &self,
        helper_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> BoundIdentifier<'a> {
        let source = ctx.ast.atom(&format!("{}/helpers/{helper_name}", self.module_name));
        let bound_ident = ctx.generate_uid_in_root_scope(helper_name, SymbolFlags::Import);
        self.imports.borrow_mut().push((
            source,
            ImportKind::new_default(bound_ident.name.clone(), bound_ident.symbol_id),
        ));
        bound_ident
    }

    fn transform_for_runtime_helper(
        &self,
        helper_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if !self.loaded_helper.borrow().contains_key(helper_name) {
            let bound_ident = self.add_default_import(helper_name, ctx);
            self.loaded_helper
                .borrow_mut()
                .insert(ctx.ast.atom(helper_name), (bound_ident.name, Some(bound_ident.symbol_id)));
        }
        let (name, symbol_id) = self.loaded_helper.borrow_mut()[helper_name].clone();
        let ident = ctx.create_reference_id(SPAN, name, symbol_id, ReferenceFlags::Read);
        ctx.ast.expression_from_identifier_reference(ident)
    }

    fn transform_for_external_helper(
        &self,
        helper_name: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if !self.loaded_helper.borrow().contains_key(&helper_name) {
            let symbol_id = ctx.scopes().find_binding(ctx.current_scope_id(), "babelHelpers");
            let name = ctx.ast.atom("babelHelpers");

            self.loaded_helper.borrow_mut().insert(helper_name.clone(), (name, symbol_id));
        }

        let (name, symbol_id) = self.loaded_helper.borrow_mut()[&helper_name].clone();
        let ident = ctx.create_reference_id(SPAN, name, symbol_id, ReferenceFlags::Read);
        let object = ctx.ast.expression_from_identifier_reference(ident);
        let property = ctx.ast.identifier_name(SPAN, helper_name);
        Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
    }

    #[allow(dead_code)]
    pub fn call(
        &mut self,
        helper_name: Atom<'a>,
        arguments: Vec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> CallExpression<'a> {
        let callee = self.get_callee(helper_name, ctx);
        ctx.ast.call_expression(
            SPAN,
            callee,
            None::<TSTypeParameterInstantiation<'a>>,
            arguments,
            false,
        )
    }

    #[allow(dead_code)]
    pub fn call_expr(
        &mut self,
        helper_name: Atom<'a>,
        arguments: Vec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let callee = self.get_callee(helper_name, ctx);
        ctx.ast.expression_call(
            SPAN,
            callee,
            None::<TSTypeParameterInstantiation<'a>>,
            arguments,
            false,
        )
    }

    pub fn get_callee(&self, helper_name: Atom<'a>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        match self.mode {
            HelperLoaderMode::Runtime => self.transform_for_runtime_helper(&helper_name, ctx),
            HelperLoaderMode::External => self.transform_for_external_helper(helper_name, ctx),
            HelperLoaderMode::Inline => {
                panic!("Inline helpers are not supported yet");
            }
        }
    }
}
