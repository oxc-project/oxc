//! Utility transform to add `import` / `require` statements to top of program.
//!
//! `ModuleImportsStore` contains an `IndexMap<ImportType<'a>, Vec<NamedImport<'a>>>`.
//! It is stored on `TransformCtx`.
//!
//! `ModuleImports` transform
//!
//! Other transforms can add `import`s / `require`s to the store by calling methods of `ModuleImportsStore`:
//!
//! ```rs
//! // import { jsx as _jsx } from 'react';
//! self.ctx.module_imports.add_import(
//!     Atom::from("react"),
//!     NamedImport::new(Atom::from("jsx"), Some(Atom::from("_jsx")), symbol_id)
//! );
//!
//! // var _react = require('react');
//! self.ctx.module_imports.add_require(
//!     Atom::from("react"),
//!     NamedImport::new(Atom::from("_react"), None, symbol_id)
//! );
//! ```
//!
//! Based on `@babel/helper-module-imports`
//! <https://github.com/nicolo-ribaudo/babel/tree/main/packages/babel-helper-module-imports>

use std::cell::RefCell;

use indexmap::{map::Entry as IndexMapEntry, IndexMap};

use oxc_ast::{ast::*, NONE};
use oxc_semantic::ReferenceFlags;
use oxc_span::{Atom, SPAN};
use oxc_syntax::symbol::SymbolId;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

pub struct ModuleImports<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ModuleImports<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for ModuleImports<'a, 'ctx> {
    fn exit_program(&mut self, _program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.ctx.module_imports.insert_into_program(self.ctx, ctx);
    }
}

pub struct NamedImport<'a> {
    imported: Atom<'a>,
    local: Option<Atom<'a>>, // Not used in `require`
    symbol_id: SymbolId,
}

impl<'a> NamedImport<'a> {
    pub fn new(imported: Atom<'a>, local: Option<Atom<'a>>, symbol_id: SymbolId) -> Self {
        Self { imported, local, symbol_id }
    }
}

/// Store for `import` / `require` statements to be added at top of program.
///
/// TODO(improve-on-babel): Insertion order does not matter. We only have to use `IndexMap`
/// to produce output that's the same as Babel's.
/// Substitute `FxHashMap` once we don't need to match Babel's output exactly.
pub struct ModuleImportsStore<'a> {
    imports: RefCell<IndexMap<Atom<'a>, Vec<NamedImport<'a>>>>,
}

// Public methods
impl<'a> ModuleImportsStore<'a> {
    /// Create new `ModuleImportsStore`.
    pub fn new() -> Self {
        Self { imports: RefCell::new(IndexMap::default()) }
    }

    /// Add `import` or `require` to top of program.
    ///
    /// Which it will be depends on the source type.
    ///
    /// * `import { named_import } from 'source';` or
    /// * `var named_import = require('source');`
    ///
    /// If `front` is `true`, `import`/`require` is added to front of the `import`s/`require`s.
    /// TODO(improve-on-babel): `front` option is only required to pass one of Babel's tests. Output
    /// without it is still valid. Remove this once our output doesn't need to match Babel exactly.
    pub fn add_import(&self, source: Atom<'a>, import: NamedImport<'a>, front: bool) {
        match self.imports.borrow_mut().entry(source) {
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

    /// Returns `true` if no imports have been scheduled for insertion.
    pub fn is_empty(&self) -> bool {
        self.imports.borrow().is_empty()
    }
}

// Internal methods
impl<'a> ModuleImportsStore<'a> {
    /// Insert `import` / `require` statements at top of program.
    fn insert_into_program(&self, transform_ctx: &TransformCtx<'a>, ctx: &mut TraverseCtx<'a>) {
        if transform_ctx.source_type.is_script() {
            self.insert_require_statements(transform_ctx, ctx);
        } else {
            self.insert_import_statements(transform_ctx, ctx);
        }
    }

    fn insert_import_statements(
        &self,
        transform_ctx: &TransformCtx<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut imports = self.imports.borrow_mut();
        let stmts =
            imports.drain(..).map(|(source, names)| Self::get_named_import(source, names, ctx));
        transform_ctx.top_level_statements.insert_statements(stmts);
    }

    fn insert_require_statements(
        &self,
        transform_ctx: &TransformCtx<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let mut imports = self.imports.borrow_mut();
        if imports.is_empty() {
            return;
        }

        let require_symbol_id = ctx.scopes().get_root_binding("require");
        let stmts = imports
            .drain(..)
            .map(|(source, names)| Self::get_require(source, names, require_symbol_id, ctx));
        transform_ctx.top_level_statements.insert_statements(stmts);
    }

    fn get_named_import(
        source: Atom<'a>,
        names: Vec<NamedImport<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let specifiers = ctx.ast.vec_from_iter(names.into_iter().map(|name| {
            let local = name.local.unwrap_or_else(|| name.imported.clone());
            ImportDeclarationSpecifier::ImportSpecifier(ctx.ast.alloc_import_specifier(
                SPAN,
                ModuleExportName::IdentifierName(IdentifierName::new(SPAN, name.imported)),
                BindingIdentifier::new_with_symbol_id(SPAN, local, name.symbol_id),
                ImportOrExportKind::Value,
            ))
        }));
        let import_stmt = ctx.ast.module_declaration_import_declaration(
            SPAN,
            Some(specifiers),
            StringLiteral::new(SPAN, source),
            NONE,
            ImportOrExportKind::Value,
        );
        ctx.ast.statement_module_declaration(import_stmt)
    }

    fn get_require(
        source: Atom<'a>,
        names: std::vec::Vec<NamedImport<'a>>,
        require_symbol_id: Option<SymbolId>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let var_kind = VariableDeclarationKind::Var;
        let ident = ctx.create_reference_id(
            SPAN,
            Atom::from("require"),
            require_symbol_id,
            ReferenceFlags::read(),
        );
        let callee = ctx.ast.expression_from_identifier_reference(ident);

        let args = {
            let arg = Argument::from(ctx.ast.expression_string_literal(SPAN, source));
            ctx.ast.vec1(arg)
        };
        let name = names.into_iter().next().unwrap();
        let id = {
            let ident = BindingIdentifier::new_with_symbol_id(SPAN, name.imported, name.symbol_id);
            ctx.ast.binding_pattern(
                ctx.ast.binding_pattern_kind_from_binding_identifier(ident),
                NONE,
                false,
            )
        };
        let decl = {
            let init = ctx.ast.expression_call(SPAN, callee, NONE, args, false);
            let decl = ctx.ast.variable_declarator(SPAN, var_kind, id, Some(init), false);
            ctx.ast.vec1(decl)
        };
        let var_decl = ctx.ast.declaration_variable(SPAN, var_kind, decl, false);
        ctx.ast.statement_declaration(var_decl)
    }
}
