use super::{ESTree, Serializer};

/// Type that is output as `[]`.
///
/// Could alternatively serialize e.g. `[(); 0]`, but this is faster.
pub struct EmptyArray;

impl ESTree for EmptyArray {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        serializer.buffer_mut().print_str("[]");
    }
}
