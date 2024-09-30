use std::cell::RefCell;

use indexmap::IndexMap;
use oxc_ast::{ast::*, NONE};
use oxc_semantic::ReferenceFlags;
use oxc_span::{Atom, SPAN};
use oxc_syntax::symbol::SymbolId;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::TransformCtx;

impl<'a> NamedImport<'a> {
    pub fn new(imported: Atom<'a>, local: Option<Atom<'a>>, symbol_id: SymbolId) -> Self {
        Self { imported, local, symbol_id }
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum ImportKind {
    Import,
    Require,
}

#[derive(Hash, Eq, PartialEq)]
pub struct ImportType<'a> {
    kind: ImportKind,
    source: Atom<'a>,
}

impl<'a> ImportType<'a> {
    fn new(kind: ImportKind, source: Atom<'a>) -> Self {
        Self { kind, source }
    }
}

/// Manage import statement globally
/// <https://github.com/nicolo-ribaudo/babel/tree/main/packages/babel-helper-module-imports>
pub struct ModuleImports<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> Traverse<'a> for ModuleImports<'a, 'ctx> {
    fn exit_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let statements =
            ctx.ast.vec_from_iter(self.ctx.module_imports.imports.borrow_mut().drain(..).map(
                |(import_type, names)| match import_type.kind {
                    ImportKind::Import => Self::get_named_import(import_type.source, names, ctx),
                    ImportKind::Require => Self::get_require(import_type.source, names, ctx),
                },
            ));
        node.body.splice(0..0, statements);
    }
}

impl<'a, 'ctx> ModuleImports<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }

    fn get_named_import(
        source: Atom<'a>,
        names: std::vec::Vec<NamedImport<'a>>,
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
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let var_kind = VariableDeclarationKind::Var;
        let symbol_id = ctx.scopes().get_root_binding("require");
        let ident =
            ctx.create_reference_id(SPAN, Atom::from("require"), symbol_id, ReferenceFlags::read());
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

pub struct NamedImport<'a> {
    imported: Atom<'a>,
    local: Option<Atom<'a>>, // Not used in `require`
    symbol_id: SymbolId,
}

/// Store for `ModuleImports`s to be added to enclosing statement block.
pub struct ModuleImportsStore<'a> {
    imports: RefCell<IndexMap<ImportType<'a>, std::vec::Vec<NamedImport<'a>>>>,
}

impl<'a> ModuleImportsStore<'a> {
    pub fn new() -> Self {
        Self { imports: RefCell::new(IndexMap::new()) }
    }
}

impl<'a> ModuleImportsStore<'a> {
    /// Add `import { named_import } from 'source'`
    pub fn add_import(&self, source: Atom<'a>, import: NamedImport<'a>) {
        self.imports
            .borrow_mut()
            .entry(ImportType::new(ImportKind::Import, source))
            .or_default()
            .push(import);
    }

    /// Add `var named_import from 'source'`
    pub fn add_require(&self, source: Atom<'a>, import: NamedImport<'a>) {
        self.imports
            .borrow_mut()
            .entry(ImportType::new(ImportKind::Require, source))
            .or_default()
            .push(import);
    }
}
