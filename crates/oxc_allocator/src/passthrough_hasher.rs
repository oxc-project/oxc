//! A no-op hasher for pre-computed hashes.
//!
//! [`PassthroughBuildHasher`] creates [`PassthroughHasher`]s which pass a pre-computed `u64` hash
//! value through untouched. This is intended for use with hash maps/sets where keys have
//! pre-computed hashes (e.g. interned strings).
//!
//! # How hashbrown uses the hash
//!
//! hashbrown splits the `u64` hash into:
//! * **h1** (low bits): bucket index
//! * **h2** (top 7 bits): SIMD fingerprint for fast filtering
//!
//! The pre-computed hash should have good entropy in both the low and high bits to work well
//! with this scheme.

use std::hash::{BuildHasher, Hasher};

/// A [`BuildHasher`] that creates [`PassthroughHasher`]s.
///
/// For use with hash maps/sets where keys store a pre-computed hash.
/// The hash value is passed through without any additional hashing.
#[derive(Debug, Clone, Copy, Default)]
pub struct PassthroughBuildHasher;

impl BuildHasher for PassthroughBuildHasher {
    type Hasher = PassthroughHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        PassthroughHasher(0)
    }
}

/// A [`Hasher`] that passes a pre-computed `u64` hash through untouched.
///
/// Only supports [`write_u64`](Hasher::write_u64). All other `write_*` methods will panic,
/// as this hasher is only intended for use with types that have a pre-computed `u64` hash.
#[derive(Debug, Clone, Copy)]
pub struct PassthroughHasher(u64);

impl Hasher for PassthroughHasher {
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!("PassthroughHasher only supports write_u64");
    }
}

#[cfg(test)]
mod test {
    use std::hash::{BuildHasher, Hash, Hasher};

    use super::{PassthroughBuildHasher, PassthroughHasher};

    #[test]
    fn passthrough_hasher_passes_u64() {
        let mut hasher = PassthroughHasher(0);
        hasher.write_u64(0x1234_5678_9ABC_DEF0);
        assert_eq!(hasher.finish(), 0x1234_5678_9ABC_DEF0);
    }

    #[test]
    fn build_hasher_creates_passthrough() {
        let build_hasher = PassthroughBuildHasher;
        let mut hasher = build_hasher.build_hasher();
        hasher.write_u64(42);
        assert_eq!(hasher.finish(), 42);
    }

    #[test]
    #[should_panic(expected = "PassthroughHasher only supports write_u64")]
    fn passthrough_hasher_panics_on_write_bytes() {
        let mut hasher = PassthroughHasher(0);
        hasher.write(b"hello");
    }

    /// A type with a pre-computed hash, to test `PassthroughBuildHasher` in a HashMap.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PreHashedKey {
        hash: u64,
        value: u32,
    }

    impl Hash for PreHashedKey {
        fn hash<H: Hasher>(&self, state: &mut H) {
            state.write_u64(self.hash);
        }
    }

    #[test]
    fn hashmap_with_passthrough() {
        let mut map = hashbrown::HashMap::<PreHashedKey, &str, PassthroughBuildHasher>::with_hasher(
            PassthroughBuildHasher,
        );

        let key1 = PreHashedKey { hash: 111, value: 1 };
        let key2 = PreHashedKey { hash: 222, value: 2 };

        map.insert(key1, "one");
        map.insert(key2, "two");

        assert_eq!(map.get(&key1), Some(&"one"));
        assert_eq!(map.get(&key2), Some(&"two"));
        assert_eq!(map.len(), 2);
    }
}
