//! Perfect hash table mapping multi-byte operators (e.g. `==`, `>>>`) to their [`TokenKind`].
//!
//! A candidate is the next 4 bytes of source plus a length to try (2, 3 or 4).
//!
//! There is only one 4-byte operator (`>>>=`), so it's not included in the hash table.
//! [`opmap_longest`] checks it separately. [`opmap_pack`] only checks 2-byte and 3-byte operators.
//!
//! [`op_key`] packs bytes + length into a `u32` key:
//! - Bottom 3 bytes: The candidate's first 3 bytes, with the 3rd zeroed if the length is 2.
//! - Top byte: 0.
//!
//! [`op_slot`] hashes the key by multiplying it by [`OPMAP_MUL`] and taking the top [`HASH_BITS`] bits
//! of the product as the slot index.
//!
//! [`OP_PACK`] holds one `u32` per slot, built at compile time from [`OPMAP_OPS`]:
//! - Bottom 3 bytes: Operator's first 3 bytes, with the 3rd byte 0 for a 2-byte operator.
//! - Top byte: Operator's [`TokenKind`].
//!
//! Empty slots hold 0.
//!
//! So a lookup is a multiply, a load, and a comparison of the key and bottom bytes of slot value.
//! If they match, the top byte of the slot value is the [`TokenKind`].
//! [`opmap_lookup`] does the whole lookup.
//! [`opmap_pack`] returns the slot value, for callers which do the comparison themselves.
//! [`opmap_longest`] checks for 4-byte, then 3-byte, then 2-byte operators in turn.
//!
//! [`OPMAP_MUL`] is hard-coded. `test_perfect_hash` test checks that it produces no collisions.
//! If a change to the operator list breaks that, the test's failure message gives a replacement.

use crate::token::{TokenKind, tk};

use crate::pipeline::bytes::is_digit;

/// Number of bits in operator perfect hash.
const HASH_BITS: usize = 6;

/// Number of entries in tables indexed by hash.
const HASH_TABLE_SIZE: usize = 1 << HASH_BITS;

/// Multiplier for the operator perfect hash.
const OPMAP_MUL: u32 = 0x0241_72D5;

/// An operator and its corresponding [`TokenKind`].
struct OpDef {
    txt: &'static [u8],
    kind: TokenKind,
}

impl OpDef {
    /// Create new [`OpDef`].
    const fn new(txt: &'static str, kind: TokenKind) -> Self {
        assert!(txt.len() == 2 || txt.len() == 3, "`txt` must be 2 or 3 bytes long");
        Self { txt: txt.as_bytes(), kind }
    }

    /// Get length of this operator as a `u32`.
    const fn len(&self) -> u32 {
        self.txt.len() as u32
    }

    /// Get a `[u8; 4]` containing all the bytes of this operator and 0 for any remaining bytes.
    const fn bytes(&self) -> [u8; 4] {
        first_4_bytes(self.txt)
    }

    /// Get hash key for this operator.
    const fn key(&self) -> u32 {
        op_key(self.bytes(), self.len())
    }

    /// Get hash of this operator, which is the slot index into [`OP_PACK`],
    /// using `mul` as the hash map multiplier.
    ///
    /// Returns a `usize` but it is guaranteed to be less than [`HASH_TABLE_SIZE`].
    const fn slot(&self, mul: u32) -> usize {
        op_slot(self.key(), mul)
    }
}

/// All 2-byte and 3-byte operators.
///
/// Only exists at build time and in tests, not referenced from any runtime code.
static OPMAP_OPS: [OpDef; 32] = [
    OpDef::new("<=", TokenKind::Le),
    OpDef::new(">=", TokenKind::Ge),
    OpDef::new("==", TokenKind::EqEq),
    OpDef::new("!=", TokenKind::BangEq),
    OpDef::new("===", TokenKind::EqEqEq),
    OpDef::new("!==", TokenKind::BangEqEq),
    OpDef::new("**", TokenKind::StarStar),
    OpDef::new("++", TokenKind::PlusPlus),
    OpDef::new("--", TokenKind::MinusMinus),
    OpDef::new("<<", TokenKind::LShift),
    OpDef::new(">>", TokenKind::RShift),
    OpDef::new(">>>", TokenKind::URShift),
    OpDef::new("&&", TokenKind::AmpAmp),
    OpDef::new("||", TokenKind::PipePipe),
    OpDef::new("??", TokenKind::Nullish),
    OpDef::new("?.", TokenKind::OptionalChain),
    OpDef::new("=>", TokenKind::Arrow),
    OpDef::new("+=", TokenKind::PlusEq),
    OpDef::new("-=", TokenKind::MinusEq),
    OpDef::new("*=", TokenKind::StarEq),
    OpDef::new("%=", TokenKind::PercentEq),
    OpDef::new("<<=", TokenKind::LShiftEq),
    OpDef::new(">>=", TokenKind::RShiftEq),
    OpDef::new("&=", TokenKind::AmpEq),
    OpDef::new("|=", TokenKind::PipeEq),
    OpDef::new("^=", TokenKind::CaretEq),
    OpDef::new("&&=", TokenKind::AmpAmpEq),
    OpDef::new("||=", TokenKind::PipePipeEq),
    OpDef::new("??=", TokenKind::NullishEq),
    OpDef::new("**=", TokenKind::StarStarEq),
    OpDef::new("...", TokenKind::Ellipsis),
    OpDef::new("/=", TokenKind::SlashEq),
];

/// Details of the single 4-byte operator (`>>>=`).
const FOUR_BYTE_OP_BYTES: [u8; 4] = *b">>>=";
const FOUR_BYTE_OP_KIND: TokenKind = TokenKind::URShiftEq;

/// Returns `true` if a byte is one of the operator characters `=!<>+-*&|^%?.`.
///
/// Note `/` is excluded - callers handle it separately.
#[inline(always)]
pub(super) const fn is_op_char(c: u8) -> bool {
    const OPCH_LO: [u8; 16] = [0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 10, 3, 7, 2];
    const OPCH_HI: [u8; 16] = [0, 0, 1, 2, 0, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];

    (OPCH_LO[(c & 15) as usize] & OPCH_HI[(c >> 4) as usize]) != 0
}

/// Operator bytes and [`TokenKind`], indexed by perfect hash slot.
///
/// Each entry contains:
/// - First 3 bytes of the operator in bottom 3 bytes (0 for 3rd byte for 2-byte operators).
/// - [`TokenKind`] in top byte.
///
/// The single 4-byte operator (`>>>=`) is not included in the table.
/// [`opmap_longest`] checks for it without consulting the table.
static OP_PACK: [u32; HASH_TABLE_SIZE] = {
    let mut op_pack = [0; HASH_TABLE_SIZE];

    let mut i = 0_usize;
    while i < OPMAP_OPS.len() {
        let op_def = &OPMAP_OPS[i];
        let slot = op_def.slot(OPMAP_MUL);

        let txt = op_def.txt;
        let bytes = if op_def.len() == 2 {
            (txt[0] as u32) | ((txt[1] as u32) << 8)
        } else {
            (txt[0] as u32) | ((txt[1] as u32) << 8) | ((txt[2] as u32) << 16)
        };
        op_pack[slot] = bytes | ((op_def.kind as u32) << 24);

        i += 1;
    }

    op_pack
};

/// Hash `key` and get the packed value for the slot in the hash table.
///
/// `key` must contain:
/// - Bytes 0-1: First 2 bytes of source
/// - Byte  2  : 3rd byte of source for a 3-byte candidate, or 0 for 2-byte candidate
/// - Byte  3  : 0
///
/// Returned value contains:
/// - Bytes 0-1: First 2 bytes of operator
/// - Byte  2  : 3rd byte of operator for a 3-byte operator, or 0 for 2-byte operator
/// - Byte  3  : [`TokenKind`] of the operator as a `u8`
///
/// If no match, returns 0 (i.e. operator bytes `\0\0\0`, `TokenKind` byte 0).
///
/// Hashmap collisions are possible, so caller must additionally check that
/// the bottom 3 bytes of `key` and the returned packed value are equal to confirm a match.
#[inline(always)]
pub(super) fn opmap_pack(key: u32) -> u32 {
    let slot = op_slot(key, OPMAP_MUL);
    OP_PACK[slot]
}

/// Check if up to 4 bytes of source contains a 2-byte, 3-byte, or 4-byte operator.
///
/// Searches for longest operator first, starting at `min(max_len, 4)` length.
///
/// `max_len` should be 2 or more to find an operator.
/// If `max_len` is < 2, and `bytes` contains padding `\0` bytes from after end of source
/// in last 3 or 4 bytes, no matching operator can be found.
///
/// If an operator is found, returns a tuple `(kind, len)` where:
/// - `kind` is the [`TokenKind`] of the operator as a `u32`
/// - `len` is the length of the found operator in bytes
///
/// If no operator is found, returns 0 as `kind`, and 1 as `len`.
///
/// `?.` followed by a digit is rejected as a match.
#[inline(always)]
pub(super) fn opmap_longest(bytes: [u8; 4], max_len: u32) -> (/* kind*/ u32, /* len */ u32) {
    if max_len >= 4 && bytes == FOUR_BYTE_OP_BYTES {
        return (FOUR_BYTE_OP_KIND as u32, 4);
    }

    if max_len >= 3 {
        let kind = opmap_lookup(bytes, 3);
        if kind != 0 {
            return (kind, 3);
        }
    }

    let kind = opmap_lookup(bytes, 2);
    if kind != 0 && !(kind == tk!(OptionalChain) as u32 && is_digit(bytes[2])) {
        return (kind, 2);
    }

    (0, 1)
}

/// Check if 2 or 3 bytes of source contain a multi-byte operator in their first `len` bytes.
///
/// * If an operator is found, returns the [`TokenKind`] of the operator as a `u32`.
/// * Otherwise, returns 0.
///
/// `len` must be 2 or 3.
#[inline(always)]
fn opmap_lookup(bytes: [u8; 4], len: u32) -> u32 {
    let key = op_key(bytes, len);
    let slot = op_slot(key, OPMAP_MUL);

    // Compare the candidate's and operator's first 3 bytes.
    //
    // - `key` has candidate's first 3 bytes in bottom 3 bytes.
    //   When `len == 2`, the 3rd byte of `key` is 0.
    // - `pack` has operator's first 3 bytes in bottom 3 bytes.
    //   For 2-byte operators, the 3rd byte of `pack` is 0.
    //
    // So when bottom 3 bytes of `key` and `pack` are the same, it's a match
    // (except for the extra check for 3-byte candidates below).
    //
    // If bottom 3 bytes of `key` are all 0, it's possible that `key` hashes to an empty slot,
    // so `pack == 0`. In that case `(pack & 0xFF_FFFF) == key` and the branch returning 0
    // is not taken. But in that case, `pack >> 24` is also 0, so 0 is returned either way.
    let pack = OP_PACK[slot];
    if (pack & 0xFF_FFFF) != key {
        return 0;
    }

    // Check a 3-byte candidate with `\0` as 3rd byte didn't match a 2-byte operator.
    // A `\0` in this position would be a syntax error, so this branch is never taken in practice.
    //
    // This function is inlined into `opmap_longest`, so `len` is statically known here.
    // Condition is shortened to `if (pack & 0xFF_0000) == 0` when `len == 3`,
    // and the branch is removed entirely when `len == 2`.
    if len == 3 && (pack & 0xFF_0000) == 0 {
        return 0;
    }

    // Return `TokenKind` as `u32`
    pack >> 24
}

/// Get hash key from `bytes` and `len`.
///
/// `len` must be 2 or 3.
#[inline(always)]
const fn op_key(bytes: [u8; 4], len: u32) -> u32 {
    let mask = if len == 3 { 0xFF_FFFF } else { 0xFFFF };
    u32::from_le_bytes(bytes) & mask
}

/// Get hash of `key`, which is the slot index into [`OP_PACK`],
/// using `mul` as the hash map multiplier.
///
/// Returns a `usize` but it is guaranteed to be less than [`HASH_TABLE_SIZE`].
#[inline(always)]
const fn op_slot(key: u32, mul: u32) -> usize {
    (key.wrapping_mul(mul) >> (32 - HASH_BITS)) as usize
}

/// Get a `[u8; 4]` containing all the bytes of `txt` and 0 for any remaining bytes.
const fn first_4_bytes(txt: &[u8]) -> [u8; 4] {
    let mut bytes = [0; 4];
    let mut i = 0;
    while i < txt.len() {
        bytes[i] = txt[i];
        i += 1;
    }
    bytes
}

#[cfg(test)]
mod tests {
    use crate::token::{OP_KIND_BASE, OP_KIND_MAX};

    use super::*;

    #[test]
    fn test_is_op_char() {
        const OPCHARS: &[u8] = b"=!<>+-*&|^%?.";

        let mut in_set = [false; 256];
        for &q in OPCHARS {
            in_set[q as usize] = true;
        }

        for c in 0..256usize {
            assert!(is_op_char(c as u8) == in_set[c], "OPCH_LO/HI wrong at byte {c:#04x}");
        }
    }

    #[test]
    fn test_op_defs_token_kind_range() {
        for (i, op_def) in OPMAP_OPS.iter().enumerate() {
            let kind = op_def.kind as u8;
            assert!(kind >= OP_KIND_BASE && kind <= OP_KIND_MAX, "OpDef {i}: `kind` out of range");
        }
    }

    #[test]
    fn test_op_defs_unique_token_kinds() {
        let mut seen = [false; 256];
        for op_def in &OPMAP_OPS {
            let kind = op_def.kind;
            assert!(!seen[kind as usize], "duplicate `TokenKind`: {kind}");
            seen[kind as usize] = true;
        }
    }

    #[test]
    fn test_op_defs_no_key_collisions() {
        for (index1, op_def1) in OPMAP_OPS.iter().enumerate() {
            for (index2, op_def2) in OPMAP_OPS.iter().enumerate().skip(index1 + 1) {
                assert!(op_def1.key() != op_def2.key(), "OpDef {index1}, {index2}: key collision");
            }
        }
    }

    #[test]
    fn test_opmap_lookup_correct_kinds() {
        for (i, op_def) in OPMAP_OPS.iter().enumerate() {
            let lookup_kind = opmap_lookup(op_def.bytes(), op_def.len());
            assert!(lookup_kind == op_def.kind as u32, "OpDef {i}: `opmap_lookup` wrong kind");
        }
    }

    #[test]
    fn test_opmap_lookup_returns_zero_on_no_match() {
        let cases = [
            // Not operators
            "..", "=/", "<<<", "&&&", "?..",
            // 3-byte candidate whose 1st 2 bytes are a 2-byte operator, and 3rd byte is `\0`.
            // `key` is identical for these and the 2-byte operators, so they hash to the same slot.
            // `opmap_lookup` needs to ensure they aren't misidentified as matching.
            "==\0", "<<\0", "||\0",
            // All bytes are 0, so `key` is 0. The check against `pack` passes if `key` hashes
            // to an empty slot. `opmap_lookup` must still return 0.
            "\0\0", "\0\0\0",
        ];
        for txt in cases {
            let kind = opmap_lookup(first_4_bytes(txt.as_bytes()), txt.len() as u32);
            assert!(kind == 0, "`opmap_lookup` should return 0 for {txt:?}");
        }
    }

    #[test]
    fn test_perfect_hash() {
        // If `OPMAP_MUL` produces no collisions, all good
        if is_collision_free(OPMAP_MUL) {
            return;
        }

        // There was a collision - find a new value for `OPMAP_MUL` which has no collisions
        let mut mul = (1u32 << 24) | 1;
        while mul < (1u32 << 28) {
            if is_collision_free(mul) {
                panic!(
                    "Current value for `OPMAP_MUL` produces collisions. Set it to 0x{:04X}_{:04X}.",
                    mul >> 16,
                    mul & 0xFFFF
                );
            }
            mul += 2;
        }

        panic!("Current value for `OPMAP_MUL` produces collisions. Could not find another value.");
    }

    fn is_collision_free(mul: u32) -> bool {
        let mut used = [false; HASH_TABLE_SIZE];
        for op_def in &OPMAP_OPS {
            let slot = op_def.slot(mul);
            if used[slot] {
                return false;
            }
            used[slot] = true;
        }
        true
    }
}
