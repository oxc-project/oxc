use std::rc::Rc;

use oxc_ast::AstKind;
use oxc_semantic::{Element, Semantic};
use oxc_syntax::class::{ClassId, ElementKind};

pub struct ClassTester<'a> {
    /// Reference to semantic analysis results, from [`SemanticTester`]
    semantic: Rc<Semantic<'a>>,
    class_id: ClassId,
}

impl<'a> ClassTester<'a> {
    pub(super) fn has_class(semantic: Semantic<'a>, name: &str) -> Self {
        let class_id = semantic.classes().iter_enumerated().find_map(|(class_id, ast_node_id)| {
            let kind = semantic.nodes().kind(*ast_node_id);
            if let AstKind::Class(class) = kind {
                if class.id.clone().is_some_and(|id| id.name == name) {
                    return Some(class_id);
                };
            }
            None
        });
        ClassTester {
            semantic: Rc::new(semantic),
            class_id: class_id.unwrap_or_else(|| panic!("Cannot find {name} class")),
        }
    }

    pub fn has_number_of_elements(&self, len: usize) -> &Self {
        let element_len = self.semantic.classes().elements[self.class_id].len();
        debug_assert!(element_len == len, "Expected {len} elements, found {element_len}");
        self
    }

    pub fn has_property(&self, name: &str) -> &Self {
        self.get_property_of_kind(name, ElementKind::Property);
        self
    }

    pub fn has_private_property(&self, name: &str) -> &Self {
        let property = self.get_property_of_kind(name, ElementKind::Property);
        debug_assert!(property.is_private, "Found property {name} but it is not private");
        self
    }

    pub fn has_method(&self, name: &str) -> &Self {
        self.get_property_of_kind(name, ElementKind::Method);
        self
    }

    pub fn has_accessor(&self, name: &str) -> &Self {
        self.get_property_of_kind(name, ElementKind::Accessor);
        self
    }

    fn get_property_of_kind(&self, name: &str, kind: ElementKind) -> &Element {
        let property =
            self.semantic.classes().elements[self.class_id].iter().find(|m| m.name == name);
        debug_assert!(property.is_some(), "Expected accessor `{name}` not found");

        let property = property.unwrap();
        debug_assert!(
            property.kind.contains(kind),
            "Found property `{name}` but it is not of kind {kind:?}"
        );

        property
    }
}
