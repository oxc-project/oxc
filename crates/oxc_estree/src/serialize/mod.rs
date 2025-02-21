// Methods which are trivial or just delegate to other methods are marked `#[inline(always)]`
#![expect(clippy::inline_always)]

mod blanket;
mod buffer;
mod config;
mod formatter;
mod primitives;
mod sequences;
mod strings;
mod structs;
use buffer::Buffer;
use config::{Config, ConfigJS, ConfigTS};
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

/// ESTree serializer which produces compact JSON, including TypeScript fields.
pub type CompactTSSerializer = ESTreeSerializer<ConfigTS, CompactFormatter>;

/// ESTree serializer which produces compact JSON, excluding TypeScript fields.
pub type CompactJSSerializer = ESTreeSerializer<ConfigJS, CompactFormatter>;

/// ESTree serializer which produces pretty JSON, including TypeScript fields.
pub type PrettyTSSerializer = ESTreeSerializer<ConfigTS, PrettyFormatter>;

/// ESTree serializer which produces pretty JSON, excluding TypeScript fields.
pub type PrettyJSSerializer = ESTreeSerializer<ConfigJS, PrettyFormatter>;

/// ESTree serializer.
pub struct ESTreeSerializer<C: Config, F: Formatter> {
    buffer: Buffer,
    formatter: F,
    #[expect(unused)]
    config: C,
}

impl<C: Config, F: Formatter> ESTreeSerializer<C, F> {
    /// Create new [`ESTreeSerializer`].
    pub fn new() -> Self {
        Self { buffer: Buffer::new(), formatter: F::new(), config: C::new() }
    }

    /// Consume this [`ESTreeSerializer`] and convert buffer to string.
    pub fn into_string(self) -> String {
        self.buffer.into_string()
    }
}

impl<C: Config, F: Formatter> Default for ESTreeSerializer<C, F> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<'s, C: Config, F: Formatter> Serializer for &'s mut ESTreeSerializer<C, F> {
    type StructSerializer = ESTreeStructSerializer<'s, C, F>;
    type SequenceSerializer = ESTreeSequenceSerializer<'s, C, F>;

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
}

impl<C: Config, F: Formatter> SerializerPrivate for &mut ESTreeSerializer<C, F> {
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
