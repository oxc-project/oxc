use oxc_index::IndexVec;
use oxc_span::{Atom, Span};
use oxc_syntax::class::{ClassId, ElementId, ElementKind};
use rustc_hash::FxHashMap;

use crate::node::AstNodeId;

#[derive(Debug)]
pub struct Element {
    pub name: Atom,
    pub span: Span,
    pub is_private: bool,
    pub kind: ElementKind,
}

impl Element {
    pub fn new(name: Atom, span: Span, is_private: bool, kind: ElementKind) -> Self {
        Self { name, span, is_private, kind }
    }
}

#[derive(Debug)]
pub struct PrivateIdentifierReference {
    pub id: AstNodeId,
    pub name: Atom,
    pub span: Span,
    pub element_ids: Vec<ElementId>,
}

impl PrivateIdentifierReference {
    pub fn new(id: AstNodeId, name: Atom, span: Span, element_ids: Vec<ElementId>) -> Self {
        Self { id, name, span, element_ids }
    }
}

/// Class Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct ClassTable {
    pub parent_ids: FxHashMap<ClassId, ClassId>,
    pub declarations: IndexVec<ClassId, AstNodeId>,
    pub elements: IndexVec<ClassId, IndexVec<ElementId, Element>>,
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

    pub fn get_element_ids(&self, class_id: ClassId, name: &Atom) -> Vec<ElementId> {
        let mut element_ids = vec![];
        for (element_id, element) in self.elements[class_id].iter_enumerated() {
            if element.name == *name {
                element_ids.push(element_id);

                // Property or Accessor only has 1 element
                if element.kind.intersects(ElementKind::Accessor | ElementKind::Property) {
                    break;
                }

                // Maximum 2 method ids, for get/set
                if element_ids.len() == 2 {
                    break;
                }
            }
        }

        element_ids
    }

    pub fn has_private_definition(&self, class_id: ClassId, name: &Atom) -> bool {
        self.elements[class_id].iter().any(|p| p.is_private && p.name == *name)
    }

    pub fn declare_class(&mut self, parent_id: Option<ClassId>, ast_node_id: AstNodeId) -> ClassId {
        let class_id = self.declarations.push(ast_node_id);
        if let Some(parent_id) = parent_id {
            self.parent_ids.insert(class_id, parent_id);
        };
        self.elements.push(IndexVec::default());
        self.private_identifiers.push(Vec::new());
        class_id
    }

    pub fn add_element(&mut self, class_id: ClassId, element: Element) {
        self.elements[class_id].push(element);
    }

    pub fn add_private_identifier_reference(
        &mut self,
        class_id: ClassId,
        private_identifier_reference: PrivateIdentifierReference,
    ) {
        self.private_identifiers[class_id].push(private_identifier_reference);
    }
}
