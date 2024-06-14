#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_ast::{visit::walk::walk_export_default_declaration, Visit};
use oxc_span::Atom;
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct ScopeTree<'a> {
    references: FxHashSet<Atom<'a>>,
}

impl<'a> ScopeTree<'a> {
    pub fn new() -> Self {
        Self { references: FxHashSet::default() }
    }

    pub fn has_reference(&self, name: &Atom<'a>) -> bool {
        self.references.contains(name)
    }

    pub fn references_len(&self) -> usize {
        self.references.len()
    }
}

impl<'a> Visit<'a> for ScopeTree<'a> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.references.insert(ident.name.clone());
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        for specifier in &decl.specifiers {
            if let ModuleExportName::Identifier(ident) = &specifier.local {
                self.references.insert(ident.name.clone());
            }
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
            self.references.insert(ident.name.clone());
        } else {
            walk_export_default_declaration(self, decl);
        }
    }
}
