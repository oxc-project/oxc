use super::{
    Buffer, Config, ESTree, ESTreeSequenceSerializer, ESTreeSerializer, Formatter, Serializer,
    SerializerPrivate,
};

/// Trait for struct serializers.
pub trait StructSerializer {
    type Config: Config;
    type Formatter: Formatter;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &str, value: &T);

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &str, value: &T);

    /// Finish serializing struct.
    fn end(self);
}

/// Serializer for structs.
///
/// This is returned by `ESTreeSerializer::serialize_struct`.
pub struct ESTreeStructSerializer<'s, C: Config, F: Formatter> {
    /// Serializer
    serializer: &'s mut ESTreeSerializer<C, F>,
    /// State of struct.
    /// Starts as `StructState::Empty`, transitions to `StructState::HasFields` on first field.
    state: StructState,
}

impl<'s, C: Config, F: Formatter> ESTreeStructSerializer<'s, C, F> {
    /// Create new [`ESTreeStructSerializer`].
    pub(super) fn new(mut serializer: &'s mut ESTreeSerializer<C, F>) -> Self {
        serializer.buffer_mut().push_ascii_byte(b'{');
        Self { serializer, state: StructState::Empty }
    }
}

impl<C: Config, F: Formatter> StructSerializer for ESTreeStructSerializer<'_, C, F> {
    type Config = C;
    type Formatter = F;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &str, value: &T) {
        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.state == StructState::Empty {
            self.state = StructState::HasFields;
            formatter.before_first_element(buffer);
        } else {
            buffer.push_ascii_byte(b',');
            formatter.before_later_element(buffer);
        }

        buffer.push_ascii_byte(b'"');
        buffer.push_str(key);
        buffer.push_str("\":");
        formatter.before_field_value(buffer);
        value.serialize(&mut *self.serializer);
    }

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &str, value: &T) {
        if C::INCLUDE_TS_FIELDS {
            self.serialize_field(key, value);
        }
    }

    /// Finish serializing struct.
    fn end(self) {
        let mut serializer = self.serializer;
        let (buffer, formatter) = serializer.buffer_and_formatter_mut();
        if self.state == StructState::HasFields {
            formatter.after_last_element(buffer);
        }
        buffer.push_ascii_byte(b'}');
    }
}

/// State of [`StructSerializer`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StructState {
    Empty,
    HasFields,
}

/// Flattening struct serializer.
///
/// The only method it's legal to call on a `FlatStructSerializer` is `serialize_struct`.
///
/// # Example
/// ```
/// struct SomeNode {
///     // We want to flatten `Span` as if its fields were on `SomeNode`
///     span: Span,
///     whatever: u32,
/// }
///
/// struct Span {
///     start: u32,
///     end: u32,
/// }
///
/// impl ESTree for SomeNode {
///     fn serialize<S: Serializer>(&self, serializer: S) {
///         let mut state = serializer.serialize_struct();
///         self.inner.serialize(FlatStructSerializer(&mut state));
///         state.serialize_field("whatever", &self.whatever);
///         state.end();
///     }
/// }
///
/// impl ESTree for Span {
///     fn serialize<S: Serializer>(&self, serializer: S) {
///         let mut state = serializer.serialize_struct();
///         state.serialize_field("start", &self.start);
///         state.serialize_field("end", &self.end);
///         state.end();
///     }
/// }
/// ```
pub struct FlatStructSerializer<'p, P: StructSerializer>(pub &'p mut P);

impl<'p, P: StructSerializer> Serializer for FlatStructSerializer<'p, P> {
    type StructSerializer = Self;
    type SequenceSerializer = ESTreeSequenceSerializer<'p, P::Config, P::Formatter>;

    /// Serialize struct.
    fn serialize_struct(self) -> Self {
        self
    }

    fn serialize_sequence(self) -> ESTreeSequenceSerializer<'p, P::Config, P::Formatter> {
        const {
            panic!("Cannot flatten a sequence into a struct");
        }
    }
}

impl<P: StructSerializer> SerializerPrivate for FlatStructSerializer<'_, P> {
    type Formatter = P::Formatter;

    fn buffer_mut(&mut self) -> &mut Buffer {
        const {
            panic!("Cannot flatten anything but a struct into another struct");
        }
    }

    fn buffer_and_formatter_mut(&mut self) -> (&mut Buffer, &mut P::Formatter) {
        const {
            panic!("Cannot flatten anything but a struct into another struct");
        }
    }
}

impl<P: StructSerializer> StructSerializer for FlatStructSerializer<'_, P> {
    type Config = P::Config;
    type Formatter = P::Formatter;

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_field<T: ESTree + ?Sized>(&mut self, key: &str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_field(key, value);
    }

    /// Serialize struct field which is TypeScript syntax.
    ///
    /// This method behaves differently, depending on the serializer's `Config`:
    /// * `INCLUDE_TS_FIELDS == true`: Behaves same as `serialize_field`
    ///   i.e. the field is included in JSON.
    /// * `INCLUDE_TS_FIELDS == false`: Do nothing.
    ///   i.e. the field is skipped.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    #[inline(always)]
    fn serialize_ts_field<T: ESTree + ?Sized>(&mut self, key: &str, value: &T) {
        // Delegate to parent `StructSerializer`
        self.0.serialize_ts_field(key, value);
    }

    /// Finish serializing struct.
    fn end(self) {
        // No-op - there may be more fields to be added to the struct in the parent
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CompactJSSerializer, CompactTSSerializer, FlatStructSerializer, PrettyJSSerializer,
        PrettyTSSerializer, Serializer,
    };
    use super::*;

    #[test]
    fn serialize_struct() {
        struct Foo<'a> {
            n: f64,
            u: u32,
            bar: Bar<'a>,
            empty: Null,
            hello: Option<&'a str>,
            maybe_bar: Option<Bar<'a>>,
            maybe_not_bar: Option<Bar<'a>>,
        }

        struct Bar<'a> {
            yes: &'a str,
            no: &'a str,
        }

        struct Null;

        impl ESTree for Foo<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("n", &self.n);
                state.serialize_field("u", &self.u);
                state.serialize_field("bar", &self.bar);
                state.serialize_field("empty", &self.empty);
                state.serialize_field("hello", &self.hello);
                state.serialize_field("maybe_bar", &self.maybe_bar);
                state.serialize_field("maybe_not_bar", &self.maybe_not_bar);
                state.end();
            }
        }

        impl ESTree for Bar<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("yes", &self.yes);
                state.serialize_field("no", &self.no);
                state.end();
            }
        }

        impl ESTree for Null {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let state = serializer.serialize_struct();
                state.end();
            }
        }

        let foo = Foo {
            n: 123.0,
            u: 12345,
            bar: Bar { yes: "yup", no: "nope" },
            empty: Null,
            hello: Some("hi!"),
            maybe_bar: Some(Bar { yes: "hell yeah!", no: "not a chance in a million, mate" }),
            maybe_not_bar: None,
        };

        let mut serializer = CompactTSSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"n":123,"u":12345,"bar":{"yes":"yup","no":"nope"},"empty":{},"hello":"hi!","maybe_bar":{"yes":"hell yeah!","no":"not a chance in a million, mate"},"maybe_not_bar":null}"#
        );

        let mut serializer = PrettyTSSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "n": 123,
  "u": 12345,
  "bar": {
    "yes": "yup",
    "no": "nope"
  },
  "empty": {},
  "hello": "hi!",
  "maybe_bar": {
    "yes": "hell yeah!",
    "no": "not a chance in a million, mate"
  },
  "maybe_not_bar": null
}"#
        );
    }

    #[test]
    fn serialize_flattened_struct() {
        struct Outer {
            outer1: &'static str,
            inner: Inner,
            outer2: &'static str,
        }

        struct Inner {
            inner1: &'static str,
            innermost: Innermost,
            inner2: &'static str,
        }

        struct Innermost {
            innermost1: &'static str,
            innermost2: &'static str,
        }

        impl ESTree for Outer {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("outer1", &self.outer1);
                self.inner.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("outer2", &self.outer2);
                state.end();
            }
        }

        impl ESTree for Inner {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("inner1", &self.inner1);
                self.innermost.serialize(FlatStructSerializer(&mut state));
                state.serialize_field("inner2", &self.inner2);
                state.end();
            }
        }

        impl ESTree for Innermost {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("innermost1", &self.innermost1);
                state.serialize_field("innermost2", &self.innermost2);
                state.end();
            }
        }

        let outer = Outer {
            outer1: "out1",
            inner: Inner {
                inner1: "in1",
                innermost: Innermost { innermost1: "inin1", innermost2: "inin2" },
                inner2: "in2",
            },
            outer2: "out2",
        };

        let mut serializer = CompactTSSerializer::new();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"outer1":"out1","inner1":"in1","innermost1":"inin1","innermost2":"inin2","inner2":"in2","outer2":"out2"}"#
        );

        let mut serializer = PrettyTSSerializer::new();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "outer1": "out1",
  "inner1": "in1",
  "innermost1": "inin1",
  "innermost2": "inin2",
  "inner2": "in2",
  "outer2": "out2"
}"#
        );
    }

    #[test]
    fn serialize_struct_with_or_without_ts() {
        struct Foo {
            js: u32,
            ts: u32,
            more_ts: u32,
            more_js: u32,
        }

        impl ESTree for Foo {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("js", &self.js);
                state.serialize_ts_field("ts", &self.ts);
                state.serialize_ts_field("moreTs", &self.more_ts);
                state.serialize_field("moreJs", &self.more_js);
                state.end();
            }
        }

        let foo = Foo { js: 1, ts: 2, more_ts: 3, more_js: 4 };

        let mut serializer = CompactTSSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"js":1,"ts":2,"moreTs":3,"moreJs":4}"#);

        let mut serializer = PrettyTSSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "js": 1,
  "ts": 2,
  "moreTs": 3,
  "moreJs": 4
}"#
        );

        let mut serializer = CompactJSSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"js":1,"moreJs":4}"#);

        let mut serializer = PrettyJSSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "js": 1,
  "moreJs": 4
}"#
        );
    }
}
