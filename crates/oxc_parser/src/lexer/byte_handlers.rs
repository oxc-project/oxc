use crate::diagnostics;

use super::{Kind, Lexer};

impl Lexer<'_> {
    /// Handle next byte of source.
    ///
    /// # SAFETY
    ///
    /// * Lexer must not be at end of file.
    /// * `byte` must be next byte of source code, corresponding to current position of `lexer.source`.
    /// * Only `BYTE_HANDLERS` for ASCII characters may use the `ascii_byte_handler!()` macro.
    pub(super) unsafe fn handle_byte(&mut self, byte: u8) -> Kind {
        // SAFETY: Caller guarantees to uphold safety invariants
        unsafe { BYTE_HANDLERS[byte as usize](self) }
    }
}

type ByteHandler = unsafe fn(&mut Lexer<'_>) -> Kind;

/// Lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/v0.7.0/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
static BYTE_HANDLERS: [ByteHandler; 256] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, ISP, ISP, LIN, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    SPS, EXL, QOD, HAS, IDT, PRC, AMP, QOS, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, GTR, QST, // 3
    AT_, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, ESC, BTC, CRT, IDT, // 5
    TPL, L_A, L_B, L_C, L_D, L_E, L_F, L_G, IDT, L_I, IDT, L_K, L_L, L_M, L_N, L_O, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // 8
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // 9
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // A
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // B
    UER, UER, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // F
];

/// The macro definitions have been removed and replaced with individual functions below.
/// This eliminates the need for the complex macro expansions while maintaining the same functionality.
///
/// For ASCII byte handlers, the unsafe assertions for optimization hints are preserved.
/// For identifier handlers, the unsafe call to identifier_name_handler is preserved.

// `\0` `\1` etc
const ERR: ByteHandler = {
    #[expect(non_snake_case)]
    fn ERR(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        let c = lexer.consume_char();
        lexer.error(diagnostics::invalid_character(c, lexer.unterminated_range()));
        Kind::Undetermined
    }
    ERR
};

// <SPACE> <TAB> Normal Whitespace
const SPS: ByteHandler = {
    #[expect(non_snake_case)]
    fn SPS(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::Skip
    }
    SPS
};

// <VT> <FF> Irregular Whitespace
const ISP: ByteHandler = {
    #[expect(non_snake_case)]
    fn ISP(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.trivia_builder.add_irregular_whitespace(lexer.token.start(), lexer.offset());
        Kind::Skip
    }
    ISP
};

// '\r' '\n'
const LIN: ByteHandler = {
    #[expect(non_snake_case)]
    fn LIN(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.line_break_handler()
    }
    LIN
};

// !
const EXL: ByteHandler = {
    #[expect(non_snake_case)]
    fn EXL(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        // Try to peek at next byte for common case first
        match lexer.peek_byte() {
            Some(b'=') => {
                lexer.consume_char();
                // Check for !== (triple equals)
                if lexer.peek_byte() == Some(b'=') {
                    lexer.consume_char();
                    Kind::Neq2
                } else {
                    Kind::Neq
                }
            }
            _ => Kind::Bang,
        }
    }
    EXL
};

// "
const QOD: ByteHandler = {
    #[expect(non_snake_case)]
    fn QOD(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        // SAFETY: This function is only called for `"`
        unsafe { lexer.read_string_literal_double_quote() }
    }
    QOD
};

// '
const QOS: ByteHandler = {
    #[expect(non_snake_case)]
    fn QOS(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        // SAFETY: This function is only called for `'`
        unsafe { lexer.read_string_literal_single_quote() }
    }
    QOS
};

// #
const HAS: ByteHandler = {
    #[expect(non_snake_case)]
    fn HAS(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.private_identifier()
    }
    HAS
};

// `A..=Z`, `a..=z` (except special cases below), `_`, `$`
const IDT: ByteHandler = {
    #[expect(non_snake_case)]
    fn IDT(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let _id_without_first_char = unsafe { lexer.identifier_name_handler() };
        Kind::Ident
    }
    IDT
};

// %
const PRC: ByteHandler = {
    #[expect(non_snake_case)]
    fn PRC(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        if lexer.next_ascii_byte_eq(b'=') { Kind::PercentEq } else { Kind::Percent }
    }
    PRC
};

// &
const AMP: ByteHandler = {
    #[expect(non_snake_case)]
    fn AMP(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        if lexer.next_ascii_byte_eq(b'&') {
            if lexer.next_ascii_byte_eq(b'=') { Kind::Amp2Eq } else { Kind::Amp2 }
        } else if lexer.next_ascii_byte_eq(b'=') {
            Kind::AmpEq
        } else {
            Kind::Amp
        }
    }
    AMP
};

// (
const PNO: ByteHandler = {
    #[expect(non_snake_case)]
    fn PNO(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::LParen
    }
    PNO
};

// )
const PNC: ByteHandler = {
    #[expect(non_snake_case)]
    fn PNC(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::RParen
    }
    PNC
};

// *
const ATR: ByteHandler = {
    #[expect(non_snake_case)]
    fn ATR(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        if lexer.next_ascii_byte_eq(b'*') {
            if lexer.next_ascii_byte_eq(b'=') { Kind::Star2Eq } else { Kind::Star2 }
        } else if lexer.next_ascii_byte_eq(b'=') {
            Kind::StarEq
        } else {
            Kind::Star
        }
    }
    ATR
};

// +
const PLS: ByteHandler = {
    #[expect(non_snake_case)]
    fn PLS(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        if lexer.next_ascii_byte_eq(b'+') {
            Kind::Plus2
        } else if lexer.next_ascii_byte_eq(b'=') {
            Kind::PlusEq
        } else {
            Kind::Plus
        }
    }
    PLS
};

// ,
const COM: ByteHandler = {
    #[expect(non_snake_case)]
    fn COM(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::Comma
    }
    COM
};

// -
const MIN: ByteHandler = {
    #[expect(non_snake_case)]
    fn MIN(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment())
    }
    MIN
};

// .
const PRD: ByteHandler = {
    #[expect(non_snake_case)]
    fn PRD(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.read_dot()
    }
    PRD
};

// /
const SLH: ByteHandler = {
    #[expect(non_snake_case)]
    fn SLH(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        match lexer.peek_byte() {
            Some(b'/') => {
                lexer.consume_char();
                lexer.skip_single_line_comment()
            }
            Some(b'*') => {
                lexer.consume_char();
                lexer.skip_multi_line_comment()
            }
            _ => {
                // regex is handled separately, see `next_regex`
                if lexer.next_ascii_byte_eq(b'=') { Kind::SlashEq } else { Kind::Slash }
            }
        }
    }
    SLH
};

// 0
const ZER: ByteHandler = {
    #[expect(non_snake_case)]
    fn ZER(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.read_zero()
    }
    ZER
};

// 1 to 9
const DIG: ByteHandler = {
    #[expect(non_snake_case)]
    fn DIG(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.decimal_literal_after_first_digit()
    }
    DIG
};

// :
const COL: ByteHandler = {
    #[expect(non_snake_case)]
    fn COL(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::Colon
    }
    COL
};

// ;
const SEM: ByteHandler = {
    #[expect(non_snake_case)]
    fn SEM(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::Semicolon
    }
    SEM
};

// <
const LSS: ByteHandler = {
    #[expect(non_snake_case)]
    fn LSS(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment())
    }
    LSS
};

// =
const EQL: ByteHandler = {
    #[expect(non_snake_case)]
    fn EQL(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        if lexer.next_ascii_byte_eq(b'=') {
            if lexer.next_ascii_byte_eq(b'=') { Kind::Eq3 } else { Kind::Eq2 }
        } else if lexer.next_ascii_byte_eq(b'>') {
            Kind::Arrow
        } else {
            Kind::Eq
        }
    }
    EQL
};

// >
const GTR: ByteHandler = {
    #[expect(non_snake_case)]
    fn GTR(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        // `>=` is re-lexed with [Lexer::next_jsx_child]
        Kind::RAngle
    }
    GTR
};

// ?
const QST: ByteHandler = {
    #[expect(non_snake_case)]
    fn QST(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();

        match lexer.peek_byte() {
            Some(b'?') => {
                lexer.consume_char();
                // Check for ??= (nullish coalescing assignment)
                if lexer.peek_byte() == Some(b'=') {
                    lexer.consume_char();
                    Kind::Question2Eq
                } else {
                    Kind::Question2
                }
            }
            Some(b'.') => {
                // Only consume if not followed by digit (to parse `?.1` as `?` `.1`)
                if lexer.peek_2_bytes().is_none_or(|bytes| !bytes[1].is_ascii_digit()) {
                    lexer.consume_char();
                    Kind::QuestionDot
                } else {
                    Kind::Question
                }
            }
            _ => Kind::Question,
        }
    }
    QST
};

// @
const AT_: ByteHandler = {
    #[expect(non_snake_case)]
    fn AT_(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::At
    }
    AT_
};

// [
const BTO: ByteHandler = {
    #[expect(non_snake_case)]
    fn BTO(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::LBrack
    }
    BTO
};

// \
const ESC: ByteHandler = {
    #[expect(non_snake_case)]
    fn ESC(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.identifier_backslash_handler()
    }
    ESC
};

// ]
const BTC: ByteHandler = {
    #[expect(non_snake_case)]
    fn BTC(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::RBrack
    }
    BTC
};

// ^
const CRT: ByteHandler = {
    #[expect(non_snake_case)]
    fn CRT(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        if lexer.next_ascii_byte_eq(b'=') { Kind::CaretEq } else { Kind::Caret }
    }
    CRT
};

// `
const TPL: ByteHandler = {
    #[expect(non_snake_case)]
    fn TPL(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        lexer.read_template_literal(Kind::TemplateHead, Kind::NoSubstitutionTemplate)
    }
    TPL
};

// {
const BEO: ByteHandler = {
    #[expect(non_snake_case)]
    fn BEO(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::LCurly
    }
    BEO
};

// |
const PIP: ByteHandler = {
    #[expect(non_snake_case)]
    fn PIP(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();

        match lexer.peek_byte() {
            Some(b'|') => {
                lexer.consume_char();
                if lexer.next_ascii_byte_eq(b'=') { Kind::Pipe2Eq } else { Kind::Pipe2 }
            }
            Some(b'=') => {
                lexer.consume_char();
                Kind::PipeEq
            }
            _ => Kind::Pipe,
        }
    }
    PIP
};

// }
const BEC: ByteHandler = {
    #[expect(non_snake_case)]
    fn BEC(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::RCurly
    }
    BEC
};

// ~
const TLD: ByteHandler = {
    #[expect(non_snake_case)]
    fn TLD(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        unsafe {
            use oxc_data_structures::assert_unchecked;
            assert_unchecked!(!lexer.source.is_eof());
            assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
        }
        lexer.consume_char();
        Kind::Tilde
    }
    TLD
};

const L_A: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_A(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "wait" => Kind::Await,
            "sync" => Kind::Async,
            "bstract" => Kind::Abstract,
            "ccessor" => Kind::Accessor,
            "ny" => Kind::Any,
            "s" => Kind::As,
            "ssert" => Kind::Assert,
            "sserts" => Kind::Asserts,
            _ => Kind::Ident,
        }
    }
    L_A
};

const L_B: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_B(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "reak" => Kind::Break,
            "oolean" => Kind::Boolean,
            "igint" => Kind::BigInt,
            _ => Kind::Ident,
        }
    }
    L_B
};

const L_C: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_C(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "onst" => Kind::Const,
            "lass" => Kind::Class,
            "ontinue" => Kind::Continue,
            "atch" => Kind::Catch,
            "ase" => Kind::Case,
            "onstructor" => Kind::Constructor,
            _ => Kind::Ident,
        }
    }
    L_C
};

const L_D: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_D(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "o" => Kind::Do,
            "elete" => Kind::Delete,
            "eclare" => Kind::Declare,
            "efault" => Kind::Default,
            "ebugger" => Kind::Debugger,
            "efer" => Kind::Defer,
            _ => Kind::Ident,
        }
    }
    L_D
};

const L_E: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_E(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "lse" => Kind::Else,
            "num" => Kind::Enum,
            "xport" => Kind::Export,
            "xtends" => Kind::Extends,
            _ => Kind::Ident,
        }
    }
    L_E
};

const L_F: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_F(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "unction" => Kind::Function,
            "alse" => Kind::False,
            "or" => Kind::For,
            "inally" => Kind::Finally,
            "rom" => Kind::From,
            _ => Kind::Ident,
        }
    }
    L_F
};

const L_G: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_G(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "et" => Kind::Get,
            "lobal" => Kind::Global,
            _ => Kind::Ident,
        }
    }
    L_G
};

const L_I: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_I(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "f" => Kind::If,
            "nstanceof" => Kind::Instanceof,
            "n" => Kind::In,
            "mplements" => Kind::Implements,
            "mport" => Kind::Import,
            "nfer" => Kind::Infer,
            "nterface" => Kind::Interface,
            "ntrinsic" => Kind::Intrinsic,
            "s" => Kind::Is,
            _ => Kind::Ident,
        }
    }
    L_I
};

const L_K: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_K(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "eyof" => Kind::KeyOf,
            _ => Kind::Ident,
        }
    }
    L_K
};

const L_L: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_L(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "et" => Kind::Let,
            _ => Kind::Ident,
        }
    }
    L_L
};

const L_M: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_M(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "eta" => Kind::Meta,
            "odule" => Kind::Module,
            _ => Kind::Ident,
        }
    }
    L_M
};

const L_N: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_N(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "ull" => Kind::Null,
            "ew" => Kind::New,
            "umber" => Kind::Number,
            "amespace" => Kind::Namespace,
            "ever" => Kind::Never,
            _ => Kind::Ident,
        }
    }
    L_N
};

const L_O: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_O(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "f" => Kind::Of,
            "bject" => Kind::Object,
            "ut" => Kind::Out,
            "verride" => Kind::Override,
            _ => Kind::Ident,
        }
    }
    L_O
};

const L_P: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_P(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "ackage" => Kind::Package,
            "rivate" => Kind::Private,
            "rotected" => Kind::Protected,
            "ublic" => Kind::Public,
            _ => Kind::Ident,
        }
    }
    L_P
};

const L_R: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_R(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "eturn" => Kind::Return,
            "equire" => Kind::Require,
            "eadonly" => Kind::Readonly,
            _ => Kind::Ident,
        }
    }
    L_R
};

const L_S: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_S(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "et" => Kind::Set,
            "uper" => Kind::Super,
            "witch" => Kind::Switch,
            "tatic" => Kind::Static,
            "ymbol" => Kind::Symbol,
            "tring" => Kind::String,
            "atisfies" => Kind::Satisfies,
            "ource" => Kind::Source,
            _ => Kind::Ident,
        }
    }
    L_S
};

const L_T: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_T(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "his" => Kind::This,
            "rue" => Kind::True,
            "hrow" => Kind::Throw,
            "ry" => Kind::Try,
            "ypeof" => Kind::Typeof,
            "arget" => Kind::Target,
            "ype" => Kind::Type,
            _ => Kind::Ident,
        }
    }
    L_T
};

const L_U: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_U(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "ndefined" => Kind::Undefined,
            "sing" => Kind::Using,
            "nique" => Kind::Unique,
            "nknown" => Kind::Unknown,
            _ => Kind::Ident,
        }
    }
    L_U
};

const L_V: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_V(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "ar" => Kind::Var,
            "oid" => Kind::Void,
            _ => Kind::Ident,
        }
    }
    L_V
};

const L_W: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_W(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "hile" => Kind::While,
            "ith" => Kind::With,
            _ => Kind::Ident,
        }
    }
    L_W
};

const L_Y: ByteHandler = {
    #[expect(non_snake_case)]
    fn L_Y(lexer: &mut Lexer) -> Kind {
        // SAFETY: This function is only used for ASCII characters
        let id_without_first_char = unsafe { lexer.identifier_name_handler() };
        match id_without_first_char {
            "ield" => Kind::Yield,
            _ => Kind::Ident,
        }
    }
    L_Y
};

// Non-ASCII characters.
const UNI: ByteHandler = {
    #[expect(non_snake_case)]
    fn UNI(lexer: &mut Lexer) -> Kind {
        lexer.unicode_char_handler()
    }
    UNI
};

// UTF-8 continuation bytes (0x80 - 0xBF) (i.e. middle of a multi-byte UTF-8 sequence)
// + and byte values which are not legal in UTF-8 strings (0xC0, 0xC1, 0xF5 - 0xFF).
// `handle_byte()` should only be called with 1st byte of a valid UTF-8 character,
// so something has gone wrong if we get here.
// https://datatracker.ietf.org/doc/html/rfc3629
const UER: ByteHandler = {
    #[expect(non_snake_case)]
    fn UER(_lexer: &mut Lexer) -> Kind {
        unreachable!();
    }
    UER
};
