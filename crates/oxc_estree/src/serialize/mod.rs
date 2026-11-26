// Methods which are trivial or just delegate to other methods are marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use itoa::Buffer as ItoaBuffer;

use oxc_data_structures::{
    code_buffer::{CodeBuffer, IndentChar},
    stack::NonEmptyStack,
};

mod blanket;
mod concat;
mod config;
mod formatter;
mod primitives;
mod sequences;
mod strings;
mod structs;
use sequences::ESTreeSequenceSerializer;
use structs::ESTreeStructSerializer;

pub use concat::{Concat2, Concat3, ConcatElement};
pub use config::{Config, ConfigFixes, ConfigNoFixes};
pub use formatter::{CompactFormatter, Formatter, PrettyFormatter};
pub use sequences::SequenceSerializer;
pub use strings::{JsonSafeString, LoneSurrogatesString};
pub use structs::{ESTreeSpan, FlatStructSerializer, StructSerializer};

/// Trait for types which can be serialized to ESTree.
pub trait ESTree {
    fn serialize<S: Serializer>(&self, serializer: S);
}

/// Trait for serializers.
pub trait Serializer {
    /// `true` if serializer's formatter produces compact JSON (not pretty-printed JSON).
    const IS_COMPACT: bool = Self::Formatter::IS_COMPACT;

    /// Type of `Formatter` this serializer uses.
    type Formatter: Formatter;

    /// Type of struct serializer this serializer uses.
    type StructSerializer: StructSerializer;
    /// Type of sequence serializer this serializer uses.
    type SequenceSerializer: SequenceSerializer;

    /// Get whether output should contain TS fields.
    fn include_ts_fields(&self) -> bool;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;

    /// Serialize struct.
    fn serialize_struct(self) -> Self::StructSerializer;

    /// Serialize sequence.
    fn serialize_sequence(self) -> Self::SequenceSerializer;

    /// Record path to current node in `fixes_buffer`.
    ///
    /// Used by serializers for the `value` field of `BigIntLiteral` and `RegExpLiteral`.
    /// These nodes cannot be serialized to JSON, because JSON doesn't support `BigInt`s or `RegExp`s.
    /// "Fix paths" can be used on JS side to locate these nodes and set their `value` fields correctly.
    fn record_fix_path(&mut self);

    /// Serialize the slice `start..end` of the source text as a string.
    ///
    /// `start` and `end` are offsets from a span in the AST.
    /// If the serializer has a span converter, they are converted from UTF-16 to UTF-8 first.
    ///
    /// Serializes `null` if the serializer has no source text, or `start..end` is not a valid slice of it.
    ///
    /// Used by the serializer for the `raw` field of `StringLiteral`.
    fn serialize_source_text_slice(self, start: u32, end: u32);

    /// Get mutable reference to buffer.
    fn buffer_mut(&mut self) -> &mut CodeBuffer;

    /// Get mutable references to buffer and formatter.
    fn buffer_and_formatter_mut(&mut self) -> (&mut CodeBuffer, &mut Self::Formatter);
}

/// Converter of UTF-16 offsets back to UTF-8 offsets.
///
/// If spans in the AST have been converted to UTF-16 before serializing, the serializer needs
/// a converter to find the UTF-8 byte offsets in source text which spans refer to.
pub trait Utf16ToUtf8 {
    /// Convert a UTF-16 offset to a UTF-8 offset.
    fn utf16_to_utf8(&self, utf16_offset: u32) -> u32;
}

/// ESTree serializer which produces compact JSON.
pub type CompactSerializer<'t> = ESTreeSerializer<'t, ConfigNoFixes, CompactFormatter>;

/// ESTree serializer which produces pretty JSON.
pub type PrettySerializer<'t> = ESTreeSerializer<'t, ConfigNoFixes, PrettyFormatter>;

/// ESTree serializer which produces compact JSON.
pub type CompactFixesSerializer<'t> = ESTreeSerializer<'t, ConfigFixes, CompactFormatter>;

/// ESTree serializer which produces pretty JSON.
pub type PrettyFixesSerializer<'t> = ESTreeSerializer<'t, ConfigFixes, PrettyFormatter>;

/// ESTree serializer.
pub struct ESTreeSerializer<'t, C: Config, F: Formatter> {
    buffer: CodeBuffer,
    formatter: F,
    trace_path: NonEmptyStack<TracePathPart>,
    fixes_buffer: CodeBuffer,
    config: C,
    /// Source text of the AST being serialized. Empty if not provided.
    source_text: &'t str,
    /// Converter for spans in the AST, if they have been converted to UTF-16.
    span_converter: Option<&'t dyn Utf16ToUtf8>,
}

impl<'t, C: Config, F: Formatter> ESTreeSerializer<'t, C, F> {
    /// Create new [`ESTreeSerializer`].
    pub fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self {
            buffer: CodeBuffer::with_indent(IndentChar::Space, 2),
            formatter: F::new(),
            trace_path: NonEmptyStack::new(TracePathPart::Index(0)),
            fixes_buffer: CodeBuffer::new(),
            config: C::new(include_ts_fields, ranges),
            source_text: "",
            span_converter: None,
        }
    }

    /// Create new [`ESTreeSerializer`] with specified buffer capacity.
    pub fn with_capacity(capacity: usize, include_ts_fields: bool, ranges: bool) -> Self {
        Self {
            buffer: CodeBuffer::with_capacity_and_indent(capacity, IndentChar::Space, 2),
            formatter: F::new(),
            trace_path: NonEmptyStack::new(TracePathPart::Index(0)),
            fixes_buffer: CodeBuffer::new(),
            config: C::new(include_ts_fields, ranges),
            source_text: "",
            span_converter: None,
        }
    }

    /// Set the source text of the AST being serialized.
    ///
    /// Fields which are derived from source text (e.g. `raw` field of `StringLiteral`)
    /// are serialized as `null` if source text is not provided.
    ///
    /// If spans in the AST have been converted to UTF-16, `span_converter` must be provided,
    /// to convert them back to UTF-8 offsets in `source_text`.
    #[must_use]
    pub fn with_source_text(
        mut self,
        source_text: &'t str,
        span_converter: Option<&'t dyn Utf16ToUtf8>,
    ) -> Self {
        self.source_text = source_text;
        self.span_converter = span_converter;
        self
    }

    /// Serialize `node` and output a `JSON` string containing
    /// `{ "node": { ... }, "fixes": [ ... ]}`, where `node` is the serialized AST node,
    /// and `fixes` is a list of paths to any `Literal`s which are `BigInt`s or `RegExp`s.
    ///
    /// The `value` field of these nodes cannot be serialized to JSON, because JSON doesn't support
    /// `BigInt`s or `RegExp`s. The `fixes` paths can be used on JS side to locate these nodes
    /// and set their `value` fields correctly.
    ///
    /// # Panics
    ///
    /// Panics if serializer's config does not enable fixes.
    pub fn serialize_with_fixes<T: ESTree>(mut self, node: &T) -> String {
        // For the built-in configs in this crate, `fixes()` is `#[inline(always)]` and returns a constant,
        // so compiler will remove this assertion when fixes are enabled
        assert!(
            self.config.fixes(),
            "Cannot call `serialize_with_fixes` on a serializer without fixes enabled"
        );

        self.buffer.print_str("{\"node\":\n");

        node.serialize(&mut self);

        debug_assert!(self.trace_path.is_exhausted());
        debug_assert_eq!(self.trace_path[0], TracePathPart::DUMMY);

        self.buffer.print_str("\n,\"fixes\":[");
        if !self.fixes_buffer.is_empty() {
            let fixes_buffer = self.fixes_buffer.as_str();
            // Omit leading `,`
            self.buffer.print_str(&fixes_buffer[1..]);
        }
        self.buffer.print_str("]}");

        self.buffer.into_string()
    }

    /// Consume this [`ESTreeSerializer`] and convert buffer to string.
    pub fn into_string(self) -> String {
        self.buffer.into_string()
    }
}

impl<C: Config, F: Formatter> Default for ESTreeSerializer<'_, C, F> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(true, false)
    }
}

impl<'s, 't, C: Config, F: Formatter> Serializer for &'s mut ESTreeSerializer<'t, C, F> {
    type Formatter = F;
    type StructSerializer = ESTreeStructSerializer<'s, 't, C, F>;
    type SequenceSerializer = ESTreeSequenceSerializer<'s, 't, C, F>;

    /// Get whether output should contain TS fields.
    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.config.include_ts_fields()
    }

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.config.ranges()
    }

    /// Serialize struct.
    #[inline(always)]
    fn serialize_struct(self) -> ESTreeStructSerializer<'s, 't, C, F> {
        ESTreeStructSerializer::new(self)
    }

    /// Serialize sequence.
    #[inline(always)]
    fn serialize_sequence(self) -> ESTreeSequenceSerializer<'s, 't, C, F> {
        ESTreeSequenceSerializer::new(self)
    }

    /// Record path to current node in `fixes_buffer`.
    ///
    /// Used by serializers for the `value` field of `BigIntLiteral` and `RegExpLiteral`.
    /// These nodes cannot be serialized to JSON, because JSON doesn't support `BigInt`s or `RegExp`s.
    /// "Fix paths" can be used on JS side to locate these nodes and set their `value` fields correctly.
    fn record_fix_path(&mut self) {
        if !self.config.fixes() {
            return;
        }

        self.fixes_buffer.print_str(",[");

        // First part is a dummy, last part is `"value"`, so skip them
        let parts = self.trace_path.as_slice();
        let parts = &parts[1..parts.len() - 1];
        for (index, part) in parts.iter().enumerate() {
            if index > 0 {
                self.fixes_buffer.print_ascii_byte(b',');
            }
            match *part {
                TracePathPart::Key(key) => {
                    self.fixes_buffer.print_strs_array(["\"", key, "\""]);
                }
                TracePathPart::Index(index) => {
                    let mut buffer = ItoaBuffer::new();
                    let s = buffer.format(index);
                    self.fixes_buffer.print_str(s);
                }
            }
        }

        self.fixes_buffer.print_ascii_byte(b']');
    }

    /// Serialize the slice `start..end` of the source text as a string.
    ///
    /// Serializes `null` if the serializer has no source text, or `start..end` is not a valid slice of it.
    fn serialize_source_text_slice(self, mut start: u32, mut end: u32) {
        if let Some(span_converter) = self.span_converter {
            start = span_converter.utf16_to_utf8(start);
            end = span_converter.utf16_to_utf8(end);
        }

        let source_text = self.source_text;
        if let Some(slice) = source_text.get(start as usize..end as usize) {
            slice.serialize(self);
        } else {
            self.buffer.print_str("null");
        }
    }

    /// Get mutable reference to buffer.
    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut CodeBuffer {
        &mut self.buffer
    }

    /// Get mutable references to buffer and formatter.
    #[inline(always)]
    fn buffer_and_formatter_mut(&mut self) -> (&mut CodeBuffer, &mut F) {
        (&mut self.buffer, &mut self.formatter)
    }
}

/// Element of a trace path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TracePathPart {
    Key(&'static str),
    Index(usize),
}

impl TracePathPart {
    pub const DUMMY: Self = TracePathPart::Index(0);
}
