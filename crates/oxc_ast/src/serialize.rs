use oxc_allocator::Vec;
use serde::{
    ser::{SerializeSeq, SerializeStruct, Serializer},
    Serialize,
};

use crate::{
    ast::{
        ArrowExpression, Directive, FormalParameters, FunctionBody, MemberExpression, Program,
        Statement,
    },
    ModuleKind,
};

#[cfg(feature = "serde_json")]
pub struct EcmaFormatter;

/// Serialize f64 with `ryu_js`
#[cfg(feature = "serde_json")]
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
    #[must_use]
    #[cfg(feature = "serde_json")]
    pub fn to_json(&self) -> String {
        let buf = std::vec::Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(buf, crate::serialize::EcmaFormatter);
        self.serialize(&mut ser).unwrap();
        String::from_utf8(ser.into_inner()).unwrap()
    }
}

impl<'a> Serialize for Program<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Program", 5)?;
        state.serialize_field("type", &"Program")?;
        state.serialize_field("start", &self.span.start)?;
        state.serialize_field("end", &self.span.end)?;
        let source_type = match self.source_type.module_kind() {
            ModuleKind::Script => "script",
            ModuleKind::Module => "module",
        };
        state.serialize_field("sourceType", &source_type)?;
        let body = BlockWrapper { directives: &self.directives, body: &self.body };
        state.serialize_field("body", &body)?;

        state.end()
    }
}

pub fn serialize_bigint<T, S>(value: &T, s: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::Display,
    S: serde::Serializer,
{
    s.collect_str(&format_args!("{value}n"))
}

/// Helper struct for serializing `Program` and `FunctionBody`
#[derive(Debug, PartialEq)]
pub struct BlockWrapper<'a, 'b> {
    pub directives: &'b Vec<'a, Directive<'a>>,
    pub body: &'b Vec<'a, Statement<'a>>,
}

impl<'a, 'b> Serialize for BlockWrapper<'a, 'b> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.directives.len() + self.body.len()))?;
        for e in self.directives {
            seq.serialize_element(e)?;
        }
        for e in self.body {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl<'a> Serialize for MemberExpression<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MemberExpression", 7)?;
        state.serialize_field("type", &"MemberExpression")?;
        match &self {
            MemberExpression::ComputedMemberExpression(expr) => {
                state.serialize_field("start", &expr.span.start)?;
                state.serialize_field("end", &expr.span.end)?;
                state.serialize_field("object", &expr.object)?;
                state.serialize_field("property", &expr.expression)?;
                state.serialize_field("computed", &true)?;
                state.serialize_field("optional", &expr.optional)?;
            }
            MemberExpression::StaticMemberExpression(expr) => {
                state.serialize_field("start", &expr.span.start)?;
                state.serialize_field("end", &expr.span.end)?;
                state.serialize_field("object", &expr.object)?;
                state.serialize_field("property", &expr.property)?;
                state.serialize_field("computed", &false)?;
                state.serialize_field("optional", &expr.optional)?;
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                state.serialize_field("start", &expr.span.start)?;
                state.serialize_field("end", &expr.span.end)?;
                state.serialize_field("object", &expr.object)?;
                state.serialize_field("property", &expr.field)?;
                state.serialize_field("computed", &false)?;
                state.serialize_field("optional", &expr.optional)?;
            }
        }
        state.end()
    }
}

impl<'a> Serialize for FormalParameters<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.items.len()))?;

        for e in &self.items {
            seq.serialize_element(e)?;
        }

        seq.end()
    }
}

impl<'a> Serialize for FunctionBody<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FunctionBody", 4)?;
        state.serialize_field("type", &"BlockStatement")?;
        state.serialize_field("start", &self.span.start)?;
        state.serialize_field("end", &self.span.end)?;
        let body = BlockWrapper { directives: &self.directives, body: &self.statements };
        state.serialize_field("body", &body)?;
        state.end()
    }
}

impl<'a> Serialize for ArrowExpression<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 9;
        if self.type_parameters.is_some() {
            len += 1;
        }
        if self.return_type.is_some() {
            len += 1;
        }
        let mut state = serializer.serialize_struct("ArrowExpression", len)?;
        state.serialize_field("type", &"ArrowFunctionExpression")?;
        state.serialize_field("start", &self.span.start)?;
        state.serialize_field("end", &self.span.end)?;
        state.serialize_field("id", &None as &Option<()>)?; // Always none in oxc_ast
        state.serialize_field("expression", &self.expression)?;
        state.serialize_field("generator", &self.generator)?;
        state.serialize_field("async", &self.r#async)?;
        state.serialize_field("params", &self.params)?;
        state.serialize_field("body", &self.body)?;
        if self.type_parameters.is_some() {
            state.serialize_field("typeParameters", &self.type_parameters)?;
        }
        if self.return_type.is_some() {
            state.serialize_field("returnType", &self.return_type)?;
        }
        state.end()
    }
}
