use super::{ESTree, Formatter, Serializer};

/// Serializer for sequences.
///
/// This is returned by [`Serializer::serialize_sequence`].
pub struct SequenceSerializer<'s, S: Serializer> {
    serializer: &'s mut S,
    state: SequenceState,
}

/// State of [`SequenceSerializer`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum SequenceState {
    First,
    Rest,
}

impl<'s, S: Serializer> SequenceSerializer<'s, S> {
    /// Create new [`StructSerializer`].
    pub(super) fn new(serializer: &'s mut S) -> Self {
        serializer.buffer_mut().push_ascii_byte(b'[');
        Self { serializer, state: SequenceState::First }
    }

    /// Serialize sequence entry.
    pub fn serialize_element<T: ESTree>(&mut self, value: &T) {
        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.state == SequenceState::Rest {
            buffer.push_ascii_byte(b',');
            formatter.enter_field(buffer);
        } else {
            formatter.enter_sequence(buffer);
            self.state = SequenceState::Rest;
        }

        value.serialize(self.serializer);
    }

    /// Finish serializing sequence.
    pub fn end(self) {
        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.state == SequenceState::Rest {
            formatter.exit_sequence(buffer);
        }
        buffer.push_ascii_byte(b']');
    }
}

/// [`ESTree`] implementation for slices.
impl<T: ESTree> ESTree for &[T] {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        let mut seq = serializer.serialize_sequence();
        for element in *self {
            seq.serialize_element(element);
        }
        seq.end();
    }
}

/// [`ESTree`] implementation for arrays.
impl<T: ESTree, const N: usize> ESTree for [T; N] {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        let mut seq = serializer.serialize_sequence();
        for element in self {
            seq.serialize_element(element);
        }
        seq.end();
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompactSerializer, PrettySerializer};
    use super::*;

    #[test]
    fn serialize_sequences() {
        struct Foo<'a> {
            none: &'a [&'a str],
            one: &'a [&'a str],
            two: [&'a str; 2],
        }

        impl ESTree for Foo<'_> {
            fn serialize<S: Serializer>(&self, serializer: &mut S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("none", &self.none);
                state.serialize_field("one", &self.one);
                state.serialize_field("two", &self.two);
                state.end();
            }
        }

        let foo = Foo { none: &[], one: &["one"], two: ["two one", "two two"] };

        let mut serializer = CompactSerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"none":[],"one":["one"],"two":["two one","two two"]}"#);

        let mut serializer = PrettySerializer::new();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(
            &s,
            r#"{
  "none": [],
  "one": [
    "one"
  ],
  "two": [
    "two one",
    "two two"
  ]
}"#
        );
    }
}
