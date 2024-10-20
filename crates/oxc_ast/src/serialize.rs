use oxc_allocator::Box;
use oxc_span::Span;
use serde::{
    ser::{SerializeSeq, Serializer},
    Serialize,
};

use crate::ast::{
    ArrayAssignmentTarget, ArrayPattern, AssignmentTargetMaybeDefault, AssignmentTargetProperty,
    AssignmentTargetRest, BindingPattern, BindingPatternKind, BindingProperty, BindingRestElement,
    Directive, Elision, FormalParameter, FormalParameterKind, FormalParameters, JSXElementName,
    JSXIdentifier, JSXMemberExpressionObject, ObjectAssignmentTarget, ObjectPattern, Program,
    RegExpFlags, Statement, StringLiteral, TSModuleBlock, TSTypeAnnotation,
};

pub struct EcmaFormatter;

/// Serialize f64 with `ryu_js`
impl serde_json::ser::Formatter for EcmaFormatter {
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> std::io::Result<()>
    where
        W: ?Sized + std::io::Write,
    {
        use oxc_syntax::number::ToJsString;
        writer.write_all(value.to_js_string().as_bytes())
    }
}

impl<'a> Program<'a> {
    /// # Panics
    pub fn to_json(&self) -> String {
        let ser = self.serializer();
        String::from_utf8(ser.into_inner()).unwrap()
    }

    /// # Panics
    pub fn serializer(&self) -> serde_json::Serializer<std::vec::Vec<u8>, EcmaFormatter> {
        let buf = std::vec::Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(buf, EcmaFormatter);
        self.serialize(&mut ser).unwrap();
        ser
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

/// Serialize `ArrayExpressionElement::Elision` variant as `null` in JSON
impl Serialize for Elision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

/// Serialize `ArrayAssignmentTarget`, `ObjectAssignmentTarget`, `ObjectPattern`, `ArrayPattern`
/// to be estree compatible, with `elements`/`properties` and `rest` fields combined.

impl<'a> Serialize for ArrayAssignmentTarget<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerArrayAssignmentTarget {
            span: self.span,
            elements: ElementsAndRest::new(&self.elements, &self.rest),
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
        ElementsAndRest<'b, Option<AssignmentTargetMaybeDefault<'a>>, AssignmentTargetRest<'a>>,
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
    properties: ElementsAndRest<'b, AssignmentTargetProperty<'a>, AssignmentTargetRest<'a>>,
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
    properties: ElementsAndRest<'b, BindingProperty<'a>, Box<'a, BindingRestElement<'a>>>,
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
    elements: ElementsAndRest<'b, Option<BindingPattern<'a>>, Box<'a, BindingRestElement<'a>>>,
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

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type FormalParameterRest = ({
    type: "RestElement",
    argument: BindingPatternKind,
    typeAnnotation: TSTypeAnnotation | null,
    optional: boolean,
}) & Span;
"#;

#[derive(Serialize)]
#[serde(tag = "type", rename = "FormalParameters")]
struct SerFormalParameters<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    kind: FormalParameterKind,
    items: ElementsAndRest<'b, FormalParameter<'a>, SerFormalParameterRest<'a, 'b>>,
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

pub struct ElementsAndRest<'b, E, R> {
    elements: &'b [E],
    rest: &'b Option<R>,
}

impl<'b, E, R> ElementsAndRest<'b, E, R> {
    pub fn new(elements: &'b [E], rest: &'b Option<R>) -> Self {
        Self { elements, rest }
    }
}

impl<'b, E: Serialize, R: Serialize> Serialize for ElementsAndRest<'b, E, R> {
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

/// Serialize `TSModuleBlock` to be ESTree compatible, with `body` and `directives` fields combined,
/// and directives output as `StringLiteral` expression statements
impl<'a> Serialize for TSModuleBlock<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let converted = SerTSModuleBlock {
            span: self.span,
            body: DirectivesAndStatements { directives: &self.directives, body: &self.body },
        };
        converted.serialize(serializer)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "TSModuleBlock")]
struct SerTSModuleBlock<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    body: DirectivesAndStatements<'a, 'b>,
}

struct DirectivesAndStatements<'a, 'b> {
    directives: &'b [Directive<'a>],
    body: &'b [Statement<'a>],
}

impl<'a, 'b> Serialize for DirectivesAndStatements<'a, 'b> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.directives.len() + self.body.len()))?;
        for directive in self.directives {
            seq.serialize_element(&DirectiveAsStatement {
                span: directive.span,
                expression: &directive.expression,
            })?;
        }
        for stmt in self.body {
            seq.serialize_element(stmt)?;
        }
        seq.end()
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "ExpressionStatement")]
struct DirectiveAsStatement<'a, 'b> {
    #[serde(flatten)]
    span: Span,
    expression: &'b StringLiteral<'a>,
}

impl<'a> Serialize for JSXElementName<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Identifier(ident) => ident.serialize(serializer),
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name.clone() }.serialize(serializer)
            }
            Self::NamespacedName(name) => name.serialize(serializer),
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}

impl<'a> Serialize for JSXMemberExpressionObject<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::IdentifierReference(ident) => {
                JSXIdentifier { span: ident.span, name: ident.name.clone() }.serialize(serializer)
            }
            Self::MemberExpression(expr) => expr.serialize(serializer),
            Self::ThisExpression(expr) => {
                JSXIdentifier { span: expr.span, name: "this".into() }.serialize(serializer)
            }
        }
    }
}
