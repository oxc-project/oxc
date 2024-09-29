pub mod options;

use std::{cell::RefCell, rc::Rc};

use options::{HelperLoaderMode, HelperLoaderOptions};
use oxc_allocator::{Allocator, Vec};
use oxc_ast::{
    ast::{Argument, CallExpression, Expression, TSTypeParameterInstantiation},
    AstBuilder,
};
use oxc_semantic::{ReferenceFlags, SymbolFlags, SymbolId};
use oxc_span::{Atom, SPAN};
use oxc_traverse::TraverseCtx;
use rustc_hash::FxHashMap;

use super::module_imports::{ImportSpecifier, ModuleImports};

pub struct HelperLoader<'a> {
    ast: AstBuilder<'a>,
    mode: HelperLoaderMode,
    module_name: Atom<'a>,
    /// (helper_name, (import_name, symbol_id))
    loaded_helper: FxHashMap<Atom<'a>, (Atom<'a>, Option<SymbolId>)>,
    module_imports: Rc<RefCell<ModuleImports<'a>>>,
}

impl<'a> HelperLoader<'a> {
    pub fn new(
        options: &HelperLoaderOptions,
        allocator: &'a Allocator,
        module_imports: &Rc<RefCell<ModuleImports<'a>>>,
    ) -> Self {
        let ast = AstBuilder::new(allocator);
        let module_name = ast.atom(&options.module_name);
        Self {
            ast,
            mode: options.mode,
            module_name,
            loaded_helper: FxHashMap::default(),
            module_imports: Rc::clone(module_imports),
        }
    }

    fn add_default_import(&self, helper_name: &str, ctx: &mut TraverseCtx<'a>) -> SymbolId {
        let source = ctx.ast.atom(&format!("{}/helpers/{helper_name}", self.module_name));
        let symbol_id = ctx.generate_uid_in_root_scope(helper_name, SymbolFlags::Import);
        let name = self.ast.atom(&ctx.symbols().names[symbol_id]);
        self.module_imports
            .borrow_mut()
            .add_default(source, ImportSpecifier::new(name, None, symbol_id));
        symbol_id
    }

    fn transform_for_runtime_helper(
        &mut self,
        helper_name: &str,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if !self.loaded_helper.contains_key(helper_name) {
            let symbol_id = self.add_default_import(helper_name, ctx);
            let name = self.ast.atom(&ctx.symbols().names[symbol_id]);
            self.loaded_helper.insert(ctx.ast.atom(helper_name), (name, Some(symbol_id)));
        }

        let (name, symbol_id) = self.loaded_helper[helper_name].clone();
        let ident = ctx.create_reference_id(SPAN, name, symbol_id, ReferenceFlags::Read);
        ctx.ast.expression_from_identifier_reference(ident)
    }

    fn transform_for_external_helper(
        &mut self,
        helper_name: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        // TODO: Whether to use find_binding instead of get_root_binding
        if !self.loaded_helper.contains_key(&helper_name) {
            let symbol_id = ctx.scopes().get_root_binding("babelHelpers");
            let name = self.ast.atom("babelHelpers");
            self.loaded_helper.insert(helper_name.clone(), (name, symbol_id));
        }

        let (name, symbol_id) = self.loaded_helper[&helper_name].clone();
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

    pub fn get_callee(
        &mut self,
        helper_name: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match self.mode {
            HelperLoaderMode::Runtime => self.transform_for_runtime_helper(&helper_name, ctx),
            HelperLoaderMode::External => self.transform_for_external_helper(helper_name, ctx),
            HelperLoaderMode::Inline => {
                panic!("Inline helpers are not supported yet");
            }
        }
    }
}
