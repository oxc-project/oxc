// Mathod which are trivial or delegate to other methods marked `#[inline(always)]`
#![expect(clippy::inline_always)]

mod buffer;
mod formatter;
mod primitives;
mod sequences;
mod strings;
mod structs;
mod wrappers;
use buffer::Buffer;
use formatter::{CompactFormatter, Formatter, PrettyFormatter};

pub use sequences::SequenceSerializer;
pub use structs::StructSerializer;

/// Trait for types which can be serialized to ESTree.
pub trait ESTree {
    fn serialize<S: Serializer>(&self, serializer: &mut S);
}

/// Trait for serializers.
#[expect(private_bounds)]
pub trait Serializer: SerializerImpl {
    // Public methods

    /// Serialize struct.
    fn serialize_struct(&mut self) -> StructSerializer<'_, Self> {
        StructSerializer::new(self)
    }

    /// Serialize sequence.
    fn serialize_sequence(&mut self) -> SequenceSerializer<'_, Self> {
        SequenceSerializer::new(self)
    }
}

/// Inner trait containing internal methods that we don't want to expose outside this crate.
trait SerializerImpl: Sized {
    fn buffer_mut(&mut self) -> &mut Buffer;
    fn enter_sequence(&mut self);
    fn enter_element(&mut self);
    fn enter_field_value(&mut self);
    fn exit_sequence(&mut self);
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
    /// Create new [`ESTreeSerializer`] with provided formatter.
    pub fn new() -> Self {
        Self { buffer: Buffer::new(), formatter: F::new() }
    }

    /// Consume [`ESTreeSerializer`] and convert buffer to string.
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

impl<F: Formatter> Serializer for ESTreeSerializer<F> {}

impl<F: Formatter> SerializerImpl for ESTreeSerializer<F> {
    /// Get mutable reference to buffer.
    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    #[inline(always)]
    fn enter_sequence(&mut self) {
        self.formatter.enter_sequence(&mut self.buffer);
    }

    #[inline(always)]
    fn enter_element(&mut self) {
        self.formatter.enter_field(&mut self.buffer);
    }

    #[inline(always)]
    fn enter_field_value(&mut self) {
        self.formatter.enter_field_value(&mut self.buffer);
    }

    #[inline(always)]
    fn exit_sequence(&mut self) {
        self.formatter.exit_sequence(&mut self.buffer);
    }
}
