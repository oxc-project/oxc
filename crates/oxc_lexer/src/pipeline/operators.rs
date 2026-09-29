//! Perfect hash table mapping multi-byte operators (e.g. `===`, `>>>=`) to their [`TokenKind`].
//!
//! A candidate is the next 4 bytes of source plus a length to try (2, 3 or 4).
//!
//! [`op_key`] packs it into a `u32` key:
//! - Bottom 3 bytes: The candidate's first 3 bytes, with the 3rd zeroed if the length is 2.
//! - Top byte: The length.
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
//! So a lookup is a multiply, a load, and a comparison of the bottom 3 bytes of key and slot value.
//! If they match, the top byte of the slot value is the [`TokenKind`].
//! [`opmap_lookup`] does the whole lookup.
//! [`opmap_pack`] returns the slot value, for callers which do the comparison themselves.
//!
//! Lengths are never compared. Instead, the table is built so that no candidate lands on the slot
//! of an operator of a different length whose bottom 3 bytes match (e.g. `====` vs `===`).
//! See the comments in [`opmap_lookup`] and `is_collision_free` in tests for how each case is ruled out.
//!
//! The key has no room for a 4th byte. `>>>=` is the only 4-byte operator,
//! so `opmap_lookup` compares its last byte separately, against [`FOUR_BYTE_OP_LAST_BYTE`].
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
const OPMAP_MUL: u32 = 0x0217_DFE7;

/// An operator and its corresponding [`TokenKind`].
struct OpDef {
    txt: &'static [u8],
    kind: TokenKind,
}

impl OpDef {
    /// Create new [`OpDef`].
    const fn new(txt: &'static str, kind: TokenKind) -> Self {
        assert!(txt.len() >= 2 && txt.len() <= 4, "`txt` must be between 2 and 4 bytes long");
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

/// All multi-byte operators.
///
/// Only exists at build time and in tests, not referenced from any runtime code.
static OPMAP_OPS: [OpDef; 33] = [
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
    OpDef::new(">>>=", TokenKind::URShiftEq),
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
/// Only the first 3 bytes of the operator are stored, as the 4th byte contains the `TokenKind`.
/// `opmap_lookup` compares the last byte of a 4-byte candidate separately.
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

/// Last byte of the single 4-byte operator (`>>>=`).
///
/// `OP_PACK` stores only an operator's first 3 bytes, so this is the only byte of a 4-byte
/// candidate that `opmap_lookup` cannot compare via `OP_PACK`.
const FOUR_BYTE_OP_LAST_BYTE: u8 = {
    let mut last_byte = None;
    let mut i = 0_usize;
    while i < OPMAP_OPS.len() {
        let op_def = &OPMAP_OPS[i];
        if op_def.len() == 4 {
            assert!(last_byte.is_none(), "more than one 4-byte operator");
            last_byte = Some(op_def.txt[3]);
        }
        i += 1;
    }
    last_byte.expect("no 4-byte operator found")
};

/// Hash `key` and get the packed value for the slot in the hash table.
///
/// `key` must contain:
/// - 3 bytes of source in bottom 3 bytes.
/// - Length of operator checking for in top byte (2 or 3).
///
/// Returned value contains:
/// - First 3 bytes of matching operator in bottom 3 bytes.
/// - [`TokenKind`] of the operator in top byte as a `u8`.
///
/// If no match, returns 0 (i.e. operator bytes `\0\0\0`, `TokenKind` byte 0).
///
/// Hashmap collisions are possible, so caller must additionally check that
/// the first 3 bytes of `key` and the returned packed value match to confirm a match.
#[inline(always)]
pub(super) fn opmap_pack(key: u32) -> u32 {
    let slot = op_slot(key, OPMAP_MUL);
    OP_PACK[slot]
}

/// Check if up to 4 bytes of source contains a 2-byte, 3-byte, or 4-byte operator.
///
/// Searches for longest operator first, starting at `max_len` length.
///
/// `max_len` must be 2, 3, or 4.
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
    let mut len = max_len;
    while len >= 2 {
        let kind = opmap_lookup(bytes, len);
        if kind != 0 && !(kind == tk!(OptionalChain) as u32 && is_digit(bytes[2])) {
            return (kind, len);
        }
        len -= 1;
    }
    (0, 1)
}

/// Check if 4 bytes of source contain a multi-byte operator in their first `len` bytes.
///
/// * If an operator is found, returns the [`TokenKind`] of the operator as a `u32`.
/// * Otherwise, returns 0.
///
/// `len` must be between 2 and 4 (inclusive).
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
    // (except for the extra check for `len == 4` below).
    //
    // If bottom 3 bytes of `key` are all 0, it's possible that `key` hashes to an empty slot,
    // so `pack == 0`. In that case `((pack ^ key) & 0xFF_FFFF) == 0` and the branch returning 0
    // is not taken. But in that case, `pack >> 24` is also 0, so 0 is returned either way.
    //
    // Lengths need no comparison, due to the construction of the hash table:
    //
    // - Every operator has a different `slot`.
    //
    // - A 3-byte or 4-byte candidate with 3rd byte == 0 has `key` with 3rd byte == 0.
    //   Bottom 3 bytes of `key` could be same as bottom 3 bytes of `OP_PACK` entry
    //   for the 2-byte operator with same first 2 bytes
    //   e.g. `==\0` candidate vs `==` operator.
    //   Hash table ensures these produce different `slot` values, so the check below fails.
    //
    // - A 4-byte candidate has `key` with bottom 3 bytes being the first 3 bytes of candidate.
    //   Bottom 3 bytes of `key` could be same as bottom 3 bytes of `OP_PACK` entry
    //   for the 3-byte operator with same first 3 bytes
    //   e.g. `====` candidate vs `===` operator.
    //   Hash table ensures these produce different `slot` values, so the check below fails.
    //
    // See `is_collision_free` in tests below.
    let pack = OP_PACK[slot];
    if ((pack ^ key) & 0xFF_FFFF) != 0 {
        return 0;
    }

    // `pack` holds only the first 3 bytes of an operator, so the check above misses the last byte
    // of a 4-byte operator. Compare that 4th byte here. After the check above, if `len == 4`,
    // the first 3 bytes of source are `>>>`. `>>>` and `>>>=` are rarely used, so this branch
    // is almost never taken - very predictable.
    if len == 4 && bytes[3] != FOUR_BYTE_OP_LAST_BYTE {
        return 0;
    }

    // Return `TokenKind` as `u32`
    pack >> 24
}

/// Get hash key from `bytes` and `len`.
///
/// `len` must be between 2 and 4 (inclusive).
#[inline(always)]
const fn op_key(bytes: [u8; 4], len: u32) -> u32 {
    let mut key = u32::from_le_bytes(bytes);
    key &= if len >= 3 { 0xFF_FFFF } else { 0xFFFF };
    key |= len << 24;
    key
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

    // `is_collision_free` has no check for 4-byte operators, because the only 3-byte candidate
    // which could collide with `>>>=` is `>>>`, which is an operator itself, so it hashes to
    // `>>>`'s own slot. If the 4-byte operator's first 3 bytes were not also an operator,
    // such a candidate could hash to the 4-byte operator's slot instead. `opmap_lookup` compares
    // only the bottom 3 bytes, so it would match, and the `len == 4` check does not run for
    // a 3-byte candidate - so `>>>` would be lexed as `>>>=`.
    #[test]
    fn test_op_defs_4_byte_operator_has_corresponding_3_byte_op() {
        let op4 = OPMAP_OPS.iter().find(|op_def| op_def.len() == 4).unwrap();
        let first_3_bytes = &op4.txt[..3];
        assert!(OPMAP_OPS.iter().any(|op_def| op_def.txt == first_3_bytes));
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
            "..", "=/", "<<<", "&&&", "?..", ">>>>", "<<<=", "++++",
            // 3-byte or 4-byte candidate whose 1st 2 bytes are a 2-byte operator, and 3rd byte is `\0`.
            // Bottom 3 bytes of `key` are the same as bottom 3 bytes of that operator's entry in `OP_PACK`,
            // so only hashing to a different slot prevents a false match.
            "==\0", "<<\0", "||\0", "==\0=", "<<\0=", "||\0=",
            // 4-byte candidate whose 1st 3 bytes are a 3-byte operator, and last is `=`.
            // Again bottom 3 bytes of `key` are the same as that operator's entry in `OP_PACK`.
            "====", "<<==", "...=",
            // 1st 3 bytes are 0, so bottom 3 bytes of `key` are 0, same as an empty slot.
            // The check against `pack` passes if `key` hashes to an empty slot,
            // and 0 is returned by `pack >> 24` instead.
            "\0\0", "\0\0\0", "\0\0\0=",
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
            // Ensure all operators hash to different slots
            let slot = op_def.slot(mul);
            if used[slot] {
                return false;
            }
            used[slot] = true;

            // `opmap_lookup` does not compare lengths, and relies on lack of collisions
            // between similar candidates to avoid false positives.
            //
            // 4-byte operators need no check here - the only candidate which could collide with
            // `>>>=` is `>>>`, and `>>>` is an operator itself, so it lands on `>>>`'s own slot,
            // which the check above keeps different from `>>>=`'s slot.
            let bytes = op_def.bytes();
            if op_def.len() == 3 {
                // A 4-byte candidate with first 3 bytes same as this 3-byte operator
                // must not land on this operator's slot.
                // `====` must not hash the same as `===`.
                if op_slot(op_key(bytes, 4), mul) == slot {
                    return false;
                }
            } else if op_def.len() == 2 {
                // A 3-byte or 4-byte candidate whose first 2 bytes are same as this operator,
                // and 3rd byte is `\0`, must not land on this operator's slot

                // `==\0` must not hash the same as `==`
                if op_slot(op_key([bytes[0], bytes[1], 0, 0], 3), mul) == slot {
                    return false;
                }

                // `==\0=` must not hash the same as `==`
                if op_slot(op_key([bytes[0], bytes[1], 0, 0], 4), mul) == slot {
                    return false;
                }
            }
        }
        true
    }
}
