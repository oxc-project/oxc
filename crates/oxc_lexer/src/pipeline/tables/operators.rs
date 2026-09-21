use crate::token::{OP_KIND_BASE, OP_KIND_MAX, TokenKind, tk};

use super::punct1::PUNCT1;

struct OpDef {
    pub txt: &'static [u8],
    pub len: u8,
    pub kind: TokenKind,
}

impl OpDef {
    const fn new(txt: &'static str, kind: TokenKind) -> Self {
        assert!(txt.len() <= 255);
        Self { txt: txt.as_bytes(), len: txt.len() as u8, kind }
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
    pub punct1_ord: [u8; 256],
}

impl OpMap {
    pub(super) fn new() -> OpMap {
        let mut m = OpMap {
            opmap_mul: 0,
            opmap_slot: [0xFF; 256],
            op2_pack: [0; 256],
            op3_pack: [0; 256],
            punct1_ord: [tk!(Invalid); 256],
        };
        m.opmap_init();
        m.build_op_pack();
        m.punct1_init();
        m
    }

    fn opmap_init(&mut self) {
        for i in 0..OPMAP_OPS.len() {
            let a = &OPMAP_OPS[i];
            assert!(
                a.len as usize == a.txt.len()
                    && a.kind as u8 >= OP_KIND_BASE
                    && a.kind as u8 <= OP_KIND_MAX,
                "bad OpDef {i}"
            );
            for j in (i + 1)..OPMAP_OPS.len() {
                let b = &OPMAP_OPS[j];
                let a2 = if a.len >= 3 { a.txt[2] } else { 0 };
                let b2 = if b.len >= 3 { b.txt[2] } else { 0 };
                assert!(
                    !(a.len == b.len && a.txt[0] == b.txt[0] && a.txt[1] == b.txt[1] && a2 == b2),
                    "(c0,c1,c2,len) collision"
                );
            }
        }
        let mut m: u64 = (1u64 << 24) | 1;
        while m < (1u64 << 28) {
            let mut used = [0u8; 256];
            let mut ok = true;
            for i in 0..OPMAP_OPS.len() {
                let o = &OPMAP_OPS[i];
                let c2 = if o.len >= 3 { o.txt[2] } else { 0 };
                let key = op_key(o.txt[0], o.txt[1], c2, o.len as u32);
                let slot = (key.wrapping_mul(m as u32) >> 24) as usize;
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
                    let c2 = if o.len >= 3 { o.txt[2] } else { 0 };
                    let key = op_key(o.txt[0], o.txt[1], c2, o.len as u32);
                    let slot = (key.wrapping_mul(self.opmap_mul) >> 24) as usize;
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
            let c2 = if o.len >= 3 { o.txt[2] } else { 0 };
            let key = op_key(o.txt[0], o.txt[1], c2, o.len as u32);
            let h = (key.wrapping_mul(self.opmap_mul) >> 24) as usize;
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

    fn punct1_init(&mut self) {
        self.punct1_ord = [tk!(Invalid); 256];
        for i in 0..PUNCT1.len() {
            self.punct1_ord[PUNCT1[i].byte as usize] = PUNCT1[i].kind as u8;
        }
    }

    #[inline(always)]
    pub fn opmap_lookup(&self, b0: u8, b1: u8, b2: u8, b3: u8, len: u32) -> u32 {
        let c2 = if len >= 3 { b2 } else { 0 };
        let key = op_key(b0, b1, c2, len);
        let idx = self.opmap_slot[(key.wrapping_mul(self.opmap_mul) >> 24) as usize];
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

#[cfg(test)]
mod tests {
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
    fn test_opmap() {
        let opmap = OpMap::new();

        let mut seen_kind = [0u8; 256];
        for i in 0..OPMAP_OPS.len() {
            let o = &OPMAP_OPS[i];
            let mut b = [0u8; 4];
            b[..o.len as usize].copy_from_slice(&o.txt[..o.len as usize]);
            assert!(
                opmap.opmap_lookup(b[0], b[1], b[2], b[3], o.len as u32) == o.kind as u32,
                "opmap_lookup wrong"
            );
            assert!(seen_kind[o.kind as usize] == 0, "duplicate ordinal");
            seen_kind[o.kind as usize] = 1;
        }

        assert!(
            opmap.opmap_lookup(b'.', b'.', 0, 0, 2) == 0
                && opmap.opmap_lookup(b'<', b'<', 0, 0, 2) == tk!(LShift) as u32
                && opmap.opmap_lookup(b'<', b'=', 0, 0, 2) == tk!(Le) as u32
                && opmap.opmap_lookup(b'>', b'>', b'>', 0, 3) == tk!(URShift) as u32
                && opmap.opmap_lookup(b'>', b'>', b'=', 0, 3) == tk!(RShiftEq) as u32
                && opmap.opmap_lookup(b'=', b'=', 0, 0, 2) == tk!(EqEq) as u32
                && opmap.opmap_lookup(b'=', b'/', 0, 0, 2) == 0,
            "op spot-checks failed"
        );
    }

    #[test]
    fn test_punct1_ord() {
        let punct1_ord = &OpMap::new().punct1_ord;

        let mut seen = [0u8; 256];
        for i in 0..PUNCT1.len() {
            let ord = punct1_ord[PUNCT1[i].byte as usize];
            assert!(
                ord == PUNCT1[i].kind as u8 && seen[ord as usize] == 0,
                "PUNCT1 ordinal wrong/dup"
            );
            seen[ord as usize] = 1;
        }

        for b in 0..256usize {
            let ord = punct1_ord[b];
            let is_known = PUNCT1.iter().any(|p| p.byte == b as u8);
            assert!(is_known || ord == tk!(Invalid), "PUNCT1_ORD should be unknown");
        }

        for byte in [b'#', b'a', b'"', b'`', b'\\', b'$', b' ', 0] {
            assert!(punct1_ord[byte as usize] == tk!(Invalid));
        }
    }
}
