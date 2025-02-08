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
use structs::{StructState, StructStateTrait};

pub use sequences::SequenceSerializer;
pub use structs::StructSerializer;

/// Trait for types which can be serialized to ESTree.
pub trait ESTree {
    fn serialize<S: Serializer>(&self, serializer: &mut S);
}

/// Trait for serializers.
#[expect(private_bounds)]
pub trait Serializer: SerializerWithFormatter {
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
trait SerializerWithFormatter: Sized {
    /// Formatter type
    type Formatter: Formatter;

    type StructState: StructStateTrait;

    /// `true` if this is the root serializer
    const IS_ROOT: bool = true;

    fn get_struct_state(&self) -> StructState;
    fn set_struct_state(&mut self, state: StructState);

    /// Get mutable reference to buffer.
    fn buffer_mut(&mut self) -> &mut Buffer;

    /// Get mutable reference to buffer and formatter.
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

impl<F: Formatter> SerializerWithFormatter for ESTreeSerializer<F> {
    type Formatter = F;
    type StructState = StructState;

    /// This is the root serializer
    const IS_ROOT: bool = true;

    fn get_struct_state(&self) -> StructState {
        unreachable!()
    }

    fn set_struct_state(&mut self, _state: StructState) {
        unreachable!()
    }

    /// Get mutable reference to buffer.
    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Get mutable reference to buffer and formatter.
    #[inline(always)]
    fn buffer_and_formatter_mut(&mut self) -> (&mut Buffer, &mut F) {
        (&mut self.buffer, &mut self.formatter)
    }
}
