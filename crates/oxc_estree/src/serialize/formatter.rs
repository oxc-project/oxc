use std::iter;

use oxc_data_structures::code_buffer::CodeBuffer;

/// Formatter trait.
pub trait Formatter {
    /// Create new [`Formatter`].
    fn new() -> Self;

    /// Called before the first field of a struct or element of a sequence.
    /// If the struct/sequence has no fields/elements, this is not called.
    fn before_first_element(&mut self, buffer: &mut CodeBuffer);

    /// Called before a later field of a struct or element of a sequence
    /// (i.e. not the first field/element).
    fn before_later_element(&mut self, buffer: &mut CodeBuffer);

    /// Called after the key of a struct field.
    fn before_field_value(&mut self, buffer: &mut CodeBuffer);

    /// Called after the last element of a sequence / last element of a struct.
    /// If the struct/sequence has no fields/elements, this is not called.
    fn after_last_element(&mut self, buffer: &mut CodeBuffer);
}

/// Compact formatter.
///
/// All methods are no-ops.
/// This formatter does not insert line breaks, indentation, or whitespace.
///
/// e.g. `{"type":"Program","start":0,"end":0,"body":[]}`
pub struct CompactFormatter;

impl Formatter for CompactFormatter {
    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline(always)]
    fn before_first_element(&mut self, _buffer: &mut CodeBuffer) {}

    #[inline(always)]
    fn before_later_element(&mut self, _buffer: &mut CodeBuffer) {}

    #[inline(always)]
    fn before_field_value(&mut self, _buffer: &mut CodeBuffer) {}

    #[inline(always)]
    fn after_last_element(&mut self, _buffer: &mut CodeBuffer) {}
}

/// Pretty-print formatter.
///
/// Produces pretty-formatted JSON with line breaks, indentation, and whitespace.
///
/// e.g.:
///
/// ```json
/// {
///   "type": "Program",
///   "start": 0,
///   "end": 0,
///   "body": []
/// }
/// ```
///
/// Note that empty structs/sequences do not contain line breaks (`[]` not `[\n  ]`, `{}` not `{\n  }`)
/// because `before_first_element` and `after_last_element` are only called if the struct/sequence
/// contains at least 1 element.
pub struct PrettyFormatter {
    indent: usize,
}

impl Formatter for PrettyFormatter {
    #[inline(always)]
    fn new() -> Self {
        Self { indent: 0 }
    }

    fn before_first_element(&mut self, buffer: &mut CodeBuffer) {
        self.indent += 2;
        self.push_new_line_and_indent(buffer);
    }

    fn before_later_element(&mut self, buffer: &mut CodeBuffer) {
        self.push_new_line_and_indent(buffer);
    }

    fn before_field_value(&mut self, buffer: &mut CodeBuffer) {
        buffer.print_ascii_byte(b' ');
    }

    fn after_last_element(&mut self, buffer: &mut CodeBuffer) {
        self.indent -= 2;
        self.push_new_line_and_indent(buffer);
    }
}

impl PrettyFormatter {
    fn push_new_line_and_indent(&self, buffer: &mut CodeBuffer) {
        buffer.print_ascii_byte(b'\n');
        // SAFETY: Spaces are ASCII
        unsafe { buffer.print_bytes_iter_unchecked(iter::repeat_n(b' ', self.indent)) };
    }
}
