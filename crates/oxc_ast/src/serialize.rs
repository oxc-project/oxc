use serde::{
    ser::{SerializeSeq, SerializeStruct, Serializer},
    Serialize,
};

use crate::ast::{
    ArrayAssignmentTarget, ArrayPattern, AssignmentTargetMaybeDefault, AssignmentTargetProperty,
    AssignmentTargetRest, BindingIdentifier, BindingPattern, BindingPatternKind, BindingProperty,
    BindingRestElement, FormalParameter, FormalParameterKind, FormalParameters, IdentifierName,
    IdentifierReference, LabelIdentifier, ObjectAssignmentTarget, ObjectPattern, Program,
    RegExpFlags, TSTypeAnnotation,
};
use oxc_allocator::{Box, Vec};
use oxc_span::{Atom, Span};

pub struct EcmaFormatter;

/// Serialize f64 with `ryu_js`
impl serde_json::ser::Formatter for EcmaFormatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        let mut buffer = ryu_js::Buffer::new();
        let s = buffer.format(value);
        writer.write_all(s.as_bytes())
    }
}

impl<'a> Program<'a> {
    /// # Panics
    pub fn to_json(&self) -> String {
        let buf = std::vec::Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(buf, crate::serialize::EcmaFormatter);
        self.serialize(&mut ser).unwrap();
        String::from_utf8(ser.into_inner()).unwrap()
    }
}

impl Serialize for RegExpFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Serialize `BindingIdentifier`, `IdentifierReference`, `IdentifierName` and `LabelIdentifier`
/// to be estree compatible with the `type` set to "Identifier".
fn serialize_identifier<S: Serializer>(
    serializer: S,
    struct_name: &'static str,
    span: Span,
    name: &Atom,
) -> Result<S::Ok, S::Error> {
    let mut state = serializer.serialize_struct(struct_name, 4)?;
    state.serialize_field("type", "Identifier")?;
    state.serialize_field("start", &span.start)?;
    state.serialize_field("end", &span.end)?;
    state.serialize_field("name", name)?;
    state.end()
}

impl<'a> Serialize for BindingIdentifier<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_identifier(serializer, "BindingIdentifier", self.span, &self.name)
    }
}

impl<'a> Serialize for IdentifierReference<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_identifier(serializer, "IdentifierReference", self.span, &self.name)
    }
}

impl<'a> Serialize for IdentifierName<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_identifier(serializer, "IdentifierName", self.span, &self.name)
    }
}

impl<'a> Serialize for LabelIdentifier<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_identifier(serializer, "LabelIdentifier", self.span, &self.name)
    }
}

/// Serialize `ArrayAssignmentTarget`, `ObjectAssignmentTarget`, `ObjectPattern`, `ArrayPattern`
/// to be estree compatible, with `elements`/`properties` and `rest` fields combined.

impl<'a> Serialize for ArrayAssignmentTarget<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerArrayAssignmentTarget {
            span: self.span,
            elements: ElementsAndRest::new(&self.elements, &self.rest),
            trailing_comma: self.trailing_comma,
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "ArrayAssignmentTarget", rename_all = "camelCase")]
struct SerArrayAssignmentTarget<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    elements:
        ElementsAndRest<'a, 'b, Option<AssignmentTargetMaybeDefault<'a>>, AssignmentTargetRest<'a>>,
    trailing_comma: Option<Span>,
}

impl<'a> Serialize for ObjectAssignmentTarget<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerObjectAssignmentTarget {
            span: self.span,
            properties: ElementsAndRest::new(&self.properties, &self.rest),
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "ObjectAssignmentTarget")]
struct SerObjectAssignmentTarget<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    properties: ElementsAndRest<'a, 'b, AssignmentTargetProperty<'a>, AssignmentTargetRest<'a>>,
}

impl<'a> Serialize for ObjectPattern<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerObjectPattern {
            span: self.span,
            properties: ElementsAndRest::new(&self.properties, &self.rest),
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "ObjectPattern")]
struct SerObjectPattern<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    properties: ElementsAndRest<'a, 'b, BindingProperty<'a>, Box<'a, BindingRestElement<'a>>>,
}

impl<'a> Serialize for ArrayPattern<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerArrayPattern {
            span: self.span,
            elements: ElementsAndRest::new(&self.elements, &self.rest),
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "ArrayPattern")]
struct SerArrayPattern<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    elements: ElementsAndRest<'a, 'b, Option<BindingPattern<'a>>, Box<'a, BindingRestElement<'a>>>,
}

/// Serialize `FormalParameters`, to be estree compatible, with `items` and `rest` fields combined
/// and `argument` field flattened.
impl<'a> Serialize for FormalParameters<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted_rest = self.rest.as_ref().map(|rest| SerFormalParameterRest {
            span: rest.span,
            argument: &rest.argument.kind,
            type_annotation: &rest.argument.type_annotation,
            optional: rest.argument.optional,
        });
        let converted = SerFormalParameters {
            span: self.span,
            kind: self.kind,
            items: ElementsAndRest::new(&self.items, &converted_rest),
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "FormalParameters")]
struct SerFormalParameters<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    kind: FormalParameterKind,
    items: ElementsAndRest<'a, 'b, FormalParameter<'a>, SerFormalParameterRest<'a, 'b>>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "RestElement", rename_all = "camelCase")]
struct SerFormalParameterRest<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    argument: &'b BindingPatternKind<'a>,
    type_annotation: &'b Option<Box<'a, TSTypeAnnotation<'a>>>,
    optional: bool,
}

pub struct ElementsAndRest<'a, 'b, E, R> {
    elements: &'b Vec<'a, E>,
    rest: &'b Option<R>,
}

impl<'a, 'b, E, R> ElementsAndRest<'a, 'b, E, R> {
    pub fn new(elements: &'b Vec<'a, E>, rest: &'b Option<R>) -> Self {
        Self { elements, rest }
    }
}

impl<'a, 'b, E: Serialize, R: Serialize> Serialize for ElementsAndRest<'a, 'b, E, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(rest) = self.rest {
            let mut seq = serializer.serialize_seq(Some(self.elements.len() + 1))?;
            for element in self.elements {
                seq.serialize_element(element)?;
            }
            seq.serialize_element(rest)?;
            seq.end()
        } else {
            self.elements.serialize(serializer)
        }
    }
}
