use crate::token::TokenKind;

/// Single-byte punctuator and its [`TokenKind`].
#[cfg_attr(
    all(
        not(test),
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    expect(dead_code, reason = "only used in scalar implementation and tests")
)]
pub struct Punct1 {
    pub byte: u8,
    pub kind: TokenKind,
}

impl Punct1 {
    const fn new(c: char, kind: TokenKind) -> Self {
        assert!(c.is_ascii());
        Self { byte: c as u8, kind }
    }
}

/// Single-byte punctuators and their [`TokenKind`]s.
/// `#` maps to `Invalid` - a bare `#` is invalid on its own
/// (private names and hashbangs are resolved earlier).
#[cfg_attr(
    all(
        not(test),
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    expect(dead_code, reason = "only used in scalar implementation and tests")
)]
pub const PUNCT1: [Punct1; 26] = [
    Punct1::new('(', TokenKind::LParen),
    Punct1::new(')', TokenKind::RParen),
    Punct1::new('[', TokenKind::LBracket),
    Punct1::new(']', TokenKind::RBracket),
    Punct1::new('{', TokenKind::LBrace),
    Punct1::new('}', TokenKind::RBrace),
    Punct1::new(';', TokenKind::Semi),
    Punct1::new(',', TokenKind::Comma),
    Punct1::new('.', TokenKind::Dot),
    Punct1::new('<', TokenKind::Lt),
    Punct1::new('>', TokenKind::Gt),
    Punct1::new('+', TokenKind::Plus),
    Punct1::new('-', TokenKind::Minus),
    Punct1::new('*', TokenKind::Star),
    Punct1::new('/', TokenKind::Slash),
    Punct1::new('%', TokenKind::Percent),
    Punct1::new('&', TokenKind::Amp),
    Punct1::new('|', TokenKind::Pipe),
    Punct1::new('^', TokenKind::Caret),
    Punct1::new('!', TokenKind::Bang),
    Punct1::new('~', TokenKind::Tilde),
    Punct1::new('?', TokenKind::Question),
    Punct1::new(':', TokenKind::Colon),
    Punct1::new('=', TokenKind::Eq),
    Punct1::new('@', TokenKind::At),
    Punct1::new('#', TokenKind::Invalid),
];

#[cfg_attr(
    all(
        not(test),
        not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "popcnt"
        ))
    ),
    expect(dead_code, reason = "only used in SIMD implementation and tests")
)]
pub mod punct1_luts {
    pub const PH_A: [u8; 16] = [4, 13, 19, 20, 0, 14, 7, 8, 10, 26, 22, 0, 29, 23, 3, 2];
    pub const PH_B: [u8; 16] = [24, 26, 2, 16, 31, 25, 19, 30, 0, 0, 0, 0, 0, 0, 0, 0];
    pub const PH_T0: [u8; 16] = [68, 38, 58, 76, 255, 72, 42, 52, 34, 33, 255, 255, 70, 48, 37, 55];
    pub const PH_T1: [u8; 16] =
        [40, 255, 43, 50, 64, 61, 255, 255, 35, 36, 80, 89, 255, 82, 32, 41];
}

#[cfg(test)]
mod tests {
    use crate::{
        pipeline::bytes::{is_word, is_ws},
        token::tk,
    };

    use super::*;

    #[test]
    fn test_punct1_hash() {
        let mut punct1_ord = [tk!(Invalid); 256];
        for i in 0..PUNCT1.len() {
            punct1_ord[PUNCT1[i].byte as usize] = PUNCT1[i].kind as u8;
        }

        for c in 0..256usize {
            let cb = c as u8;
            if is_word(cb) || is_ws(cb) {
                continue;
            }
            assert!(punct1_hash(cb) == punct1_ord[c], "PH_A/B/T wrong at byte {c:#04x}");
        }
    }

    fn punct1_hash(c: u8) -> u8 {
        use punct1_luts::{PH_A, PH_B, PH_T0, PH_T1};

        if c < 0x20 {
            return tk!(Invalid);
        }
        let h = (PH_A[(c & 15) as usize] ^ PH_B[((c >> 4) & 15) as usize]) & 31;
        if h < 16 { PH_T0[h as usize] } else { PH_T1[(h & 15) as usize] }
    }
}
