use std::rc::Rc;

use oxc_semantic::Semantic;
use oxc_syntax::class::ClassId;

pub struct ClassTester<'a> {
    /// Reference to semantic analysis results, from [`SemanticTester`]
    semantic: Rc<Semantic<'a>>,
    class_id: ClassId,
}

impl<'a> ClassTester<'a> {
    pub(super) fn has_class(semantic: Semantic<'a>, name: &str) -> Self {
        let class_id = semantic.classes().iter_enumerated().find_map(|(class_id, &node_id)| {
            let kind = semantic.nodes().kind(node_id);
            let class = kind.as_class()?;

            if class.id.clone().is_some_and(|id| id.name == name) {
                return Some(class_id);
            };

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
        let property = self.semantic.classes().elements[self.class_id]
            .iter()
            .find(|p| p.kind.is_property() && p.name == name);
        debug_assert!(property.is_some(), "Expected property `{name}` not found");
        self
    }

    pub fn has_method(&self, name: &str) -> &Self {
        let method = self.semantic.classes().elements[self.class_id]
            .iter()
            .find(|m| m.kind.is_method() && m.name == name);
        debug_assert!(method.is_some(), "Expected method `{name}` not found");
        self
    }

    pub fn has_accessor(&self, name: &str) -> &Self {
        let method = self.semantic.classes().elements[self.class_id]
            .iter()
            .find(|m| m.kind.is_accessor() && m.name == name);
        debug_assert!(method.is_some(), "Expected accessor `{name}` not found");
        self
    }
}
