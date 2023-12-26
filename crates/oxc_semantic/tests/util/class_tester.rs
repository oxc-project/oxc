use std::rc::Rc;

use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_syntax::class::ClassId;

pub struct ClassTester<'a> {
    /// Reference to semantic analysis results, from [`SemanticTester`]
    semantic: Rc<Semantic<'a>>,
    class_id: ClassId,
}

#[allow(dead_code)] // Only used in #[test]
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

    pub fn has_number_of_properties(&self, len: usize) -> &Self {
        let property_len = self.semantic.classes().properties[self.class_id].len();
        debug_assert!(property_len == len, "Expected {len} properties, found {property_len}");
        self
    }

    pub fn has_number_of_methods(&self, len: usize) -> &Self {
        let method_len = self.semantic.classes().methods[self.class_id].len();
        debug_assert!(method_len == len, "Expected `{len}` methods, found {method_len}");
        self
    }

    pub fn has_property(&self, name: &str) -> &Self {
        let property =
            self.semantic.classes().properties[self.class_id].iter().find(|p| p.name == name);
        debug_assert!(property.is_some(), "Expected property `{name}` not found");
        self
    }

    pub fn has_method(&self, name: &str) -> &Self {
        let method = self.semantic.classes().methods[self.class_id].iter().find(|m| m.name == name);
        debug_assert!(method.is_some(), "Expected method `{name}` not found");
        self
    }
}
