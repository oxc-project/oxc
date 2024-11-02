#![allow(dead_code)]

use serde::{ser::SerializeMap, Serialize, Serializer};

use std::io::Write;

use serde_json::{
    error::{Error as JsonError, Result as JsonResult},
    ser::{Compound, Formatter, Serializer as JsonSerializer},
};

struct WrappedSerializer<W: Write, F: Formatter> {
    inner: JsonSerializer<W, F>,
    include_ts_fields: bool,
}

trait WrappedSerializerTrait: Serializer {
    fn include_ts_fields(self) -> bool;
}

impl<'s, W: Write, F: Formatter> WrappedSerializerTrait for &'s mut WrappedSerializer<W, F> {
    fn include_ts_fields(self) -> bool {
        self.include_ts_fields
    }
}

impl<'a, W: Write, F: Formatter> Serializer for &'a mut WrappedSerializer<W, F> {
    type Ok = ();
    type Error = JsonError;

    type SerializeSeq = Compound<'a, W, F>;
    type SerializeTuple = Compound<'a, W, F>;
    type SerializeTupleStruct = Compound<'a, W, F>;
    type SerializeTupleVariant = Compound<'a, W, F>;
    type SerializeMap = Compound<'a, W, F>;
    type SerializeStruct = Compound<'a, W, F>;
    type SerializeStructVariant = Compound<'a, W, F>;

    #[inline]
    fn serialize_bool(self, value: bool) -> JsonResult<()> {
        self.inner.serialize_bool(value)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> JsonResult<()> {
        self.inner.serialize_i8(value)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> JsonResult<()> {
        self.inner.serialize_i16(value)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> JsonResult<()> {
        self.inner.serialize_i32(value)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> JsonResult<()> {
        self.inner.serialize_i64(value)
    }

    #[inline]
    fn serialize_i128(self, value: i128) -> JsonResult<()> {
        self.inner.serialize_i128(value)
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> JsonResult<()> {
        self.inner.serialize_u8(value)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> JsonResult<()> {
        self.inner.serialize_u16(value)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> JsonResult<()> {
        self.inner.serialize_u32(value)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> JsonResult<()> {
        self.inner.serialize_u64(value)
    }

    fn serialize_u128(self, value: u128) -> JsonResult<()> {
        self.inner.serialize_u128(value)
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> JsonResult<()> {
        self.inner.serialize_f32(value)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> JsonResult<()> {
        self.inner.serialize_f64(value)
    }

    #[inline]
    fn serialize_char(self, value: char) -> JsonResult<()> {
        self.inner.serialize_char(value)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> JsonResult<()> {
        self.inner.serialize_str(value)
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> JsonResult<()> {
        self.inner.serialize_bytes(value)
    }

    #[inline]
    fn serialize_unit(self) -> JsonResult<()> {
        self.inner.serialize_unit()
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> JsonResult<()> {
        self.inner.serialize_unit_struct(name)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> JsonResult<()> {
        self.inner.serialize_unit_variant(name, variant_index, variant)
    }

    /// Serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> JsonResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.inner.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> JsonResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.inner.serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline]
    fn serialize_none(self) -> JsonResult<()> {
        self.inner.serialize_none()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> JsonResult<()>
    where
        T: ?Sized + Serialize,
    {
        self.inner.serialize_some(value)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> JsonResult<Self::SerializeSeq> {
        self.inner.serialize_seq(len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> JsonResult<Self::SerializeTuple> {
        self.inner.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> JsonResult<Self::SerializeTupleStruct> {
        self.inner.serialize_tuple_struct(name, len)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> JsonResult<Self::SerializeTupleVariant> {
        self.inner.serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> JsonResult<Self::SerializeMap> {
        self.inner.serialize_map(len)
    }

    #[inline]
    fn serialize_struct(self, name: &'static str, len: usize) -> JsonResult<Self::SerializeStruct> {
        self.inner.serialize_struct(name, len)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> JsonResult<Self::SerializeStructVariant> {
        self.inner.serialize_struct_variant(name, variant_index, variant, len)
    }

    fn collect_str<T>(self, value: &T) -> JsonResult<()>
    where
        T: ?Sized + std::fmt::Display,
    {
        self.inner.collect_str(value)
    }
}

trait WrappedSerialize {
    fn wrapped_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: WrappedSerializerTrait;
}

struct Foo {
    span: Span,
    big: u64,
    small: u8,
    bar: Bar,
}

struct Span {
    start: u32,
    end: u32,
}

struct Bar {
    something: u32,
}

impl WrappedSerialize for Foo {
    fn wrapped_serialize<S: WrappedSerializerTrait>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Foo")?;
        self.span.wrapped_serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        map.serialize_entry("big", &self.big)?;
        map.serialize_entry("small", &self.small)?;
        map.serialize_entry("bar", &self.bar)?;
        map.end()
    }
}

impl WrappedSerialize for Span {
    fn wrapped_serialize<S: WrappedSerializerTrait>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("start", &self.start)?;
        map.serialize_entry("end", &self.end)?;
        map.end()
    }
}

impl WrappedSerialize for Bar {
    fn wrapped_serialize<S: WrappedSerializerTrait>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", "Bar")?;
        map.serialize_entry("something", &self.something)?;
        map.end()
    }
}
