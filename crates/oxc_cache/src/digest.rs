//! BLAKE3 digests: of file contents, and of the values cache keys are made of.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A BLAKE3 digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest([u8; 32]);

impl Digest {
    /// Hashes `bytes`.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Hashes `value`'s JSON, which is self-delimiting: two different values never hash the same
    /// bytes. Returns `None` if it doesn't serialize (a path that isn't UTF-8).
    pub fn try_of(value: &impl Serialize) -> Option<Self> {
        let mut hasher = blake3::Hasher::new();
        serde_json::to_writer(&mut hasher, value).ok()?;
        Some(Self(*hasher.finalize().as_bytes()))
    }

    /// Hashes `value` as [`Self::try_of`] does, for values that always serialize.
    ///
    /// # Panics
    /// If `value` doesn't serialize.
    pub fn of(value: &impl Serialize) -> Self {
        Self::try_of(value).expect("cache keys serialize")
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keys are made of several fields: no two different tuples share a digest, however their
    /// bytes line up (`("ab", "c")` and `("a", "bc")`).
    #[test]
    fn different_values_have_different_digests() {
        #[expect(clippy::needless_pass_by_value, reason = "quickcheck passes values")]
        fn property(a: (String, Vec<u8>, Option<u32>), b: (String, Vec<u8>, Option<u32>)) -> bool {
            (Digest::of(&a) == Digest::of(&b)) == (a == b)
        }
        quickcheck::QuickCheck::new().tests(1000).quickcheck(property as fn(_, _) -> bool);
        assert_ne!(Digest::of(&("ab", "c")), Digest::of(&("a", "bc")));
    }

    #[test]
    fn a_path_that_isnt_utf8_has_no_digest() {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let path = std::path::Path::new(std::ffi::OsStr::from_bytes(b"a\xff"));
            assert_eq!(Digest::try_of(&path), None);
        }
    }
}
