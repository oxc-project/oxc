use crate::{ESTree, SequenceSerializer, Serializer};

/// A helper struct for serializing a sequence followed by an optional element.
/// This is only used by generated ESTree serialization code.
pub struct AppendTo<'a, TVec, TAfter> {
    pub array: &'a [TVec],
    pub after: &'a Option<TAfter>,
}

impl<TVec: ESTree, TAfter: ESTree> ESTree for AppendTo<'_, TVec, TAfter> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        if let Some(after) = self.after {
            let mut seq = serializer.serialize_sequence();
            for element in self.array {
                seq.serialize_element(element);
            }
            seq.serialize_element(after);
            seq.end();
        } else {
            self.array.serialize(serializer);
        }
    }
}

pub struct AppendToConcat<'a, TVec, TAfter> {
    pub array: &'a [TVec],
    pub after: &'a [TAfter],
}

impl<TVec: ESTree, TAfter: ESTree> ESTree for AppendToConcat<'_, TVec, TAfter> {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut seq = serializer.serialize_sequence();
        for element in self.array {
            seq.serialize_element(element);
        }
        for element in self.after {
            seq.serialize_element(element);
        }
        seq.end();
    }
}
