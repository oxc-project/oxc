// Methods which are trivial or just delegate to other methods are marked `#[inline(always)]`
#![expect(clippy::inline_always)]

mod blanket;
mod buffer;
mod formatter;
mod primitives;
mod sequences;
mod strings;
mod structs;
use buffer::Buffer;
use formatter::{CompactFormatter, Formatter, PrettyFormatter};
use sequences::ESTreeSequenceSerializer;
use structs::ESTreeStructSerializer;

pub use sequences::SequenceSerializer;
pub use structs::{FlatStructSerializer, StructSerializer};

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
    /// Type of struct serializer this serializer uses.
    type StructSerializer: StructSerializer;
    /// Type of sequence serializer this serializer uses.
    type SequenceSerializer: SequenceSerializer;

    /// Serialize struct.
    fn serialize_struct(self) -> Self::StructSerializer;

    /// Serialize sequence.
    fn serialize_sequence(self) -> Self::SequenceSerializer;
}

/// Trait containing internal methods of [`Serializer`]s that we don't want to expose outside this crate.
trait SerializerPrivate: Sized {
    /// Formatter type
    type Formatter: Formatter;

    /// Get mutable reference to buffer.
    fn buffer_mut(&mut self) -> &mut Buffer;

    /// Get mutable references to buffer and formatter.
    fn buffer_and_formatter_mut(&mut self) -> (&mut Buffer, &mut Self::Formatter);
}

/// ESTree serializer which produces compact JSON.
pub type CompactSerializer = ESTreeSerializer<CompactFormatter>;

/// ESTree serializer which produces pretty JSON.
pub type PrettySerializer = ESTreeSerializer<PrettyFormatter>;

/// ESTree serializer.
pub struct ESTreeSerializer<F: Formatter> {
    buffer: Buffer,
    formatter: F,
}

impl<F: Formatter> ESTreeSerializer<F> {
    /// Create new [`ESTreeSerializer`].
    pub fn new() -> Self {
        Self { buffer: Buffer::new(), formatter: F::new() }
    }

    /// Consume this [`ESTreeSerializer`] and convert buffer to string.
    pub fn into_string(self) -> String {
        self.buffer.into_string()
    }
}

impl<F: Formatter> Default for ESTreeSerializer<F> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<'s, F: Formatter> Serializer for &'s mut ESTreeSerializer<F> {
    type StructSerializer = ESTreeStructSerializer<'s, F>;
    type SequenceSerializer = ESTreeSequenceSerializer<'s, F>;

    /// Serialize struct.
    #[inline(always)]
    fn serialize_struct(self) -> ESTreeStructSerializer<'s, F> {
        ESTreeStructSerializer::new(self)
    }

    /// Serialize sequence.
    #[inline(always)]
    fn serialize_sequence(self) -> ESTreeSequenceSerializer<'s, F> {
        ESTreeSequenceSerializer::new(self)
    }
}

impl<F: Formatter> SerializerPrivate for &mut ESTreeSerializer<F> {
    type Formatter = F;

    /// Get mutable reference to buffer.
    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Get mutable references to buffer and formatter.
    #[inline(always)]
    fn buffer_and_formatter_mut(&mut self) -> (&mut Buffer, &mut F) {
        (&mut self.buffer, &mut self.formatter)
    }
}
