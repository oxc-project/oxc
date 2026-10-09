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
pub use config::{CommentRecord, Config, ConfigCommentFixes, ConfigFixes, ConfigNoFixes};
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

    /// Record native ownership at the current ESTree path. Default serializers do no work.
    fn record_comment_owner(&mut self, _node_id: u32, _kind: &'static str, _start: u32, _end: u32) {
    }

    /// Get mutable reference to buffer.
    fn buffer_mut(&mut self) -> &mut CodeBuffer;

    /// Get mutable references to buffer and formatter.
    fn buffer_and_formatter_mut(&mut self) -> (&mut CodeBuffer, &mut Self::Formatter);
}

/// ESTree serializer which produces compact JSON.
pub type CompactSerializer = ESTreeSerializer<ConfigNoFixes, CompactFormatter>;

/// ESTree serializer which produces pretty JSON.
pub type PrettySerializer = ESTreeSerializer<ConfigNoFixes, PrettyFormatter>;

/// ESTree serializer which produces compact JSON.
pub type CompactFixesSerializer = ESTreeSerializer<ConfigFixes, CompactFormatter>;

/// ESTree serializer which produces pretty JSON.
pub type PrettyFixesSerializer = ESTreeSerializer<ConfigFixes, PrettyFormatter>;

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
    pub fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self {
            buffer: CodeBuffer::with_indent(IndentChar::Space, 2),
            formatter: F::new(),
            trace_path: NonEmptyStack::new(TracePathPart::Index(0)),
            fixes_buffer: CodeBuffer::new(),
            config: C::new(include_ts_fields, ranges),
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
        }
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
        self.buffer.print_ascii_byte(b']');
        if let Some(fixes) = self.config.comment_fixes() {
            self.buffer.print_str(",\"commentFixes\":[");
            if !fixes.is_empty() {
                self.buffer.print_str(&fixes.as_str()[1..]);
            }
            self.buffer.print_ascii_byte(b']');
        }
        self.buffer.print_ascii_byte(b'}');

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
        Self::new(true, false)
    }
}

impl<'s, C: Config, F: Formatter> Serializer for &'s mut ESTreeSerializer<C, F> {
    type Formatter = F;
    type StructSerializer = ESTreeStructSerializer<'s, C, F>;
    type SequenceSerializer = ESTreeSequenceSerializer<'s, C, F>;

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

    fn record_comment_owner(&mut self, node_id: u32, kind: &'static str, start: u32, end: u32) {
        if !C::COMMENTS {
            return;
        }
        let Some(comments) = self.config.take_comments(node_id) else {
            return;
        };
        let parent = matches!(kind, "FormalParameters" | "WithClause");
        let elision = kind == "Elision";
        let projected = parent
            || elision
            || matches!(kind, "FormalParameter" | "CatchParameter" | "FormalParameterRest");
        let parts = self.trace_path.as_slice();
        // The first trace element is the root sentinel. Array-only containers
        // project to the node containing the current field.
        let parts = &parts[1..parts.len() - if elision { 2 } else { usize::from(parent) }];
        let fixes = self.config.comment_fixes().unwrap();
        for comment in comments {
            fixes.print_str(",[ [");
            for (index, part) in parts.iter().enumerate() {
                if index != 0 {
                    fixes.print_ascii_byte(b',');
                }
                match part {
                    TracePathPart::Key(key) => fixes.print_strs_array(["\"", key, "\""]),
                    TracePathPart::Index(index) => {
                        fixes.print_str(itoa::Buffer::new().format(*index));
                    }
                }
            }
            fixes.print_str("],");
            for value in [
                comment.index,
                u32::from(comment.placement),
                u32::from(comment.kind),
                u32::from(comment.newlines),
                u32::from(comment.content),
            ] {
                fixes.print_str(itoa::Buffer::new().format(value));
                fixes.print_ascii_byte(b',');
            }
            if projected || (kind == "ImportSpecifiers" && comment.placement == 2) {
                fixes.print_strs_array(["[\"", kind, "\","]);
                fixes.print_str(itoa::Buffer::new().format(start));
                fixes.print_ascii_byte(b',');
                fixes.print_str(itoa::Buffer::new().format(end));
                fixes.print_ascii_byte(b']');
            } else {
                fixes.print_str("null");
            }
            fixes.print_ascii_byte(b']');
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

impl<F: Formatter> ESTreeSerializer<ConfigCommentFixes, F> {
    /// Configure the sparse source comment index.
    #[must_use]
    pub fn with_comments(mut self, comments: impl Iterator<Item = CommentRecord>) -> Self {
        self.config.set_comments(comments);
        self
    }
}
