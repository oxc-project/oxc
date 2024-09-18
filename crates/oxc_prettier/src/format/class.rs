use oxc_ast::ast::*;
use oxc_span::GetSpan;

use super::assignment::AssignmentLikeNode;
use crate::{
    array,
    doc::{Doc, DocBuilder},
    format::assignment,
    hardline, space, ss, Format, Prettier,
};

pub(super) fn print_class<'a>(p: &mut Prettier<'a>, class: &Class<'a>) -> Doc<'a> {
    let mut parts = p.vec();

    for decorator in &class.decorators {
        parts.push(ss!("@"));
        parts.push(decorator.expression.format(p));
        parts.extend(hardline!());
    }

    if class.declare {
        parts.push(ss!("declare "));
    }

    if class.r#abstract {
        parts.push(ss!("abstract "));
    }

    parts.push(ss!("class "));
    if let Some(id) = &class.id {
        parts.push(id.format(p));
    }

    if let Some(params) = &class.type_parameters {
        parts.push(params.format(p));
    }

    if class.id.is_some() || class.type_parameters.is_some() {
        parts.push(space!());
    }

    if let Some(super_class) = &class.super_class {
        parts.push(ss!("extends "));
        parts.push(super_class.format(p));

        if let Some(super_type_parameters) = &class.super_type_parameters {
            parts.push(super_type_parameters.format(p));
        }

        parts.push(space!());
    }

    if let Some(implements) = &class.implements {
        if implements.len() > 0 {
            parts.push(ss!("implements "));

            let mut print_comma = false;
            for implementation in implements {
                if print_comma {
                    parts.push(ss!(", "));
                } else {
                    print_comma = true;
                }

                parts.push(implementation.format(p));
            }

            parts.push(space!());
        }
    }

    parts.push(class.body.format(p));
    Doc::Array(parts)
}

pub(super) fn print_class_body<'a>(p: &mut Prettier<'a>, class_body: &ClassBody<'a>) -> Doc<'a> {
    let mut parts_inner = p.vec();

    for (i, node) in class_body.body.iter().enumerate() {
        parts_inner.push(node.format(p));

        if !p.options.semi
            && node.is_property()
            && should_print_semicolon_after_class_property(node, class_body.body.get(i + 1))
        {
            parts_inner.push(ss!(";"));
        }

        if i < class_body.body.len() - 1 {
            parts_inner.extend(hardline!());

            if p.is_next_line_empty(node.span()) {
                parts_inner.extend(hardline!());
            }
        }
    }

    // TODO: if there are any dangling comments, print them

    let mut parts = p.vec();
    // TODO is class_body.len() != 0, print hardline after heritage

    parts.push(ss!("{"));
    if !parts_inner.is_empty() {
        let indent = {
            let mut parts = p.vec();
            parts.extend(hardline!());
            parts.push(Doc::Array(parts_inner));
            Doc::Indent(parts)
        };
        parts.push(array![p, indent]);
        parts.extend(hardline!());
    }

    parts.push(ss!("}"));

    Doc::Array(parts)
}

#[derive(Debug)]
pub enum ClassMemberish<'a, 'b> {
    PropertyDefinition(&'b PropertyDefinition<'a>),
    AccessorProperty(&'b AccessorProperty<'a>),
}

impl<'a, 'b> ClassMemberish<'a, 'b> {
    fn format_key(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.key.format(p)
            }
            ClassMemberish::AccessorProperty(accessor_property) => accessor_property.key.format(p),
        }
    }

    fn decorators(&self) -> Option<&oxc_allocator::Vec<Decorator<'a>>> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                Some(&property_definition.decorators)
            }

            ClassMemberish::AccessorProperty(accessor_property) => None,
        }
    }

    fn is_static(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.r#static,
            ClassMemberish::AccessorProperty(accessor_property) => accessor_property.r#static,
        }
    }

    fn is_override(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.r#override
            }
            ClassMemberish::AccessorProperty(accessor_property) => false,
        }
    }

    fn is_readonly(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.readonly,
            ClassMemberish::AccessorProperty(_) => true,
        }
    }

    fn is_declare(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.declare,
            ClassMemberish::AccessorProperty(_) => false,
        }
    }

    fn is_abstract(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
            }
            ClassMemberish::AccessorProperty(accessor_property) => {
                accessor_property.r#type == AccessorPropertyType::TSAbstractAccessorProperty
            }
        }
    }

    fn is_optional(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.optional,
            ClassMemberish::AccessorProperty(_) => false,
        }
    }

    fn is_definite(&self) -> bool {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => property_definition.definite,
            ClassMemberish::AccessorProperty(accessor_property) => accessor_property.definite,
        }
    }

    fn right_expr(&self) -> Option<&Expression<'a>> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.value.as_ref()
            }
            ClassMemberish::AccessorProperty(_) => None,
        }
    }

    fn format_accessibility(&self, p: &mut Prettier<'a>) -> Option<Doc<'a>> {
        match self {
            ClassMemberish::AccessorProperty(def) => def.accessibility.map(|v| ss!(v.as_str())),
            ClassMemberish::PropertyDefinition(def) => def.accessibility.map(|v| ss!(v.as_str())),
        }
    }

    fn format_type_annotation(&self, p: &mut Prettier<'a>) -> Option<Doc<'a>> {
        match self {
            ClassMemberish::PropertyDefinition(property_definition) => {
                property_definition.type_annotation.as_ref().map(|v| v.type_annotation.format(p))
            }
            ClassMemberish::AccessorProperty(accessor_definition) => {
                accessor_definition.type_annotation.as_ref().map(|v| v.type_annotation.format(p))
            }
        }
    }
}

pub(super) fn print_class_property<'a>(
    p: &mut Prettier<'a>,
    node: &ClassMemberish<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();

    if let Some(decarators) = node.decorators() {
        for decorator in decarators {
            parts.push(ss!("@"));
            parts.push(decorator.expression.format(p));
            parts.extend(hardline!());
        }
    }

    if let Some(accessibility) = node.format_accessibility(p) {
        parts.push(accessibility);
        parts.push(space!());
    }

    if node.is_declare() {
        parts.push(ss!("declare "));
    }

    if node.is_static() {
        parts.push(ss!("static "));
    }

    if node.is_abstract() {
        parts.push(ss!("abstract "));
    }

    if node.is_override() {
        parts.push(ss!("override "));
    }

    if node.is_readonly() {
        parts.push(ss!("readonly "));
    }

    parts.push(node.format_key(p));

    if node.is_optional() {
        parts.push(ss!("?"));
    } else if node.is_definite() {
        parts.push(ss!("!"));
    }

    if let Some(type_annotation) = node.format_type_annotation(p) {
        parts.push(ss!(": "));
        parts.push(type_annotation);
    }

    let right_expr = node.right_expr();
    let node = match node {
        ClassMemberish::PropertyDefinition(v) => AssignmentLikeNode::PropertyDefinition(v),
        ClassMemberish::AccessorProperty(v) => AssignmentLikeNode::AccessorProperty(v),
    };
    let mut result =
        assignment::print_assignment(p, node, Doc::Array(parts), Doc::Str(" ="), right_expr);

    if p.options.semi {
        let mut parts = p.vec();
        parts.push(result);
        parts.push(ss!(";"));
        result = Doc::Array(parts);
    }
    result
}

fn should_print_semicolon_after_class_property<'a>(
    node: &ClassElement<'a>,
    next_node: Option<&ClassElement<'a>>,
) -> bool {
    if !node.computed() {
        if let ClassElement::PropertyDefinition(property_definition) = node {
            if property_definition.value.is_none() && property_definition.type_annotation.is_none()
            {
                if let Some(key) = property_definition.key.static_name() {
                    if key == "static" || key == "get" || key == "set" {
                        return true;
                    }
                }
            }
        }
    }

    let Some(next_node) = next_node else {
        return false;
    };

    if next_node.r#static() || next_node.accessibility().is_some() {
        return false;
    }

    if !next_node.computed() {
        if let Some(prop_key) = next_node.property_key() {
            if let Some(prop_key) = prop_key.static_name() {
                if prop_key == "in" || prop_key == "instanceof" {
                    return true;
                }
            }
        }
    }

    match next_node {
        ClassElement::PropertyDefinition(property_definition) => property_definition.computed,
        ClassElement::StaticBlock(_) => false,
        ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => true,
        ClassElement::MethodDefinition(method_definition) => {
            let is_async = method_definition.value.r#async;

            if is_async
                || method_definition.kind == MethodDefinitionKind::Get
                || method_definition.kind == MethodDefinitionKind::Set
            {
                return false;
            }

            let is_generator = method_definition.value.generator;

            if method_definition.computed || is_generator {
                return true;
            }

            false
        }
    }
}
