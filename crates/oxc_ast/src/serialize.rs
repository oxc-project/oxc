use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};

use crate::ast::{
    BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier, Program, RegExpFlags,
};
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
