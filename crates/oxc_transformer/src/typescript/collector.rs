use oxc_ast::ast::{ExportNamedDeclaration, IdentifierReference};
use oxc_span::Atom;
use rustc_hash::FxHashSet;

/// Collects identifier references
/// Indicates whether the BindingIdentifier is referenced or used in the ExportNamedDeclaration
#[derive(Debug)]
pub struct TypeScriptReferenceCollector<'a> {
    names: FxHashSet<Atom<'a>>,
}

impl<'a> TypeScriptReferenceCollector<'a> {
    pub fn new() -> Self {
        Self { names: FxHashSet::default() }
    }

    pub fn has_reference(&self, name: &Atom) -> bool {
        self.names.contains(name)
    }

    pub fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.names.insert(ident.name.clone());
    }

    pub fn visit_transform_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if decl.export_kind.is_type() {
            return;
        }

        for specifier in &decl.specifiers {
            if specifier.export_kind.is_value() {
                self.names.insert(specifier.local.name().clone());
            }
        }
    }
}
