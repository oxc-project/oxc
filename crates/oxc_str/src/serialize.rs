use oxc_estree::{ESTree, Serializer};

use crate::JSStr;

impl ESTree for JSStr<'_> {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(value) = self.as_str() {
            value.serialize(serializer);
        } else {
            self.serialize_lone_surrogates(serializer);
        }
    }
}

impl JSStr<'_> {
    #[cold]
    fn serialize_lone_surrogates<S: Serializer>(&self, mut serializer: S) {
        const HEX: &[u8; 16] = b"0123456789abcdef";

        let buffer = serializer.buffer_mut();
        buffer.print_ascii_byte(b'"');
        for ch in self.chars() {
            match ch.to_u32() {
                0x08 => buffer.print_str("\\b"),
                0x09 => buffer.print_str("\\t"),
                0x0A => buffer.print_str("\\n"),
                0x0C => buffer.print_str("\\f"),
                0x0D => buffer.print_str("\\r"),
                0x22 => buffer.print_str("\\\""),
                0x5C => buffer.print_str("\\\\"),
                value @ (0..=0x1F | 0xD800..=0xDFFF) => {
                    buffer.print_ascii_bytes([
                        b'\\',
                        b'u',
                        HEX[((value >> 12) & 15) as usize],
                        HEX[((value >> 8) & 15) as usize],
                        HEX[((value >> 4) & 15) as usize],
                        HEX[(value & 15) as usize],
                    ]);
                }
                // Surrogates were handled above; every remaining code point is a scalar.
                _ => buffer.print_char(ch.to_char().unwrap()),
            }
        }
        buffer.print_ascii_byte(b'"');
    }
}
