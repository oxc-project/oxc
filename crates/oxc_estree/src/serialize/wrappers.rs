use super::{ESTree, Serializer};

/// [`ESTree`] implementation for `Option`.
impl<T: ESTree> ESTree for Option<T> {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        if let Some(it) = self {
            it.serialize(serializer);
        } else {
            serializer.buffer_mut().push_str("null");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::CompactSerializer;
    use super::*;

    #[test]
    fn serialize_options() {
        let cases = [(None, "null"), (Some(123.0f64), "123")];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::new();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }
}
