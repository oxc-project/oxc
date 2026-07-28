use std::cell::Cell;

use bitflags::bitflags;
use rustc_hash::FxHashMap;

use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::*};
use oxc_str::Str;
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
    bindings: FxHashMap<Str<'a>, KindFlags>,
    references: FxHashMap<Str<'a>, KindFlags>,
    flags: ScopeFlags,
}

impl Scope<'_> {
    fn new(flags: ScopeFlags) -> Self {
        Self { bindings: FxHashMap::default(), references: FxHashMap::default(), flags }
    }
}

/// Linear tree of declaration scopes.
#[derive(Debug)]
pub struct ScopeTree<'a> {
    levels: Vec<Scope<'a>>,
    /// Pool of scopes whose maps have been emptied. Scopes are entered and left in a stack
    /// pattern, so rather than dropping a left scope's two `FxHashMap`s (and allocating +
    /// re-growing fresh ones on the next `enter_scope`), keep them here to reuse their heap
    /// allocations and retained capacity.
    free_scopes: Vec<Scope<'a>>,
}

impl<'a> ScopeTree<'a> {
    pub fn new() -> Self {
        let levels = vec![Scope::new(ScopeFlags::Top)];
        Self { levels, free_scopes: Vec::new() }
    }

    pub fn is_ts_module_block(&self) -> bool {
        let scope = self.levels.last().unwrap();
        scope.flags.contains(ScopeFlags::TsModuleBlock)
    }

    pub fn has_reference(&self, name: &str) -> bool {
        let scope = self.levels.last().unwrap();
        scope.references.contains_key(name)
    }

    /// Check if the current scope has a value reference for the given name.
    pub fn has_value_reference(&self, name: &str) -> bool {
        let scope = self.levels.last().unwrap();
        scope.references.get(name).iter().any(|flags| flags.contains(KindFlags::Value))
    }

    fn add_binding(&mut self, name: Str<'a>, flags: KindFlags) {
        let scope = self.levels.last_mut().unwrap();
        scope.bindings.insert(name, flags);
    }

    fn add_reference(&mut self, name: Str<'a>, flags: KindFlags) {
        let scope = self.levels.last_mut().unwrap();
        scope.references.entry(name).and_modify(|f| *f |= flags).or_insert(flags);
    }

    /// Resolve references in the current scope, and propagate unresolved ones.
    fn resolve_references(&mut self) {
        debug_assert!(self.levels.len() >= 2);

        // Remove the current scope, taking ownership of its maps so they can be recycled.
        let Scope { mut bindings, mut references, .. } = self.levels.pop().unwrap();

        // Resolve references in the current scope against its own bindings.
        references.retain(|name, reference_flags| {
            !bindings.get(name).is_some_and(|flags| flags.contains(*reference_flags))
        });

        // Merge unresolved references to the parent scope. `drain` empties the map while keeping
        // its heap allocation, so it can be reused by a later `enter_scope`.
        let parent_scope = self.levels.last_mut().unwrap();
        for (name, flags) in references.drain() {
            parent_scope.references.entry(name).and_modify(|f| *f |= flags).or_insert(flags);
        }

        // Recycle both now-empty maps (capacity retained) to avoid re-allocating + re-growing
        // them for the next scope. `flags` is a placeholder; `enter_scope` overwrites it.
        bindings.clear();
        self.free_scopes.push(Scope { bindings, references, flags: ScopeFlags::empty() });
    }
}

impl<'a> Visit<'a> for ScopeTree<'a> {
    fn enter_scope(&mut self, flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        // Reuse a recycled scope's map allocations when one is available.
        let scope = match self.free_scopes.pop() {
            Some(mut scope) => {
                scope.flags = flags;
                scope
            }
            None => Scope::new(flags),
        };
        self.levels.push(scope);
    }

    fn leave_scope(&mut self) {
        self.resolve_references();
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.add_reference(ident.name.into(), KindFlags::Value);
    }

    fn visit_binding_pattern(&mut self, pattern: &BindingPattern<'a>) {
        if let BindingPattern::BindingIdentifier(ident) = pattern {
            self.add_binding(ident.name.into(), KindFlags::Value);
        }
        walk_binding_pattern(self, pattern);
    }

    fn visit_ts_type_name(&mut self, name: &TSTypeName<'a>) {
        if let TSTypeName::IdentifierReference(ident) = name {
            self.add_reference(ident.name.into(), KindFlags::Type);
        } else {
            walk_ts_type_name(self, name);
        }
    }

    // `typeof Value` or `typeof Value<Parameters>`
    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        if let Some(type_name) = ty.expr_name.as_ts_type_name() {
            if let Some(ident) = TSTypeName::get_identifier_reference(type_name) {
                self.add_reference(ident.name.into(), KindFlags::Value);
                // `typeof Type<Parameters>`
                //              ^^^^^^^^^^^
                if let Some(type_parameters) = &ty.type_arguments {
                    self.visit_ts_type_parameter_instantiation(type_parameters);
                }
            }
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
                    self.add_reference(name.into(), KindFlags::All);
                }
            }
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
            self.add_reference(ident.name.into(), KindFlags::All);
        } else {
            walk_export_default_declaration(self, decl);
        }
    }

    fn visit_declaration(&mut self, declaration: &Declaration<'a>) {
        #[expect(clippy::match_same_arms)]
        match declaration {
            Declaration::VariableDeclaration(_) => {
                // add binding in BindingPattern
            }
            Declaration::FunctionDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_binding(id.name.into(), KindFlags::Value);
                }
            }
            Declaration::ClassDeclaration(decl) => {
                if let Some(id) = decl.id.as_ref() {
                    self.add_binding(id.name.into(), KindFlags::Value);
                }
            }
            Declaration::TSTypeAliasDeclaration(decl) => {
                self.add_binding(decl.id.name.into(), KindFlags::Type);
            }
            Declaration::TSInterfaceDeclaration(decl) => {
                self.add_binding(decl.id.name.into(), KindFlags::Type);
            }
            Declaration::TSEnumDeclaration(decl) => {
                self.add_binding(decl.id.name.into(), KindFlags::All);
            }
            Declaration::TSModuleDeclaration(decl) => {
                if let TSModuleDeclarationName::Identifier(ident) = &decl.id {
                    self.add_binding(ident.name.into(), KindFlags::All);
                }
            }
            Declaration::TSGlobalDeclaration(_) => {
                // no binding
            }
            Declaration::TSImportEqualsDeclaration(decl) => {
                self.add_binding(decl.id.name.into(), KindFlags::Value);
            }
        }
        walk_declaration(self, declaration);
    }
}
