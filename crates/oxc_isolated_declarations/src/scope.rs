use std::cell::Cell;

use bitflags::bitflags;
use rustc_hash::FxHashMap;

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{visit::walk::*, Visit};
use oxc_span::Atom;
use oxc_syntax::scope::{ScopeFlags, ScopeId};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KindFlags: u8 {
        const Value = 1 << 0;
        const Type = 1 << 1;
        const All = Self::Value.bits() | Self::Type.bits();
    }
}

/// Declaration scope.
#[derive(Debug)]
struct Scope<'a> {
    bindings: FxHashMap<Atom<'a>, KindFlags>,
    references: FxHashMap<Atom<'a>, KindFlags>,
    flags: ScopeFlags,
}

impl<'a> Scope<'a> {
    fn new(flags: ScopeFlags) -> Self {
        Self { bindings: FxHashMap::default(), references: FxHashMap::default(), flags }
    }
}

/// Linear tree of declaration scopes.
#[derive(Debug)]
pub struct ScopeTree<'a> {
    levels: Vec<Scope<'a>>,
}

impl<'a> ScopeTree<'a> {
    pub fn new() -> Self {
        let levels = vec![Scope::new(ScopeFlags::Top)];
        Self { levels }
    }

    pub fn is_ts_module_block(&self) -> bool {
        let scope = self.levels.last().unwrap();
        scope.flags.contains(ScopeFlags::TsModuleBlock)
    }

    pub fn has_reference(&self, name: &str) -> bool {
        let scope = self.levels.last().unwrap();
        scope.references.contains_key(name)
    }

    fn add_binding(&mut self, name: Atom<'a>, flags: KindFlags) {
        let scope = self.levels.last_mut().unwrap();
        scope.bindings.insert(name, flags);
    }

    fn add_reference(&mut self, name: Atom<'a>, flags: KindFlags) {
        let scope = self.levels.last_mut().unwrap();
        scope.references.insert(name, flags);
    }

    /// Resolve references in the current scope, and propagate unresolved ones.
    fn resolve_references(&mut self) {
        debug_assert!(self.levels.len() >= 2);

        // Remove the current scope.
        let current_scope = self.levels.pop().unwrap();

        // Resolve references in the current scope.
        let current_bindings = current_scope.bindings;
        let mut current_references = current_scope.references;
        current_references.retain(|name, reference_flags| {
            !current_bindings.get(name).is_some_and(|flags| flags.contains(*reference_flags))
        });

        // Merge unresolved references to the parent scope.
        self.levels.last_mut().unwrap().references.extend(current_references);
    }
}

impl<'a> Visit<'a> for ScopeTree<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        let scope = Scope::new(flags);
        self.levels.push(scope);
    }

    fn leave_scope(&mut self) {
        self.resolve_references();
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.add_reference(ident.name.clone(), KindFlags::Value);
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPatternKind::BindingIdentifier(ident) = &pattern.kind {
            self.add_binding(ident.name.clone(), KindFlags::Value);
        }
        walk_binding_pattern(self, pattern);
    }

    fn visit_ts_type_name(&mut self, name: &TSTypeName<'a>) {
        if let TSTypeName::IdentifierReference(ident) = name {
            self.add_reference(ident.name.clone(), KindFlags::Type);
        } else {
            walk_ts_type_name(self, name);
        }
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        if let Some(type_name) = ty.expr_name.as_ts_type_name() {
            let ident = TSTypeName::get_first_name(type_name);
            self.add_reference(ident.name.clone(), KindFlags::Value);
        } else {
            walk_ts_type_query(self, ty);
        }
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(declaration) = &decl.declaration {
            walk_declaration(self, declaration);
        } else if decl.source.is_none() {
            // export { ... }
            for specifier in &decl.specifiers {
                if let Some(name) = specifier.local.identifier_name() {
                    self.add_reference(name, KindFlags::All);
                }
            }
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
            self.add_reference(ident.name.clone(), KindFlags::All);
        } else {
            walk_export_default_declaration(self, decl);
        }
    }

    fn visit_declaration(&mut self, declaration: &Declaration<'a>) {
        match declaration {
            Declaration::VariableDeclaration(_) => {
                // add binding in BindingPattern
            }
            Declaration::FunctionDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_binding(id.name.clone(), KindFlags::Value);
                }
            }
            Declaration::ClassDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_binding(id.name.clone(), KindFlags::Value);
                }
            }
            Declaration::TSTypeAliasDeclaration(decl) => {
                self.add_binding(decl.id.name.clone(), KindFlags::Type);
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                self.add_binding(decl.id.name.clone(), KindFlags::Type);
            }
            Declaration::TSEnumDeclaration(decl) => {
                self.add_binding(decl.id.name.clone(), KindFlags::All);
            }
            Declaration::TSModuleDeclaration(decl) => {
                if let TSModuleDeclarationName::Identifier(ident) = &decl.id {
                    self.add_binding(ident.name.clone(), KindFlags::All);
                }
            }
            Declaration::TSImportEqualsDeclaration(decl) => {
                self.add_binding(decl.id.name.clone(), KindFlags::Value);
            }
        }
        walk_declaration(self, declaration);
    }
}
