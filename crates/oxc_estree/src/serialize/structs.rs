use super::{Buffer, ESTree, Formatter, SequenceSerializer, Serializer, SerializerWithFormatter};

/// Serializer for structs.
///
/// This is returned by [`Serializer::serialize_struct`].
pub struct StructSerializer<'s, S: Serializer> {
    serializer: &'s mut S,
    state: S::StructState,
}

pub(super) trait StructStateTrait: Copy {
    fn new() -> Self;

    fn get(self) -> StructState;

    fn set(&mut self, state: StructState);
}

/// State of [`StructSerializer`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StructState {
    First,
    Rest,
}

impl StructStateTrait for StructState {
    fn new() -> Self {
        StructState::First
    }

    fn get(self) -> StructState {
        self
    }

    fn set(&mut self, state: StructState) {
        *self = state;
    }
}

impl StructStateTrait for () {
    fn new() -> Self {}

    fn get(self) -> StructState {
        unreachable!()
    }

    fn set(&mut self, _state: StructState) {
        unreachable!()
    }
}

impl<'s, S: Serializer> StructSerializer<'s, S> {
    /// Create new [`StructSerializer`].
    pub(super) fn new(serializer: &'s mut S) -> Self {
        if S::IS_ROOT {
            serializer.buffer_mut().push_ascii_byte(b'{');
        }
        Self { serializer, state: S::StructState::new() }
    }

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    pub fn serialize_field<T: ESTree>(&mut self, key: &str, value: &T) {
        let state = if S::IS_ROOT {
            let state = self.state.get();
            if state == StructState::First {
                self.state.set(StructState::Rest);
            }
            state
        } else {
            let state = self.serializer.get_struct_state();
            if state == StructState::First {
                self.serializer.set_struct_state(StructState::Rest);
            }
            state
        };

        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if state == StructState::Rest {
            buffer.push_ascii_byte(b',');
            formatter.enter_field(buffer);
        } else {
            formatter.enter_sequence(buffer);
        }

        buffer.push_ascii_byte(b'"');
        buffer.push_str(key);
        buffer.push_str("\":");
        formatter.enter_field_value(buffer);
        value.serialize(self.serializer);
    }

    /// Finish serializing struct.
    pub fn end(self) {
        if S::IS_ROOT {
            let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
            if self.state.get() == StructState::Rest {
                formatter.exit_sequence(buffer);
            }
            buffer.push_ascii_byte(b'}');
        }
    }
}

impl<S: Serializer> Serializer for StructSerializer<'_, S> {
    /// Serialize struct.
    fn serialize_struct(&mut self) -> StructSerializer<'_, Self> {
        StructSerializer::new(self)
    }

    /// Serialize sequence.
    fn serialize_sequence(&mut self) -> SequenceSerializer<'_, Self> {
        const { panic!() }
    }
}

impl<S: Serializer> SerializerWithFormatter for StructSerializer<'_, S> {
    type Formatter = S::Formatter;
    type StructState = ();

    /// This is not the root serializer
    const IS_ROOT: bool = false;

    fn get_struct_state(&self) -> StructState {
        if S::IS_ROOT {
            self.state.get()
        } else {
            self.serializer.get_struct_state()
        }
    }

    fn set_struct_state(&mut self, state: StructState) {
        if S::IS_ROOT {
            self.state.set(state);
        } else {
            self.serializer.set_struct_state(state);
        }
    }

    /// Get mutable reference to buffer.
    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut Buffer {
        self.serializer.buffer_mut()
    }

    /// Get mutable reference to buffer and formatter.
    #[inline(always)]
    fn buffer_and_formatter_mut(&mut self) -> (&mut Buffer, &mut Self::Formatter) {
        self.serializer.buffer_and_formatter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompactSerializer, PrettySerializer};
    use super::*;

    #[test]
    fn serialize_structs() {
        struct Foo<'a> {
            n: f64,
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
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("n", &self.n);
                state.serialize_field("bar", &self.bar);
                state.serialize_field("empty", &self.empty);
                state.serialize_field("hello", &self.hello);
                state.serialize_field("maybe_bar", &self.maybe_bar);
                state.serialize_field("maybe_not_bar", &self.maybe_not_bar);
                state.end();
            }
        }

        impl ESTree for Bar<'_> {
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("yes", &self.yes);
                state.serialize_field("no", &self.no);
                state.end();
            }
        }

        impl ESTree for Null {
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
                let state = serializer.serialize_struct();
                state.end();
            }
        }

        let foo = Foo {
            n: 123.0,
            bar: Bar { yes: "yup", no: "nope" },
            empty: Null,
            hello: Some("hi!"),
            maybe_bar: Some(Bar { yes: "hell yeah!", no: "not a chance in a million, mate" }),
            maybe_not_bar: None,
        };

        let mut serializer = CompactSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"n":123,"bar":{"yes":"yup","no":"nope"},"empty":{},"hello":"hi!","maybe_bar":{"yes":"hell yeah!","no":"not a chance in a million, mate"},"maybe_not_bar":null}"#
        );

        let mut serializer = PrettySerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "n": 123,
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
    fn serialize_flattened_structs() {
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
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("outer1", &self.outer1);
                self.inner.serialize(&mut state);
                state.serialize_field("outer2", &self.outer2);
                state.end();
            }
        }

        impl ESTree for Inner {
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("inner1", &self.inner1);
                self.innermost.serialize(&mut state);
                state.serialize_field("inner2", &self.inner2);
                state.end();
            }
        }

        impl ESTree for Innermost {
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
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

        let mut serializer = CompactSerializer::new();
        outer.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{"outer1":"out1","inner1":"in1","innermost1":"inin1","innermost2":"inin2","inner2":"in2","outer2":"out2"}"#
        );
    }
}
