use oxc_ast::{
    ast::{BindingIdentifier, ExportSpecifier, ImportSpecifier, ModuleExportName, Program},
    visit::walk::walk_import_specifier,
    Visit,
};
use oxc_semantic::{ScopeTree, SymbolTable};

pub struct SemanticTester {
    scopes: ScopeTree,
    symbols: SymbolTable,
    errors: Vec<String>,
}

impl SemanticTester {
    pub fn new(scopes: ScopeTree, symbols: SymbolTable) -> Self {
        Self { scopes, symbols, errors: Vec::new() }
    }

    pub fn test(mut self, program: &Program) -> Vec<String> {
        self.visit_program(program);
        self.errors
    }
}

impl<'a> Visit<'a> for SemanticTester {
    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        let symbol_id = it.symbol_id.get();
        if let Some(symbol_id) = symbol_id {
            if self.symbols.get_flag(symbol_id).is_empty() {
                self.errors.push(format!(
                    "Expect SymbolFlags for BindingIdentifier({}) to not be empty",
                    it.name
                ));
            }
            if !self.scopes.has_binding(self.symbols.get_scope_id(symbol_id), &it.name) {
                self.errors.push(format!(
                    "Cannot find BindingIdentifier({}) in the Scope corresponding to the Symbol",
                    it.name
                ));
            }
        } else {
            self.errors.push(format!("Expect BindingIdentifier({}) to have a symbol_id", it.name));
        }
    }
    fn visit_identifier_reference(&mut self, it: &oxc_ast::ast::IdentifierReference<'a>) {
        if let Some(reference_id) = it.reference_id.get() {
            let reference = self.symbols.get_reference(reference_id);
            if reference.flag().is_empty() {
                self.errors.push(format!(
                    "Expect ReferenceFlags for IdentifierReference({}) to not be empty",
                    it.name
                ));
            }
        } else {
            self.errors
                .push(format!("Expect IdentifierReference({}) to have a reference_id", it.name));
        }
    }
    fn visit_import_specifier(&mut self, it: &ImportSpecifier<'a>) {
        let symbol_id = it.local.symbol_id.get();
        if let Some(symbol_id) = symbol_id {
            if !self.symbols.get_flag(symbol_id).is_import() {
                self.errors.push(format!(
                    "Expect SymbolFlags for ImportSpecifier({}) should contain SymbolFlags::Import",
                    it.local.name
                ));
            }
        }
        walk_import_specifier(self, it);
    }
    fn visit_export_specifier(&mut self, it: &ExportSpecifier<'a>) {
        if let ModuleExportName::IdentifierReference(ident) = &it.local {
            let reference_id = ident.reference_id.get();
            if let Some(symbol_id) = reference_id
                .and_then(|reference_id| self.symbols.get_reference(reference_id).symbol_id())
            {
                if self.symbols.get_flag(symbol_id).is_empty() {
                    self.errors.push(format!(
                        "Expect SymbolFlags for ExportSpecifier({}) should contain SymbolFlags::Import",
                        it.local
                    ));
                }
            }
        }
    }
}
