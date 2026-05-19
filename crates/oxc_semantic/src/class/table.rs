use std::borrow::Cow;

use rustc_hash::FxHashMap;

use oxc_index::IndexVec;
use oxc_span::Span;
use oxc_str::Ident;
use oxc_syntax::{
    class::{ClassId, ElementId, ElementKind},
    node::NodeId,
};

#[derive(Debug)]
pub struct Element<'a> {
    pub name: Cow<'a, str>,
    pub span: Span,
    pub is_private: bool,
    pub r#static: bool,
    pub kind: ElementKind,
}

impl<'a> Element<'a> {
    pub fn new(
        name: Cow<'a, str>,
        span: Span,
        r#static: bool,
        is_private: bool,
        kind: ElementKind,
    ) -> Self {
        Self { name, span, is_private, r#static, kind }
    }
}

#[derive(Debug)]
pub struct PrivateIdentifierReference<'a> {
    pub id: NodeId,
    pub name: Ident<'a>,
    pub span: Span,
    pub element_ids: Vec<ElementId>,
}

impl<'a> PrivateIdentifierReference<'a> {
    pub fn new(id: NodeId, name: Ident<'a>, span: Span, element_ids: Vec<ElementId>) -> Self {
        Self { id, name, span, element_ids }
    }
}

/// Class Table
///
/// `SoA` (Struct of Arrays) for memory efficiency.
#[derive(Debug, Default)]
pub struct ClassTable<'a> {
    pub parent_ids: FxHashMap<ClassId, ClassId>,
    pub declarations: IndexVec<ClassId, NodeId>,
    pub elements: IndexVec<ClassId, IndexVec<ElementId, Element<'a>>>,
    pub private_identifier_references: IndexVec<ClassId, Vec<PrivateIdentifierReference<'a>>>,
}

impl<'a> ClassTable<'a> {
    pub fn ancestors(&self, class_id: ClassId) -> impl Iterator<Item = ClassId> + '_ {
        std::iter::successors(Some(class_id), |class_id| self.parent_ids.get(class_id).copied())
    }

    pub fn len(&self) -> usize {
        self.declarations.raw.len()
    }

    pub fn iter_enumerated(&self) -> impl Iterator<Item = (ClassId, &NodeId)> + '_ {
        self.declarations.iter_enumerated()
    }

    pub fn iter_private_identifiers(
        &self,
        class_id: ClassId,
    ) -> impl Iterator<Item = &PrivateIdentifierReference<'_>> + '_ {
        self.private_identifier_references[class_id].iter()
    }

    pub fn get_node_id(&self, class_id: ClassId) -> NodeId {
        self.declarations[class_id]
    }

    pub fn get_element_ids(
        &self,
        class_id: ClassId,
        name: Ident<'_>,
        is_private: bool,
    ) -> Vec<ElementId> {
        let mut element_ids = vec![];
        for (element_id, element) in self.elements[class_id].iter_enumerated() {
            if element.name == name && element.is_private == is_private {
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

    pub fn has_private_definition(&self, class_id: ClassId, name: Ident<'_>) -> bool {
        self.elements[class_id].iter().any(|p| p.is_private && p.name == name)
    }

    /// Declare a new class.
    ///
    /// `elements_capacity` / `private_id_refs_capacity` pre-size the *inner*
    /// vecs for this class. Pass `0` when no estimate is available; pass an
    /// average derived from [`crate::Stats::class_elements`] /
    /// [`crate::Stats::class_private_id_refs`] to avoid the first growth in
    /// `add_element` / `add_private_identifier_reference`.
    pub fn declare_class(
        &mut self,
        parent_id: Option<ClassId>,
        node_id: NodeId,
        elements_capacity: usize,
        private_id_refs_capacity: usize,
    ) -> ClassId {
        let class_id = self.declarations.push(node_id);
        if let Some(parent_id) = parent_id {
            self.parent_ids.insert(class_id, parent_id);
        }
        self.elements.push(IndexVec::with_capacity(elements_capacity));
        self.private_identifier_references.push(Vec::with_capacity(private_id_refs_capacity));
        class_id
    }

    pub fn add_element(&mut self, class_id: ClassId, element: Element<'a>) {
        self.elements[class_id].push(element);
    }

    pub fn add_private_identifier_reference(
        &mut self,
        class_id: ClassId,
        private_identifier_reference: PrivateIdentifierReference<'a>,
    ) {
        self.private_identifier_references[class_id].push(private_identifier_reference);
    }
}
