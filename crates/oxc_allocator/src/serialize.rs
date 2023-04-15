use serde::{ser::SerializeSeq, Serialize, Serializer};

use crate::Box;

impl<'alloc, T> Serialize for Box<'alloc, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(s)
    }
}

impl<'alloc, T> Serialize for Vec<'alloc, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}
