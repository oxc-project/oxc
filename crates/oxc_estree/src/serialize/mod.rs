// Methods which are trivial or just delegate to other methods are marked `#[inline(always)]`
#![expect(clippy::inline_always)]

use std::mem;

use itoa::Buffer as ItoaBuffer;

use oxc_data_structures::{code_buffer::CodeBuffer, stack::NonEmptyStack};

mod blanket;
mod concat;
mod config;
mod formatter;
mod primitives;
mod sequences;
mod strings;
mod structs;
use config::{Config, ConfigFixesJS, ConfigFixesTS, ConfigJS, ConfigTS};
use formatter::{CompactFormatter, Formatter, PrettyFormatter};
use sequences::ESTreeSequenceSerializer;
use structs::ESTreeStructSerializer;

pub use concat::{Concat2, Concat3, ConcatElement};
pub use sequences::SequenceSerializer;
pub use strings::{JsonSafeString, LoneSurrogatesString};
pub use structs::{ESTreeSpan, FlatStructSerializer, StructSerializer};

/// Trait for types which can be serialized to ESTree.
pub trait ESTree {
    fn serialize<S: Serializer>(&self, serializer: S);
}

/// Trait for serializers.
//
// This trait contains public methods.
// Internal methods we don't want to expose outside this crate are in [`SerializerPrivate`] trait.
#[expect(private_bounds)]
pub trait Serializer: SerializerPrivate {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;

    /// Type of struct serializer this serializer uses.
    type StructSerializer: StructSerializer;
    /// Type of sequence serializer this serializer uses.
    type SequenceSerializer: SequenceSerializer;

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
}

/// Trait containing internal methods of [`Serializer`]s that we don't want to expose outside this crate.
trait SerializerPrivate: Sized {
    /// Formatter type
    type Formatter: Formatter;

    /// Get mutable reference to buffer.
    fn buffer_mut(&mut self) -> &mut CodeBuffer;

    /// Get mutable references to buffer and formatter.
    fn buffer_and_formatter_mut(&mut self) -> (&mut CodeBuffer, &mut Self::Formatter);
}

/// ESTree serializer which produces compact JSON, including TypeScript fields.
pub type CompactTSSerializer = ESTreeSerializer<ConfigTS, CompactFormatter>;

/// ESTree serializer which produces compact JSON, excluding TypeScript fields.
pub type CompactJSSerializer = ESTreeSerializer<ConfigJS, CompactFormatter>;

/// ESTree serializer which produces pretty JSON, including TypeScript fields.
pub type PrettyTSSerializer = ESTreeSerializer<ConfigTS, PrettyFormatter>;

/// ESTree serializer which produces pretty JSON, excluding TypeScript fields.
pub type PrettyJSSerializer = ESTreeSerializer<ConfigJS, PrettyFormatter>;

/// ESTree serializer which produces compact JSON, including TypeScript fields.
pub type CompactFixesTSSerializer = ESTreeSerializer<ConfigFixesTS, CompactFormatter>;

/// ESTree serializer which produces compact JSON, excluding TypeScript fields.
pub type CompactFixesJSSerializer = ESTreeSerializer<ConfigFixesJS, CompactFormatter>;

/// ESTree serializer which produces pretty JSON, including TypeScript fields.
pub type PrettyFixesTSSerializer = ESTreeSerializer<ConfigFixesTS, PrettyFormatter>;

/// ESTree serializer which produces pretty JSON, excluding TypeScript fields.
pub type PrettyFixesJSSerializer = ESTreeSerializer<ConfigFixesJS, PrettyFormatter>;

/// ESTree serializer.
pub struct ESTreeSerializer<C: Config, F: Formatter> {
    buffer: CodeBuffer,
    formatter: F,
    trace_path: NonEmptyStack<TracePathPart>,
    fixes_buffer: CodeBuffer,
    config: C,
}

impl<C: Config, F: Formatter> ESTreeSerializer<C, F> {
    /// Create new [`ESTreeSerializer`].
    pub fn new(ranges: bool) -> Self {
        Self {
            buffer: CodeBuffer::new(),
            formatter: F::new(),
            trace_path: NonEmptyStack::new(TracePathPart::Index(0)),
            fixes_buffer: CodeBuffer::new(),
            config: C::new(ranges),
        }
    }

    /// Create new [`ESTreeSerializer`] with specified buffer capacity.
    pub fn with_capacity(capacity: usize, ranges: bool) -> Self {
        Self {
            buffer: CodeBuffer::with_capacity(capacity),
            formatter: F::new(),
            trace_path: NonEmptyStack::new(TracePathPart::Index(0)),
            fixes_buffer: CodeBuffer::new(),
            config: C::new(ranges),
        }
    }

    /// Serialize `node` and output a `JSON` string containing
    /// `{ "node": { ... }, "fixes": [ ... ]}`, where `node` is the serialized AST node,
    /// and `fixes` is a list of paths to any `Literal`s which are `BigInt`s or `RegExp`s.
    ///
    /// The `value` field of these nodes cannot be serialized to JSON, because JSON doesn't support
    /// `BigInt`s or `RegExp`s. The `fixes` paths can be used on JS side to locate these nodes
    /// and set their `value` fields correctly.
    pub fn serialize_with_fixes<T: ESTree>(mut self, node: &T) -> String {
        const {
            assert!(
                C::FIXES,
                "Cannot call `serialize_with_fixes` on a serializer without fixes enabled"
            );
        }

        self.buffer.print_str("{\"node\":\n");

        node.serialize(&mut self);

        debug_assert!(self.trace_path.is_exhausted());
        debug_assert_eq!(self.trace_path[0], TracePathPart::DUMMY);

        self.buffer.print_str("\n,\"fixes\":[");
        if !self.fixes_buffer.is_empty() {
            let traces_buffer = mem::take(&mut self.fixes_buffer).into_string();
            self.buffer.print_str(&traces_buffer[1..]);
        }
        self.buffer.print_str("]}");

        self.buffer.into_string()
    }

    /// Consume this [`ESTreeSerializer`] and convert buffer to string.
    pub fn into_string(self) -> String {
        self.buffer.into_string()
    }
}

impl<C: Config, F: Formatter> Default for ESTreeSerializer<C, F> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(false)
    }
}

impl<'s, C: Config, F: Formatter> Serializer for &'s mut ESTreeSerializer<C, F> {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool = C::INCLUDE_TS_FIELDS;

    type StructSerializer = ESTreeStructSerializer<'s, C, F>;
    type SequenceSerializer = ESTreeSequenceSerializer<'s, C, F>;

    /// Get whether output should contain `range` fields.
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.config.ranges()
    }

    /// Serialize struct.
    #[inline(always)]
    fn serialize_struct(self) -> ESTreeStructSerializer<'s, C, F> {
        ESTreeStructSerializer::new(self)
    }

    /// Serialize sequence.
    #[inline(always)]
    fn serialize_sequence(self) -> ESTreeSequenceSerializer<'s, C, F> {
        ESTreeSequenceSerializer::new(self)
    }

    /// Record path to current node in `fixes_buffer`.
    ///
    /// Used by serializers for the `value` field of `BigIntLiteral` and `RegExpLiteral`.
    /// These nodes cannot be serialized to JSON, because JSON doesn't support `BigInt`s or `RegExp`s.
    /// "Fix paths" can be used on JS side to locate these nodes and set their `value` fields correctly.
    fn record_fix_path(&mut self) {
        if !C::FIXES {
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
                    self.fixes_buffer.print_ascii_byte(b'"');
                    self.fixes_buffer.print_str(key);
                    self.fixes_buffer.print_ascii_byte(b'"');
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
}

impl<C: Config, F: Formatter> SerializerPrivate for &mut ESTreeSerializer<C, F> {
    type Formatter = F;

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
