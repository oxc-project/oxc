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

    #[inline(always)]
    const fn key(&self) -> u32 {
        let txt = self.txt;
        let c2 = if self.len >= 3 { txt[2] } else { 0 };
        op_key(txt[0], txt[1], c2, self.len as u32)
    }

    #[inline(always)]
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

static OPMAP_SLOT: [u8; 256] = {
    let mut slots = [0xFF; 256];

    let mut i = 0_usize;
    while i < OPMAP_OPS.len() {
        let op_def = &OPMAP_OPS[i];
        let slot = op_def.slot(OPMAP_MUL);
        slots[slot] = i as u8;
        i += 1;
    }

    slots
};

pub struct OpMap {
    pub opmap_mul: u32,
    pub opmap_slot: [u8; 256],
    pub op2_pack: [u32; 256],
    pub op3_pack: [u64; 256],
}

impl OpMap {
    /// Create an [`OpMap`].
    pub(super) fn new() -> OpMap {
        let mut op2_pack = [0; 256];
        let mut op3_pack = [0; 256];

        for op_def in &OPMAP_OPS {
            let slot = op_def.slot(OPMAP_MUL);
            if op_def.len == 2 {
                op2_pack[slot] = 2u32
                    | ((op_def.txt[0] as u32) << 8)
                    | ((op_def.txt[1] as u32) << 16)
                    | ((op_def.kind as u32) << 24);
            } else if op_def.len == 3 {
                op3_pack[slot] = 3u64
                    | ((op_def.txt[0] as u64) << 8)
                    | ((op_def.txt[1] as u64) << 16)
                    | ((op_def.txt[2] as u64) << 24)
                    | ((op_def.kind as u64) << 32);
            }
        }

        Self { opmap_mul: OPMAP_MUL, opmap_slot: OPMAP_SLOT, op2_pack, op3_pack }
    }

    #[inline(always)]
    pub fn opmap_lookup(&self, b0: u8, b1: u8, b2: u8, b3: u8, len: u32) -> u32 {
        let c2 = if len >= 3 { b2 } else { 0 };
        let key = op_key(b0, b1, c2, len);
        let slot = op_slot(key, self.opmap_mul);
        let idx = self.opmap_slot[slot];
        if idx == 0xFF {
            return 0;
        }
        let o = &OPMAP_OPS[idx as usize];
        if o.len as u32 != len {
            return 0;
        }
        if o.txt[0] != b0 || o.txt[1] != b1 {
            return 0;
        }
        if len >= 3 && o.txt[2] != b2 {
            return 0;
        }
        if len >= 4 && o.txt[3] != b3 {
            return 0;
        }
        o.kind as u32
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

        let cases = ["..", "=/"];
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
            let slot = op_def.slot(mul);
            if used[slot] {
                return false;
            }
            used[slot] = true;
        }
        true
    }
}
