use crate::token::TokenKind;

struct OpDef {
    pub txt: &'static [u8],
    pub len: u8,
    pub kind: TokenKind,
}

impl OpDef {
    const fn new(txt: &'static str, kind: TokenKind) -> Self {
        Self { txt: txt.as_bytes(), len: txt.len() as u8, kind }
    }

    const fn key(&self) -> u32 {
        let txt = self.txt;
        let c2 = if self.len >= 3 { txt[2] } else { 0 };
        op_key(txt[0], txt[1], c2, self.len as u32)
    }

    const fn slot(&self, mul: u32) -> usize {
        op_slot(self.key(), mul)
    }
}

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

const OPCH_LO: [u8; 16] = [0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 10, 3, 7, 2];
const OPCH_HI: [u8; 16] = [0, 0, 1, 2, 0, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];

#[inline(always)]
pub const fn is_op_char(c: u8) -> bool {
    (OPCH_LO[(c & 15) as usize] & OPCH_HI[(c >> 4) as usize]) != 0
}

/// Multiplier for the operator perfect hash.
const OPMAP_MUL: u32 = 0x0101_0749;

/// Operator bytes and [`TokenKind`], indexed by perfect hash slot.
///
/// Only the first 3 bytes of the operator are stored, as the 4th byte contains the `TokenKind`.
/// `opmap_lookup` compares the last byte of a 4-byte candidate separately.
static OP_PACK: [u32; 256] = {
    let mut op_pack = [0; 256];

    let mut i = 0_usize;
    while i < OPMAP_OPS.len() {
        let op_def = &OPMAP_OPS[i];
        let slot = op_def.slot(OPMAP_MUL);

        let txt = op_def.txt;
        let bytes = if op_def.len == 2 {
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
        if op_def.len == 4 {
            assert!(last_byte.is_none(), "more than one 4-byte operator");
            last_byte = Some(op_def.txt[3]);
        }
        i += 1;
    }
    last_byte.expect("no 4-byte operator found")
};

pub struct OpMap {
    pub opmap_mul: u32,
    pub op_pack: [u32; 256],
}

impl OpMap {
    /// Create an [`OpMap`].
    pub(super) fn new() -> OpMap {
        Self { opmap_mul: OPMAP_MUL, op_pack: OP_PACK }
    }

    /// Check if 4 bytes of source contain a multi-byte operator in their first `len` bytes.
    ///
    /// * If an operator is found, returns the [`TokenKind`] of the operator as a `u32`.
    /// * Otherwise, returns 0.
    ///
    /// `len` must be between 2 and 4 (inclusive).
    #[inline(always)]
    pub fn opmap_lookup(&self, b0: u8, b1: u8, b2: u8, b3: u8, len: u32) -> u32 {
        let c2 = if len >= 3 { b2 } else { 0 };
        let key = op_key(b0, b1, c2, len);
        let slot = op_slot(key, self.opmap_mul);

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
        // If `b0`, `b1`, and `c2` are all 0, then it's possible that `key` hashes to an empty slot,
        // so `pack == 0`. In that case `((pack ^ key) & 0xFF_FFFF) == 0` and the branch returning 0
        // is not taken. But in that case, `pack >> 24` is also 0, so 0 is returned either way.
        //
        // Lengths need no comparison, due to the construction of the hash table:
        //
        // - Every operator has a different `slot`.
        //
        // - A 3-byte or 4-byte candidate with 3rd byte == 0 has `key` with 3rd byte == 0.
        //   Bottom 3 bytes of `key` could be same as bottom 3 bytes of `op_pack` entry
        //   for the 2-byte operator with same first 2 bytes
        //   e.g. `==\0` candidate vs `==` operator.
        //   Hash table ensures these produce different `slot` values, so the check below fails.
        //
        // - A 4-byte candidate has `key` with bottom 3 bytes being the first 3 bytes of candidate.
        //   Bottom 3 bytes of `key` could be same as bottom 3 bytes of `op_pack` entry
        //   for the 3-byte operator with same first 3 bytes
        //   e.g. `====` candidate vs `===` operator.
        //   Hash table ensures these produce different `slot` values, so the check below fails.
        //
        // See `is_collision_free` in tests below.
        let pack = self.op_pack[slot];
        if ((pack ^ key) & 0xFF_FFFF) != 0 {
            return 0;
        }

        // `pack` holds only the first 3 bytes of an operator, so the check above misses the last byte
        // of a 4-byte operator. Compare that 4th byte here. After the check above, if `len == 4`,
        // the first 3 bytes of source are `>>>`. `>>>` and `>>>=` are rarely used, so this branch
        // is almost never taken - very predictable.
        if len == 4 && b3 != FOUR_BYTE_OP_LAST_BYTE {
            return 0;
        }

        // Return `TokenKind` as `u32`
        pack >> 24
    }
}

#[inline(always)]
const fn op_key(c0: u8, c1: u8, c2: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | ((c2 as u32) << 16) | (len << 24)
}

#[inline(always)]
const fn op_slot(key: u32, mul: u32) -> usize {
    (key.wrapping_mul(mul) >> 24) as usize
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
    fn test_op_defs_length() {
        for (i, op_def) in OPMAP_OPS.iter().enumerate() {
            let len = op_def.txt.len();
            assert!(len >= 2 && len <= 4, "OpDef {i}: length out of range");
            assert!(op_def.len as usize == len, "OpDef {i}: `len` and `txt.len()` do not match");
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
        let op4 = OPMAP_OPS.iter().find(|op_def| op_def.len == 4).unwrap();
        let first_3_bytes = &op4.txt[..3];
        assert!(OPMAP_OPS.iter().any(|op_def| op_def.txt == first_3_bytes));
    }

    #[test]
    fn test_opmap_lookup_correct_kinds() {
        let opmap = OpMap::new();

        for (i, op_def) in OPMAP_OPS.iter().enumerate() {
            let txt = op_def.txt;
            let lookup_kind = opmap.opmap_lookup(
                txt[0],
                txt[1],
                *txt.get(2).unwrap_or(&0),
                *txt.get(3).unwrap_or(&0),
                op_def.len as u32,
            );
            assert!(lookup_kind == op_def.kind as u32, "OpDef {i}: `opmap_lookup` wrong kind");
        }
    }

    #[test]
    fn test_opmap_lookup_returns_zero_on_no_match() {
        let opmap = OpMap::new();

        let cases = [
            // Not operators
            "..", "=/", "<<<", "&&&", "?..", ">>>>", "<<<=", "++++",
            // 3-byte or 4-byte candidate whose 1st 2 bytes are a 2-byte operator, and 3rd byte is `\0`.
            // Bottom 3 bytes of `key` are the same as bottom 3 bytes of that operator's entry in `op_pack`,
            // so only hashing to a different slot prevents a false match.
            "==\0", "<<\0", "||\0", "==\0=", "<<\0=", "||\0=",
            // 4-byte candidate whose 1st 3 bytes are a 3-byte operator, and last is `=`.
            // Again bottom 3 bytes of `key` are the same as that operator's entry in `op_pack`.
            "====", "<<==", "...=",
            // 1st 3 bytes are 0, so bottom 3 bytes of `key` are 0, same as an empty slot.
            // The check against `pack` passes if `key` hashes to an empty slot,
            // and 0 is returned by `pack >> 24` instead.
            "\0\0", "\0\0\0", "\0\0\0=",
        ];
        for txt in cases {
            let bytes = txt.as_bytes();
            let kind = opmap.opmap_lookup(
                bytes[0],
                bytes[1],
                *bytes.get(2).unwrap_or(&0),
                *bytes.get(3).unwrap_or(&0),
                bytes.len() as u32,
            );
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
        let mut used = [false; 256];
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
            let txt = op_def.txt;
            if op_def.len == 3 {
                // A 4-byte candidate with first 3 bytes same as this 3-byte operator
                // must not land on this operator's slot.
                // `====` must not hash the same as `===`.
                if op_slot(op_key(txt[0], txt[1], txt[2], 4), mul) == slot {
                    return false;
                }
            } else if op_def.len == 2 {
                // A 3-byte or 4-byte candidate whose first 2 bytes are same as this operator,
                // and 3rd byte is `\0`, must not land on this operator's slot

                // `==\0` must not hash the same as `==`
                if op_slot(op_key(txt[0], txt[1], 0, 3), mul) == slot {
                    return false;
                }

                // `==\0=` must not hash the same as `==`
                if op_slot(op_key(txt[0], txt[1], 0, 4), mul) == slot {
                    return false;
                }
            }
        }
        true
    }
}
