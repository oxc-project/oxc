use crate::context::Ctx;
use crate::modules::options::{format::ModulesFormat, ModulesOptions};
use oxc_ast::ast::{BindingIdentifier, IdentifierName, IdentifierReference, Program};
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Traverse, TraverseCtx};
use std::cell::Cell;

pub mod options;

pub struct Modules<'a> {
    ctx: Ctx<'a>,
    options: ModulesOptions,

    /// Because the `exports` is the keyword ONLY when the format is `Commonjs`, `let exports = {}` is valid in ES6.
    /// But if we want to transform the code to CommonJS, we need to rename the `exports` to `_exports`, or other deconflicted names.
    rename_exports_to: String,
}

impl<'a> Modules<'a> {
    pub fn new(options: ModulesOptions, ctx: Ctx<'a>) -> Self {
        Self { ctx, options, rename_exports_to: "_exports".to_string() }
    }
}

impl<'a> Traverse<'a> for Modules<'a> {
    fn exit_identifier_name(&mut self, ident: &mut IdentifierName<'a>, _ctx: &mut TraverseCtx<'a>) {
        // deconflict exports
        if ident.name.as_str() == "exports"
            && matches!(self.options.format, ModulesFormat::Commonjs)
        {
            ident.name = self.ctx.ast.atom(self.rename_exports_to.as_str());
        }
    }

    fn exit_identifier_reference(
        &mut self,
        reference: &mut IdentifierReference<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if reference.name.as_str() == "exports"
            && matches!(self.options.format, ModulesFormat::Commonjs)
        {
            reference.name = ctx.ast.atom(self.rename_exports_to.as_str());
        }
    }

    fn enter_binding_identifier(
        &mut self,
        ident: &mut BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if ident.name.as_str() == "exports"
            && matches!(self.options.format, ModulesFormat::Commonjs)
        {
            let symbol_id = ident.symbol_id.get();
            if let Some(symbol) = symbol_id {
                let symbols = ctx.symbols_mut();
                let names = &symbols.names.raw;
                // 1. Check if `_exports` is available.
                // 2. Check if `_exports{i}` is available (i = 1, 2, 3, ...).
                // 3. If not, create a new symbol, and save to `self.rename_exports_to`.
                let target_name = if names.iter().any(|name| name == "_exports") {
                    let mut i = 1;
                    let mut dest = format!("_exports{i}");
                    while names.iter().any(|name| name == &dest) {
                        i += 1;
                        dest = format!("_exports{i}");
                    }
                    dest
                } else {
                    "_exports".to_string()
                };
                symbols.set_name(symbol, target_name.as_str().into());
                ident.name = ctx.ast.atom(target_name.as_str());
                self.rename_exports_to = target_name;
                // Find out if there's `_exports`, `_exports1`, etc.
            }
        }
    }
}
