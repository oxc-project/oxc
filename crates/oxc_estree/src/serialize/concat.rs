use crate::{ESTree, SequenceSerializer, Serializer};

/// Trait for types which can be concatenated.
///
/// Implemented by `Option` and `Vec`.
pub trait ConcatElement {
    fn push_to_sequence<S: SequenceSerializer>(&self, seq: &mut S);
}

impl<T: ESTree> ConcatElement for Option<T> {
    fn push_to_sequence<S: SequenceSerializer>(&self, seq: &mut S) {
        if let Some(value) = self {
            seq.serialize_element(value);
        }
    }
}

/// Helper struct for concatenating 2 elements into a sequence.
pub struct Concat2<'t, C1: ConcatElement, C2: ConcatElement>(pub &'t C1, pub &'t C2);

impl<C1: ConcatElement, C2: ConcatElement> ESTree for Concat2<'_, C1, C2> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        self.0.push_to_sequence(&mut seq);
        self.1.push_to_sequence(&mut seq);
        seq.end();
    }
}

/// Helper struct for concatenating 3 elements into a sequence.
pub struct Concat3<'t, C1: ConcatElement, C2: ConcatElement, C3: ConcatElement>(
    pub &'t C1,
    pub &'t C2,
    pub &'t C3,
);

impl<C1: ConcatElement, C2: ConcatElement, C3: ConcatElement> ESTree for Concat3<'_, C1, C2, C3> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        self.0.push_to_sequence(&mut seq);
        self.1.push_to_sequence(&mut seq);
        self.2.push_to_sequence(&mut seq);
        seq.end();
    }
}
