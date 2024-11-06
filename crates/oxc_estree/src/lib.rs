#[cfg(feature = "serialize")]
use serde::ser::{Serialize, SerializeSeq, Serializer};

/// Empty trait that will be used later for custom serialization and TypeScript
/// generation for AST nodes.
pub trait ESTree {}

#[cfg(feature = "serialize")]
pub struct AppendTo<'a, TVec, TChild>(pub &'a [TVec], pub &'a Option<TChild>);

#[cfg(feature = "serialize")]
impl<'b, TVec: Serialize, TChild: Serialize> Serialize for AppendTo<'b, TVec, TChild> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(child) = self.1 {
            let mut seq = serializer.serialize_seq(Some(self.0.len() + 1))?;
            for element in self.0 {
                seq.serialize_element(element)?;
            }
            seq.serialize_element(child)?;
            seq.end()
        } else {
            self.0.serialize(serializer)
        }
    }
}
