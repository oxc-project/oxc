use oxc_ast::{
    ast::{
        AccessorProperty, ClassBody, ClassElement, MethodDefinition, MethodDefinitionKind,
        PrivateIdentifier, PropertyDefinition,
    },
    AstKind,
};
use oxc_span::GetSpan;
use oxc_syntax::class::{ClassId, ElementKind};

use crate::{AstNodes, NodeId};

use super::{
    table::{Element, PrivateIdentifierReference},
    ClassTable,
};

#[derive(Debug, Default)]
pub struct ClassTableBuilder {
    pub current_class_id: Option<ClassId>,
    pub classes: ClassTable,
}

impl ClassTableBuilder {
    pub fn new() -> Self {
        Self { current_class_id: None, classes: ClassTable::default() }
    }

    pub fn build(self) -> ClassTable {
        self.classes
    }

    pub fn declare_class_body(
        &mut self,
        class: &ClassBody,
        current_node_id: NodeId,
        nodes: &AstNodes,
    ) {
        let parent_id = nodes.parent_id(current_node_id).unwrap_or_else(|| unreachable!());
        self.current_class_id = Some(self.classes.declare_class(self.current_class_id, parent_id));

        for element in &class.body {
            match element {
                ClassElement::PropertyDefinition(definition) => {
                    self.declare_class_property(definition);
                }
                ClassElement::MethodDefinition(definition) => {
                    self.declare_class_method(definition);
                }
                ClassElement::AccessorProperty(definition) => {
                    self.declare_class_accessor(definition);
                }
                _ => {}
            }
        }
    }

    pub fn declare_class_accessor(&mut self, property: &AccessorProperty) {
        let is_private = property.key.is_private_identifier();
        let name = property.key.name();

        if let Some(name) = name {
            if let Some(class_id) = self.current_class_id {
                self.classes.add_element(
                    class_id,
                    Element::new(
                        name.into(),
                        property.key.span(),
                        property.r#static,
                        is_private,
                        ElementKind::Accessor,
                    ),
                );
            }
        }
    }

    pub fn declare_class_property(&mut self, property: &PropertyDefinition) {
        let is_private = property.key.is_private_identifier();
        let name = property.key.name();

        if let Some(name) = name {
            if let Some(class_id) = self.current_class_id {
                self.classes.add_element(
                    class_id,
                    Element::new(
                        name.into(),
                        property.key.span(),
                        property.r#static,
                        is_private,
                        ElementKind::Property,
                    ),
                );
            }
        }
    }

    pub fn add_private_identifier_reference(
        &mut self,
        ident: &PrivateIdentifier,
        current_node_id: NodeId,
        nodes: &AstNodes,
    ) {
        let parent_kind = nodes.parent_kind(current_node_id);
        if let Some(parent_kind) = parent_kind {
            if matches!(parent_kind, AstKind::PrivateInExpression(_) | AstKind::MemberExpression(_))
            {
                if let Some(class_id) = self.current_class_id {
                    let element_ids = self.classes.get_element_ids(class_id, &ident.name);

                    let reference = PrivateIdentifierReference::new(
                        current_node_id,
                        ident.name.to_compact_str(),
                        ident.span,
                        element_ids,
                    );
                    self.classes.add_private_identifier_reference(class_id, reference);
                }
            }
        }
    }

    pub fn declare_class_method(&mut self, method: &MethodDefinition) {
        if method.kind.is_constructor() || method.value.is_typescript_syntax() {
            return;
        }
        let is_private = method.key.is_private_identifier();
        let name = method.key.name();

        if let Some(name) = name {
            if let Some(class_id) = self.current_class_id {
                self.classes.add_element(
                    class_id,
                    Element::new(
                        name.into(),
                        method.key.span(),
                        method.r#static,
                        is_private,
                        match method.kind {
                            MethodDefinitionKind::Method => ElementKind::Method,
                            MethodDefinitionKind::Get => ElementKind::Method | ElementKind::Getter,
                            MethodDefinitionKind::Set => ElementKind::Method | ElementKind::Setter,
                            MethodDefinitionKind::Constructor => {
                                // Skip constructor
                                unreachable!()
                            }
                        },
                    ),
                );
            }
        }
    }

    pub fn pop_class(&mut self) {
        self.current_class_id = self
            .current_class_id
            .and_then(|current_class_id| self.classes.parent_ids.get(&current_class_id).copied());
    }
}
