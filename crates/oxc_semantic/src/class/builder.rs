use oxc_ast::{
    AstKind,
    ast::{
        AccessorProperty, ClassBody, ClassElement, MethodDefinition, MethodDefinitionKind,
        PrivateIdentifier, PropertyDefinition,
    },
};
use oxc_span::GetSpan;
use oxc_syntax::class::{ClassId, ElementKind};

use crate::{AstNodes, NodeId};

use super::{
    ClassTable,
    table::{Element, PrivateIdentifierReference},
};

#[derive(Debug, Default)]
pub struct ClassTableBuilder<'a> {
    pub current_class_id: Option<ClassId>,
    pub classes: ClassTable<'a>,
}

impl<'a> ClassTableBuilder<'a> {
    pub fn new() -> Self {
        Self { current_class_id: None, classes: ClassTable::default() }
    }

    pub fn build(self) -> ClassTable<'a> {
        self.classes
    }

    pub fn declare_class_body(
        &mut self,
        class: &ClassBody<'a>,
        current_node_id: NodeId,
        nodes: &AstNodes,
    ) {
        let parent_id = nodes.parent_id(current_node_id);
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

    pub fn declare_class_accessor(&mut self, property: &AccessorProperty<'a>) {
        let is_private = property.key.is_private_identifier();
        let name = property.key.name();

        if let Some(name) = name
            && let Some(class_id) = self.current_class_id
        {
            self.classes.add_element(
                class_id,
                Element::new(
                    name,
                    property.key.span(),
                    property.r#static,
                    is_private,
                    ElementKind::Accessor,
                ),
            );
        }
    }

    pub fn declare_class_property(&mut self, property: &PropertyDefinition<'a>) {
        let is_private = property.key.is_private_identifier();
        let name = property.key.name();

        if let Some(name) = name
            && let Some(class_id) = self.current_class_id
        {
            self.classes.add_element(
                class_id,
                Element::new(
                    name,
                    property.key.span(),
                    property.r#static,
                    is_private,
                    ElementKind::Property,
                ),
            );
        }
    }

    pub fn add_private_identifier_reference(
        &mut self,
        ident: &PrivateIdentifier<'a>,
        current_node_id: NodeId,
        nodes: &AstNodes,
    ) {
        let parent_kind = nodes.parent_kind(current_node_id);

        if (matches!(parent_kind, AstKind::PrivateInExpression(_))
            || parent_kind.is_member_expression_kind())
            && let Some(class_id) = self.current_class_id
        {
            let element_ids =
                self.classes.get_element_ids(class_id, &ident.name, /* is_private */ true);

            let reference = PrivateIdentifierReference::new(
                current_node_id,
                ident.name,
                ident.span,
                element_ids,
            );
            self.classes.add_private_identifier_reference(class_id, reference);
        }
    }

    pub fn declare_class_method(&mut self, method: &MethodDefinition<'a>) {
        if method.kind.is_constructor() || method.value.is_typescript_syntax() {
            return;
        }
        let is_private = method.key.is_private_identifier();
        let name = method.key.name();

        if let Some(name) = name
            && let Some(class_id) = self.current_class_id
        {
            self.classes.add_element(
                class_id,
                Element::new(
                    name,
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

    pub fn pop_class(&mut self) {
        self.current_class_id = self
            .current_class_id
            .and_then(|current_class_id| self.classes.parent_ids.get(&current_class_id).copied());
    }
}
