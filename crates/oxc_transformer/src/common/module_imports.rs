//! Utility transform to add `import` / `require` statements to top of program.
//!
//! `ModuleImportsStore` contains an `IndexMap<Str<'a>, Vec<Import<'a>>>`.
//! It is stored on `TransformState`.
//!
//! Other transforms can add `import`s / `require`s to the store by calling methods of `ModuleImportsStore`:
//!
//! ### Usage
//!
//! ```rs
//! // import { jsx as _jsx } from 'react';
//! ctx.state.module_imports.add_named_import(
//!     Str::from("react"),
//!     Str::from("jsx"),
//!     BoundIdentifier::new(Ident::new_const("_jsx"), symbol_id),
//!     false,
//! );
//!
//! // ESM: import React from 'react';
//! // CJS: var _React = require('react');
//! ctx.state.module_imports.add_default_import(
//!     Str::from("react"),
//!     BoundIdentifier::new(Ident::new_const("React"), symbol_id),
//!     false,
//! );
//! ```
//!
//! > NOTE: Using `import` or `require` is determined by `TransformState::source_type`.
//!
//! Based on `@babel/helper-module-imports`
//! <https://github.com/nicolo-ribaudo/babel/tree/v7.25.8/packages/babel-helper-module-imports>

use indexmap::{IndexMap, map::Entry as IndexMapEntry};

use oxc_allocator::ArenaVec;
use oxc_ast::{NONE, ast::*};
use oxc_semantic::ReferenceFlags;
use oxc_span::SPAN;
use oxc_str::{Str, static_ident};
use oxc_syntax::symbol::SymbolId;
use oxc_traverse::BoundIdentifier;

use crate::context::TraverseCtx;

pub struct NamedImport<'a> {
    imported: Str<'a>,
    local: BoundIdentifier<'a>,
}

pub enum Import<'a> {
    Named(NamedImport<'a>),
    Default(BoundIdentifier<'a>),
}

/// Store for `import` / `require` statements to be added at top of program.
///
/// TODO(improve-on-babel): Insertion order does not matter. We only have to use `IndexMap`
/// to produce output that's the same as Babel's.
/// Substitute `FxHashMap` once we don't need to match Babel's output exactly.
pub struct ModuleImportsStore<'a> {
    pub(crate) imports: IndexMap<Str<'a>, Vec<Import<'a>>>,
}

// Public methods
impl<'a> ModuleImportsStore<'a> {
    /// Create new `ModuleImportsStore`.
    pub fn new() -> Self {
        Self { imports: IndexMap::default() }
    }

    /// Add default `import` or `require` to top of program.
    ///
    /// Which it will be depends on the source type.
    ///
    /// * `import named_import from 'source';` or
    /// * `var named_import = require('source');`
    ///
    /// If `front` is `true`, `import`/`require` is added to front of the `import`s/`require`s.
    pub fn add_default_import(&mut self, source: Str<'a>, local: BoundIdentifier<'a>, front: bool) {
        self.add_import(source, Import::Default(local), front);
    }

    /// Add named `import` to top of program.
    ///
    /// `import { named_import } from 'source';`
    ///
    /// If `front` is `true`, `import` is added to front of the `import`s.
    ///
    /// Adding named `require`s is not supported, and will cause a panic later on.
    pub fn add_named_import(
        &mut self,
        source: Str<'a>,
        imported: Str<'a>,
        local: BoundIdentifier<'a>,
        front: bool,
    ) {
        self.add_import(source, Import::Named(NamedImport { imported, local }), front);
    }

    /// Returns `true` if no imports have been scheduled for insertion.
    pub fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }
}

// Internal methods
impl<'a> ModuleImportsStore<'a> {
    /// Add `import` or `require` to top of program.
    ///
    /// Which it will be depends on the source type.
    ///
    /// * `import { named_import } from 'source';` or
    /// * `var named_import = require('source');`
    ///
    /// Adding a named `require` is not supported, and will cause a panic later on.
    ///
    /// If `front` is `true`, `import`/`require` is added to front of the `import`s/`require`s.
    /// TODO(improve-on-babel): `front` option is only required to pass one of Babel's tests. Output
    /// without it is still valid. Remove this once our output doesn't need to match Babel exactly.
    fn add_import(&mut self, source: Str<'a>, import: Import<'a>, front: bool) {
        match self.imports.entry(source) {
            IndexMapEntry::Occupied(mut entry) => {
                entry.get_mut().push(import);
                if front && entry.index() != 0 {
                    entry.move_index(0);
                }
            }
            IndexMapEntry::Vacant(entry) => {
                let named_imports = vec![import];
                if front {
                    entry.shift_insert(0, named_imports);
                } else {
                    entry.insert(named_imports);
                }
            }
        }
    }

    pub(crate) fn get_import(
        source: Str<'a>,
        names: Vec<Import<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Statement<'a> {
        let specifiers = ArenaVec::from_iter_in(
            names.into_iter().map(|import| match import {
                Import::Named(import) => ImportDeclarationSpecifier::new_import_specifier(
                    SPAN,
                    ModuleExportName::new_identifier_name(SPAN, import.imported, ctx),
                    import.local.create_binding_identifier(ctx),
                    ImportOrExportKind::Value,
                    ctx,
                ),
                Import::Default(local) => ImportDeclarationSpecifier::new_import_default_specifier(
                    SPAN,
                    local.create_binding_identifier(ctx),
                    ctx,
                ),
            }),
            ctx,
        );

        Statement::new_import_declaration(
            SPAN,
            Some(specifiers),
            StringLiteral::new(SPAN, source, None, ctx),
            None,
            NONE,
            ImportOrExportKind::Value,
            ctx,
        )
    }

    pub(crate) fn get_require(
        source: Str<'a>,
        names: Vec<Import<'a>>,
        require_symbol_id: Option<SymbolId>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let callee = ctx.create_ident_expr(
            SPAN,
            static_ident!("require"),
            require_symbol_id,
            ReferenceFlags::read(),
        );

        let args = {
            let arg = Argument::new_string_literal(SPAN, source, None, ctx);
            ArenaVec::from_value_in(arg, ctx)
        };
        let Some(Import::Default(local)) = names.into_iter().next() else { unreachable!() };
        let id = local.create_binding_pattern(ctx);
        let var_kind = VariableDeclarationKind::Var;
        let decl = {
            let init = Expression::new_call_expression(SPAN, callee, NONE, args, false, ctx);
            let decl = VariableDeclarator::new(SPAN, var_kind, id, NONE, Some(init), false, ctx);
            ArenaVec::from_value_in(decl, ctx)
        };
        Statement::new_variable_declaration(SPAN, var_kind, decl, false, ctx)
    }
}
