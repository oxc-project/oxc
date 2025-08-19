use super::{
    Config, ESTree, ESTreeSerializer, Formatter, Serializer, SerializerPrivate, TracePathPart,
};

/// Trait for sequence serializers.
pub trait SequenceSerializer {
    /// Serialize sequence entry.
    fn serialize_element<T: ESTree + ?Sized>(&mut self, value: &T);

    /// Finish serializing sequence.
    fn end(self);
}

/// Serializer for sequences.
///
/// This is returned by `ESTreeSerializer::serialize_sequence`.
pub struct ESTreeSequenceSerializer<'s, C: Config, F: Formatter> {
    /// Serializer
    serializer: &'s mut ESTreeSerializer<C, F>,
    /// Length of sequence
    len: usize,
}

impl<'s, C: Config, F: Formatter> ESTreeSequenceSerializer<'s, C, F> {
    /// Create new [`ESTreeSequenceSerializer`].
    pub(super) fn new(mut serializer: &'s mut ESTreeSerializer<C, F>) -> Self {
        // Push item to `trace_path`. It will be replaced with a `TracePathPart::Index`
        // when serializing each item in the sequence, and popped off again in `end` method.
        if C::FIXES {
            serializer.trace_path.push(TracePathPart::DUMMY);
        }

        serializer.buffer_mut().print_ascii_byte(b'[');

        Self { serializer, len: 0 }
    }
}

impl<C: Config, F: Formatter> SequenceSerializer for ESTreeSequenceSerializer<'_, C, F> {
    /// Serialize sequence entry.
    fn serialize_element<T: ESTree + ?Sized>(&mut self, value: &T) {
        // Update last item in trace path to current sequence index
        if C::FIXES {
            *self.serializer.trace_path.last_mut() = TracePathPart::Index(self.len);
        }

        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.len == 0 {
            formatter.before_first_element(buffer);
        } else {
            buffer.print_ascii_byte(b',');
            formatter.before_later_element(buffer);
        }

        value.serialize(&mut *self.serializer);
        self.len += 1;
    }

    /// Finish serializing sequence.
    fn end(mut self) {
        // Pop entry for this sequence from `trace_path`
        if C::FIXES {
            // SAFETY: `trace_path` is pushed to in `new`, which is only way to create an `ESTreeSequenceSerializer`.
            // This method consumes the `ESTreeSequenceSerializer`, so this method can't be called more
            // times than `new`. So there must be an item to pop.
            unsafe { self.serializer.trace_path.pop_unchecked() };
        }

        let (buffer, formatter) = self.serializer.buffer_and_formatter_mut();
        if self.len > 0 {
            formatter.after_last_element(buffer);
        }
        buffer.print_ascii_byte(b']');
    }
}

/// [`ESTree`] implementation for slices.
impl<T: ESTree> ESTree for &[T] {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        for element in *self {
            seq.serialize_element(element);
        }
        seq.end();
    }
}

/// [`ESTree`] implementation for arrays.
impl<T: ESTree, const N: usize> ESTree for [T; N] {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        for element in self {
            seq.serialize_element(element);
        }
        seq.end();
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompactTSSerializer, PrettyTSSerializer, StructSerializer};
    use super::*;

    #[test]
    fn serialize_sequence() {
        struct Foo<'a> {
            none: &'a [&'a str],
            one: &'a [&'a str],
            two: [&'a str; 2],
        }

        impl ESTree for Foo<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) {
                let mut state = serializer.serialize_struct();
                state.serialize_field("none", &self.none);
                state.serialize_field("one", &self.one);
                state.serialize_field("two", &self.two);
                state.end();
            }
        }

        let foo = Foo { none: &[], one: &["one"], two: ["two one", "two two"] };

        let mut serializer = CompactTSSerializer::default();
        foo.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, r#"{"none":[],"one":["one"],"two":["two one","two two"]}"#);

        let mut serializer = PrettyTSSerializer::default();
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
