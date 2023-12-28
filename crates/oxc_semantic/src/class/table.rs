use oxc_ast::ast::MethodDefinitionKind;
use oxc_index::IndexVec;
use oxc_span::{Atom, Span};
use oxc_syntax::class::{ClassId, MethodId, PropertyId};
use rustc_hash::FxHashMap;

use crate::node::AstNodeId;

#[derive(Debug)]
pub struct Property {
    pub name: Atom,
    pub span: Span,
    pub is_private: bool,
}

impl Property {
    pub fn new(name: Atom, span: Span, is_private: bool) -> Self {
        Self { name, span, is_private }
    }
}

#[derive(Debug)]
pub struct Method {
    pub name: Atom,
    pub span: Span,
    pub is_private: bool,
    pub kind: MethodDefinitionKind,
}

impl Method {
    pub fn new(name: Atom, span: Span, is_private: bool, kind: MethodDefinitionKind) -> Self {
        Self { name, span, is_private, kind }
    }
}

#[derive(Debug)]
pub struct PrivateIdentifierReference {
    pub id: AstNodeId,
    pub name: Atom,
    pub span: Span,
    pub property_id: Option<PropertyId>,
    /// If the private identifier is used in a get/set method, this will be has 2 method ids
    pub method_ids: Vec<MethodId>,
}

impl PrivateIdentifierReference {
    pub fn new(
        id: AstNodeId,
        name: Atom,
        span: Span,
        property_id: Option<PropertyId>,
        method_ids: Vec<MethodId>,
    ) -> Self {
        Self { id, name, span, property_id, method_ids }
    }
}

/// Class Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct ClassTable {
    pub parent_ids: FxHashMap<ClassId, ClassId>,
    pub declarations: IndexVec<ClassId, AstNodeId>,
    // PropertyDefinition
    pub properties: IndexVec<ClassId, IndexVec<PropertyId, Property>>,
    // MethodDefinition
    pub methods: IndexVec<ClassId, IndexVec<MethodId, Method>>,
    // PrivateIdentifier reference
    pub private_identifiers: IndexVec<ClassId, Vec<PrivateIdentifierReference>>,
}

impl ClassTable {
    pub fn ancestors(&self, class_id: ClassId) -> impl Iterator<Item = ClassId> + '_ {
        std::iter::successors(Some(class_id), |class_id| self.parent_ids.get(class_id).copied())
    }

    pub fn iter_enumerated(&self) -> impl Iterator<Item = (ClassId, &AstNodeId)> + '_ {
        self.declarations.iter_enumerated()
    }

    pub fn iter_private_identifiers(
        &self,
        class_id: ClassId,
    ) -> impl Iterator<Item = &PrivateIdentifierReference> + '_ {
        self.private_identifiers[class_id].iter()
    }

    pub fn get_node_id(&self, class_id: ClassId) -> AstNodeId {
        self.declarations[class_id]
    }

    pub fn get_property_id(&self, class_id: ClassId, name: &Atom) -> Option<PropertyId> {
        self.properties[class_id].iter_enumerated().find_map(|(property_id, property)| {
            if property.name == *name {
                Some(property_id)
            } else {
                None
            }
        })
    }

    pub fn get_method_ids(&self, class_id: ClassId, name: &Atom) -> Vec<MethodId> {
        let mut method_ids = vec![];
        for (method_id, method) in self.methods[class_id].iter_enumerated() {
            if method.name == *name {
                method_ids.push(method_id);
                // Only have 1 method id for MethodDefinition::Method
                if method.kind.is_method() {
                    break;
                }
                // Maximum 2 method ids, for get/set
                if method_ids.len() == 2 {
                    break;
                }
            }
        }

        method_ids
    }

    pub fn has_private_definition(&self, class_id: ClassId, name: &Atom) -> bool {
        self.properties[class_id].iter().any(|p| p.is_private && p.name == *name)
            || self.methods[class_id].iter().any(|m| m.is_private && m.name == *name)
    }

    pub fn declare_class(&mut self, parent_id: Option<ClassId>, ast_node_id: AstNodeId) -> ClassId {
        let class_id = self.declarations.push(ast_node_id);
        if let Some(parent_id) = parent_id {
            self.parent_ids.insert(class_id, parent_id);
        };
        self.properties.push(IndexVec::default());
        self.methods.push(IndexVec::default());
        self.private_identifiers.push(Vec::new());
        class_id
    }

    pub fn add_property(&mut self, class_id: ClassId, property: Property) {
        self.properties[class_id].push(property);
    }

    pub fn add_method(&mut self, class_id: ClassId, method: Method) {
        self.methods[class_id].push(method);
    }

    pub fn add_private_identifier_reference(
        &mut self,
        class_id: ClassId,
        private_identifier_reference: PrivateIdentifierReference,
    ) {
        self.private_identifiers[class_id].push(private_identifier_reference);
    }
}
