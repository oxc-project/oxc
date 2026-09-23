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
    fn key(&self) -> u32 {
        let txt = self.txt;
        let c2 = if self.len >= 3 { txt[2] } else { 0 };
        op_key(txt[0], txt[1], c2, self.len as u32)
    }

    #[inline(always)]
    fn slot(&self, mul: u32) -> usize {
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

pub struct OpMap {
    pub opmap_mul: u32,
    pub opmap_slot: [u8; 256],
    pub op2_pack: [u32; 256],
    pub op3_pack: [u64; 256],
}

impl OpMap {
    pub(super) fn new() -> OpMap {
        let mut m =
            OpMap { opmap_mul: 0, opmap_slot: [0xFF; 256], op2_pack: [0; 256], op3_pack: [0; 256] };
        m.opmap_init();
        m.build_op_pack();
        m
    }

    fn opmap_init(&mut self) {
        let mut m: u64 = (1u64 << 24) | 1;
        while m < (1u64 << 28) {
            let mut used = [0u8; 256];
            let mut ok = true;
            for i in 0..OPMAP_OPS.len() {
                let o = &OPMAP_OPS[i];
                let slot = o.slot(m as u32);
                if used[slot] != 0 {
                    ok = false;
                    break;
                }
                used[slot] = 1;
            }
            if ok {
                self.opmap_mul = m as u32;
                self.opmap_slot = [0xFF; 256];
                for i in 0..OPMAP_OPS.len() {
                    let o = &OPMAP_OPS[i];
                    let slot = o.slot(self.opmap_mul);
                    self.opmap_slot[slot] = i as u8;
                }
                return;
            }
            m += 2;
        }
        panic!("opmap perfect-hash search FAILED");
    }

    fn build_op_pack(&mut self) {
        self.op2_pack = [0; 256];
        self.op3_pack = [0; 256];
        for i in 0..OPMAP_OPS.len() {
            let o = &OPMAP_OPS[i];
            let h = o.slot(self.opmap_mul);
            if o.len == 2 {
                self.op2_pack[h] = 2u32
                    | ((o.txt[0] as u32) << 8)
                    | ((o.txt[1] as u32) << 16)
                    | ((o.kind as u32) << 24);
            } else if o.len == 3 {
                self.op3_pack[h] = 3u64
                    | ((o.txt[0] as u64) << 8)
                    | ((o.txt[1] as u64) << 16)
                    | ((o.txt[2] as u64) << 24)
                    | ((o.kind as u64) << 32);
            }
        }
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
fn op_key(c0: u8, c1: u8, c2: u8, len: u32) -> u32 {
    (c0 as u32) | ((c1 as u32) << 8) | ((c2 as u32) << 16) | (len << 24)
}

#[inline(always)]
fn op_slot(key: u32, mul: u32) -> usize {
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
}
