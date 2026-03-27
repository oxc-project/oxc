use std::collections::HashMap;

use oxc_span::CompactStr;
use oxc_syntax::symbol::SymbolId;
use oxc_types::{InterfaceType, ObjectFlags, TypeData, TypeFlags, TypeId};
use smallvec::SmallVec;

use crate::Checker;

impl Checker<'_> {
    /// Get the declared type of a type-namespace symbol (type alias, interface,
    /// class, enum). Uses caching and cycle detection, mirroring tsgo's
    /// `getDeclaredTypeOfSymbol`.
    pub fn get_declared_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache
        if let Some(&cached) = self.declared_type_cache.get(&symbol_id) {
            return cached;
        }

        // Cycle detection
        if self.resolving_symbols.contains(&symbol_id) {
            return self.any_type;
        }

        self.resolving_symbols.push(symbol_id);
        let result = self.resolve_declared_type(symbol_id);
        self.resolving_symbols.pop();
        self.declared_type_cache.insert(symbol_id, result);
        result
    }

    /// Resolve the declared type from a type-namespace declaration.
    fn resolve_declared_type(&mut self, symbol_id: SymbolId) -> TypeId {
        use oxc_ast::AstKind;

        let node_id = self.semantic().scoping().symbol_declaration(symbol_id);
        let node = self.semantic().nodes().get_node(node_id);

        match node.kind() {
            AstKind::TSTypeAliasDeclaration(decl) => {
                // Non-generic type alias: follow to the aliased type.
                // TODO: handle type parameters for generic aliases
                self.get_type_from_type_node(&decl.type_annotation)
            }
            AstKind::TSInterfaceDeclaration(decl) => {
                self.get_type_of_interface_declaration(decl)
            }
            // TODO: ClassDeclaration, TSEnumDeclaration
            _ => self.any_type,
        }
    }

    /// Build an interface type from a TSInterfaceDeclaration.
    fn get_type_of_interface_declaration(
        &mut self,
        decl: &oxc_ast::ast::TSInterfaceDeclaration<'_>,
    ) -> TypeId {
        let mut properties = HashMap::new();

        for sig in &decl.body.body {
            use oxc_ast::ast::TSSignature;
            if let TSSignature::TSPropertySignature(prop) = sig {
                if let Some(name) = prop.key.static_name() {
                    let prop_type = if let Some(ann) = &prop.type_annotation {
                        self.get_type_from_type_node(&ann.type_annotation)
                    } else {
                        self.any_type
                    };
                    properties.insert(CompactStr::new(&name), prop_type);
                }
            }
            // TODO: method signatures, index signatures, call signatures
        }

        self.type_arena.new_type(
            TypeFlags::Object,
            ObjectFlags::Interface,
            TypeData::Interface(InterfaceType {
                target: None,
                resolved_type_arguments: SmallVec::new(),
                all_type_parameters: SmallVec::new(),
                this_type: None,
                resolved_base_types: SmallVec::new(),
                properties,
            }),
            None,
        )
    }
}
