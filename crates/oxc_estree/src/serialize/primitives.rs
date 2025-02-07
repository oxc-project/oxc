use ryu_js::Buffer;

use super::{ESTree, Serializer};

/// [`ESTree`] implementation for `bool`.
impl ESTree for bool {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        serializer.buffer_mut().push_str(if *self { "true" } else { "false" });
    }
}

/// [`ESTree`] implementation for `f64`.
impl ESTree for f64 {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        let mut buffer = Buffer::new();
        let s = buffer.format(*self).to_string();
        serializer.buffer_mut().push_str(&s);
    }
}

/// [`ESTree`] implementation for `()`.
impl ESTree for () {
    fn serialize<S: Serializer>(&self, serializer: &mut S) {
        serializer.buffer_mut().push_str("null");
    }
}

#[cfg(test)]
mod tests {
    use super::super::CompactSerializer;
    use super::*;

    #[test]
    fn serialize_bool() {
        let cases = [(true, "true"), (false, "false")];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::new();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_f64() {
        let cases = [
            (0.0, "0"),
            (1.0, "1"),
            (123_456.0, "123456"),
            (0.12345, "0.12345"),
            (123.45, "123.45"),
        ];

        for (input, output) in cases {
            let mut serializer = CompactSerializer::new();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_unit() {
        let mut serializer = CompactSerializer::new();
        ().serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(&s, "null");
    }
}
