use serde::ser::{Serialize, SerializeSeq, Serializer};

/// A helper struct for serializing a sequence followed by an optional element.
/// This is only used by generated ESTree serialization code.
pub struct AppendTo<'a, TVec, TAfter> {
    pub array: &'a [TVec],
    pub after: &'a Option<TAfter>,
}

impl<TVec: Serialize, TAfter: Serialize> Serialize for AppendTo<'_, TVec, TAfter> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(after) = self.after {
            let mut seq = serializer.serialize_seq(Some(self.array.len() + 1))?;
            for element in self.array {
                seq.serialize_element(element)?;
            }
            seq.serialize_element(after)?;
            seq.end()
        } else {
            self.array.serialize(serializer)
        }
    }
}

pub struct AppendToConcat<'a, TVec, TAfter> {
    pub array: &'a [TVec],
    pub after: &'a [TAfter],
}

impl<TVec: Serialize, TAfter: Serialize> Serialize for AppendToConcat<'_, TVec, TAfter> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.array.len() + self.after.len()))?;
        for element in self.array {
            seq.serialize_element(element)?;
        }
        for element in self.after {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}
