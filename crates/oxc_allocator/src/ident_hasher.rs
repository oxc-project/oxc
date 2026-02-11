//! A custom hasher for `Ident` keys with precomputed hashes.
//!
//! [`IdentBuildHasher`] creates [`IdentHasher`]s that handle two hash paths:
//! * `Ident::hash()` — writes a precomputed `u64` via [`write_u64`](std::hash::Hasher::write_u64)
//! * `str::hash()` — writes bytes via [`write`](std::hash::Hasher::write), then computes hash data
//!
//! Both paths are normalized to the same final state so `Equivalent<Ident> for str` lookups
//! always match `Ident`-keyed inserts.

use std::hash::{BuildHasher, Hasher};

/// Read 4 bytes from `s` at `offset` as a little-endian `u32`.
#[inline]
const fn read_u32(s: &[u8], i: usize) -> u32 {
    (s[i] as u32) | ((s[i + 1] as u32) << 8) | ((s[i + 2] as u32) << 16) | ((s[i + 3] as u32) << 24)
}

/// Fold two `u32` values into one via 64-bit multiply + XOR-fold.
///
/// This is the core mixer from wyhash/rapidhash. The 64-bit multiply is non-linear,
/// so every input bit influences many output bits. The XOR-fold combines the upper
/// and lower halves for full avalanche.
#[inline]
#[expect(clippy::cast_possible_truncation)]
const fn mix(a: u32, b: u32) -> u32 {
    let m = (a as u64).wrapping_mul(b as u64);
    (m as u32) ^ ((m >> 32) as u32)
}

/// Compute a fast, high-quality hash of identifier bytes.
///
/// Design:
/// - **Read all bytes** for short strings (≤8), sample head+middle+tail for longer
/// - **wyhash-style mixing** — 64-bit multiply + XOR-fold provides non-linear
///   combination where every input bit affects many output bits
/// - **Asymmetric seeds** — XOR with different constants before mixing, so even
///   when read windows overlap (e.g. 4-byte strings where head == tail), the two
///   mixer inputs are always different
/// - **`const fn`** — enables compile-time hashing for `ident!("await")` macros
#[inline]
pub const fn ident_hash(s: &[u8]) -> u32 {
    // Seeds from murmur3 / splitmix
    const SEED1: u32 = 0x9E37_79B9;
    const SEED2: u32 = 0x85EB_CA6B;

    let len = s.len();
    if len == 0 {
        return 0;
    }

    if len < 4 {
        // 1-3 bytes: wyhash trick — read first, middle, last byte.
        // Captures all bytes regardless of length (middle overlaps for len=1,2).
        let packed = ((s[0] as u32) << 16) | ((s[len >> 1] as u32) << 8) | (s[len - 1] as u32);
        mix(packed ^ SEED1, packed ^ SEED2)
    } else if len <= 8 {
        // 4-8 bytes: read first 4 + last 4 (covers all bytes, may overlap).
        // XOR with different seeds so even when head == tail (len=4), the mix inputs differ.
        let head = read_u32(s, 0);
        let tail = read_u32(s, len - 4);
        mix(head ^ SEED1, tail ^ SEED2)
    } else if len <= 16 {
        // 9-16 bytes: head + middle + tail (12 bytes sampled).
        let head = read_u32(s, 0);
        let mid = read_u32(s, (len >> 1) - 2);
        let tail = read_u32(s, len - 4);
        mix(head ^ mid ^ SEED1, tail ^ SEED2)
    } else {
        // 17+ bytes: 4 evenly-spaced samples (16 bytes total).
        let head = read_u32(s, 0);
        let mid1 = read_u32(s, len / 3);
        let mid2 = read_u32(s, 2 * len / 3);
        let tail = read_u32(s, len - 4);
        mix(head ^ mid1 ^ SEED1, mid2 ^ tail ^ SEED2)
    }
}

/// Pack length and hash into a single `u64`.
///
/// Lower 32 bits = length, upper 32 bits = hash.
///
/// This is used as compact precomputed data inside `Ident` and as the
/// `Ident::hash()` payload passed into [`IdentHasher::write_u64`].
#[inline]
pub const fn pack_len_hash(len: u32, hash: u32) -> u64 {
    (len as u64) | ((hash as u64) << 32)
}

/// Compose the final hash state used by hashbrown.
///
/// Keep upper bits derived from `hash` (for tag/SIMD filtering), while ensuring
/// the low bits used for bucket index also contain hash entropy instead of being
/// dominated by identifier length.
#[inline]
const fn hashbrown_state(len: u32, hash: u32) -> u64 {
    let low = hash ^ len.rotate_left(16);
    (low as u64) | ((hash as u64) << 32)
}

/// Unpack `len` (low 32 bits) and `hash` (high 32 bits) from packed `u64`.
#[expect(clippy::cast_possible_truncation)]
#[inline]
const fn unpack_len_hash(packed: u64) -> (u32, u32) {
    (packed as u32, (packed >> 32) as u32)
}

/// A [`BuildHasher`] for `Ident`-keyed hash maps.
///
/// Creates [`IdentHasher`]s that accept both precomputed `u64` hashes (from `Ident`)
/// and raw byte slices (from `str` lookups).
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentBuildHasher;

impl BuildHasher for IdentBuildHasher {
    type Hasher = IdentHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        IdentHasher { state: 0 }
    }
}

/// A [`Hasher`] that handles both `Ident` (precomputed) and `str` (computed) hash paths.
#[derive(Debug, Clone, Copy)]
pub struct IdentHasher {
    state: u64,
}

impl Hasher for IdentHasher {
    /// `Ident::hash()` path: decode packed `len|hash` and normalize.
    #[inline]
    fn write_u64(&mut self, i: u64) {
        let (len, hash) = unpack_len_hash(i);
        self.state = hashbrown_state(len, hash);
    }

    /// `str::hash()` path: compute `len` + hash from bytes, then normalize.
    #[inline]
    #[expect(clippy::cast_possible_truncation)] // Identifier strings are always < 4GB
    fn write(&mut self, bytes: &[u8]) {
        let hash = ident_hash(bytes);
        self.state = hashbrown_state(bytes.len() as u32, hash);
    }

    /// `str::hash()` writes a 0xFF terminator byte after the string bytes. Ignore it.
    #[inline]
    fn write_u8(&mut self, _: u8) {}

    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }
}

#[cfg(test)]
mod test {
    use std::hash::{BuildHasher, Hasher};

    use super::{
        IdentBuildHasher, IdentHasher, hashbrown_state, ident_hash, pack_len_hash, unpack_len_hash,
    };

    // ---- ident_hash quality tests ----

    #[test]
    fn hash_empty() {
        assert_eq!(ident_hash(b""), 0);
    }

    #[test]
    fn hash_nonzero_for_all_lengths() {
        // Every non-empty string should produce a non-zero hash
        for s in
            &[b"x" as &[u8], b"ab", b"abc", b"abcd", b"hello", b"useState", b"longIdentifierName"]
        {
            assert_ne!(
                ident_hash(s),
                0,
                "hash should be non-zero for {:?}",
                std::str::from_utf8(s)
            );
        }
    }

    #[test]
    fn hash_discriminates_similar_idents() {
        // Same-length identifiers with small differences should hash differently
        assert_ne!(ident_hash(b"null"), ident_hash(b"void"));
        assert_ne!(ident_hash(b"this"), ident_hash(b"that"));
        assert_ne!(ident_hash(b"abcd"), ident_hash(b"abce"));
        assert_ne!(ident_hash(b"useState"), ident_hash(b"useEffect"));
        assert_ne!(ident_hash(b"fooBar"), ident_hash(b"fooBaz"));
        assert_ne!(ident_hash(b"a"), ident_hash(b"b"));
        assert_ne!(ident_hash(b"ab"), ident_hash(b"ba")); // spellchecker:disable-line
    }

    #[test]
    fn hash_four_byte_no_zero_collision() {
        // The old hash had all 4-byte strings colliding to 0. Verify that's fixed.
        let keywords = [b"null", b"void", b"this", b"that", b"true", b"else", b"case", b"from"];
        for kw in &keywords {
            assert_ne!(ident_hash(*kw), 0, "4-byte keyword {kw:?} should not hash to 0");
        }
        // Also verify they're all distinct
        for (i, a) in keywords.iter().enumerate() {
            for b in &keywords[i + 1..] {
                assert_ne!(
                    ident_hash(*a),
                    ident_hash(*b),
                    "{a:?} and {b:?} should have different hashes",
                );
            }
        }
    }

    #[test]
    fn hash_long_strings_with_middle_differences() {
        // Strings that differ only in the middle — the 17+ path must catch this
        assert_ne!(
            ident_hash(b"privateInterfaceWithPrivatePropertyTypes"),
            ident_hash(b"privateInterfaceWithPrivateParmeterTypes"), // spellchecker:disable-line
        );
    }

    // ---- pack_len_hash tests ----

    #[test]
    fn pack_round_trip() {
        let len = 42u32;
        let hash = 0xDEAD_BEEFu32;
        let packed = pack_len_hash(len, hash);
        assert_eq!((packed & 0xFFFF_FFFF) as u32, len);
        assert_eq!((packed >> 32) as u32, hash);
    }

    // ---- IdentHasher tests ----

    #[test]
    fn hasher_write_u64_path() {
        let mut hasher = IdentHasher { state: 0 };
        let hash = ident_hash(b"hello");
        let packed = pack_len_hash(5, hash);
        hasher.write_u64(packed);
        assert_eq!(hasher.finish(), hashbrown_state(5, hash));
    }

    #[test]
    fn hasher_write_bytes_path() {
        let mut hasher = IdentHasher { state: 0 };
        hasher.write(b"hello");
        let hash = ident_hash(b"hello");
        let expected = hashbrown_state(5, hash);
        assert_eq!(hasher.finish(), expected);
    }

    #[test]
    fn str_hash_matches_ident_packed() {
        // Simulate what str::hash() does: write bytes, then write_u8(0xFF)
        let build_hasher = IdentBuildHasher;
        let mut hasher = build_hasher.build_hasher();
        hasher.write(b"fooBar");
        hasher.write_u8(0xFF);
        let str_hash = hasher.finish();

        // Simulate what Ident::hash() does: write_u64 with packed value
        let mut hasher2 = build_hasher.build_hasher();
        let packed = pack_len_hash(6, ident_hash(b"fooBar"));
        hasher2.write_u64(packed);
        let ident_hash_val = hasher2.finish();

        assert_eq!(str_hash, ident_hash_val);
    }

    #[test]
    fn build_hasher_default() {
        let bh = IdentBuildHasher;
        let hasher = bh.build_hasher();
        assert_eq!(hasher.finish(), 0);
    }

    /// Test that `str` hashing through standard `Hash` trait produces the same
    /// result as our manual `write` path.
    #[test]
    fn std_str_hash_compatible() {
        let build_hasher = IdentBuildHasher;
        let std_hash = build_hasher.hash_one("hello");

        let mut hasher2 = build_hasher.build_hasher();
        hasher2.write(b"hello");
        hasher2.write_u8(0xFF);
        let manual_hash = hasher2.finish();

        assert_eq!(std_hash, manual_hash);
    }

    /// Verify str and Ident hash paths agree across all length buckets.
    #[test]
    fn str_hash_matches_across_lengths() {
        let test_cases = [
            "x",                                        // 1 byte
            "ab",                                       // 2 bytes
            "foo",                                      // 3 bytes
            "this",                                     // 4 bytes
            "hello",                                    // 5 bytes
            "fooBar",                                   // 6 bytes
            "useState",                                 // 8 bytes
            "useEffect",                                // 9 bytes
            "myVariable123",                            // 13 bytes
            "longIdentifierNm",                         // 17 bytes
            "privateInterfaceWithPrivatePropertyTypes", // 40 bytes
        ];

        let build_hasher = IdentBuildHasher;
        for s in &test_cases {
            // str path
            let str_hash = build_hasher.hash_one(*s);

            // Ident path (precomputed)
            let mut hasher = build_hasher.build_hasher();
            #[expect(clippy::cast_possible_truncation)]
            let packed = pack_len_hash(s.len() as u32, ident_hash(s.as_bytes()));
            hasher.write_u64(packed);
            let ident_hash_val = hasher.finish();

            assert_eq!(str_hash, ident_hash_val, "hash mismatch for {s:?} (len={})", s.len());
        }
    }

    #[test]
    fn hashbrown_state_uses_hash_entropy_for_h1() {
        let len = 6u32;
        let hash1 = ident_hash(b"fooBar");
        let hash2 = ident_hash(b"fooBaz");

        let state1 = hashbrown_state(len, hash1);
        let state2 = hashbrown_state(len, hash2);

        // Keep tag bits tied to identifier hash.
        assert_eq!((state1 >> 32) as u32, hash1);
        assert_eq!((state2 >> 32) as u32, hash2);
        // Ensure same-length identifiers don't collapse to same low bits.
        let (low1, _) = unpack_len_hash(state1);
        let (low2, _) = unpack_len_hash(state2);
        assert_ne!(low1, low2);
    }
}
