use super::{ESTree, Serializer};

/// Serializer for structs.
///
/// This is returned by [`Serializer::serialize_struct`].
pub struct StructSerializer<'s, S: Serializer> {
    serializer: &'s mut S,
    state: StructState,
}

/// State of [`StructSerializer`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum StructState {
    First,
    Rest,
}

impl<'s, S: Serializer> StructSerializer<'s, S> {
    /// Create new [`StructSerializer`].
    pub(super) fn new(serializer: &'s mut S) -> Self {
        serializer.buffer_mut().push_ascii_byte(b'{');
        Self { serializer, state: StructState::First }
    }

    /// Serialize struct field.
    ///
    /// `key` must not contain any characters which require escaping in JSON.
    pub fn serialize_field<T: ESTree>(&mut self, key: &str, value: &T) {
        if self.state == StructState::Rest {
            self.serializer.buffer_mut().push_ascii_byte(b',');
            self.serializer.enter_element();
        } else {
            self.serializer.enter_sequence();
            self.state = StructState::Rest;
        }

        self.serializer.buffer_mut().push_ascii_byte(b'"');
        self.serializer.buffer_mut().push_str(key);
        self.serializer.buffer_mut().push_str("\":");
        self.serializer.enter_field_value();
        value.serialize(self.serializer);
    }

    /// Finish serializing struct.
    pub fn end(self) {
        if self.state == StructState::Rest {
            self.serializer.exit_sequence();
        }
        self.serializer.buffer_mut().push_ascii_byte(b'}');
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
}
