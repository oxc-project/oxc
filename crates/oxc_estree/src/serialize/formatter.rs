use super::Buffer;

/// Formatter trait.
pub trait Formatter {
    fn new() -> Self;

    fn enter_sequence(&mut self, buffer: &mut Buffer);

    fn enter_field(&mut self, buffer: &mut Buffer);

    fn enter_field_value(&mut self, buffer: &mut Buffer);

    fn exit_sequence(&mut self, buffer: &mut Buffer);
}

/// Compact formatter.
pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline(always)]
    fn enter_sequence(&mut self, _buffer: &mut Buffer) {}

    #[inline(always)]
    fn enter_field(&mut self, _buffer: &mut Buffer) {}

    #[inline(always)]
    fn enter_field_value(&mut self, _buffer: &mut Buffer) {}

    #[inline(always)]
    fn exit_sequence(&mut self, _buffer: &mut Buffer) {}
}

/// Pretty-print formatter.
pub struct PrettyFormatter {
    indent: usize,
}

impl Formatter for PrettyFormatter {
    #[inline(always)]
    fn new() -> Self {
        Self { indent: 0 }
    }

    #[inline]
    fn enter_sequence(&mut self, buffer: &mut Buffer) {
        self.indent += 1;
        buffer.push_ascii_byte(b'\n');
        self.push_indent(buffer);
    }

    fn enter_field(&mut self, buffer: &mut Buffer) {
        buffer.push_ascii_byte(b'\n');
        self.push_indent(buffer);
    }

    fn enter_field_value(&mut self, buffer: &mut Buffer) {
        buffer.push_ascii_byte(b' ');
    }

    fn exit_sequence(&mut self, buffer: &mut Buffer) {
        self.indent -= 1;
        buffer.push_ascii_byte(b'\n');
        self.push_indent(buffer);
    }
}

impl PrettyFormatter {
    fn push_indent(&self, buffer: &mut Buffer) {
        // SAFETY: Spaces are ASCII
        unsafe { buffer.push_bytes(&b"  ".repeat(self.indent)) };
    }
}
