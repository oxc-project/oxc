use oxc_ast::CommentKind;
use oxc_data_structures::assert_unchecked;

use crate::{
    config::{LexerConfig as Config, NoTokensLexerConfig, RuntimeLexerConfig, TokensLexerConfig},
    diagnostics,
};

use super::{Kind, Lexer};

impl<C: Config> Lexer<'_, C> {
    /// Handle next byte of source.
    ///
    /// # SAFETY
    ///
    /// * Lexer must not be at end of file.
    /// * `byte` must be next byte of source code, corresponding to current position of `lexer.source`.
    /// * Only byte handlers for ASCII characters may use the `ascii_byte_handler!()` macro.
    // `#[inline(always)]` to ensure is inlined into `read_next_token`
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) unsafe fn handle_byte(&mut self, byte: u8) -> Kind {
        // Bypass indirect dispatch for these common one-byte tokens.
        // Keep this match small: adding other handlers regressed ARM64 parser benchmarks
        // through extra dispatch work and reduced inlining into parser callers.
        // Re-benchmark before extending this set.
        let kind = match byte {
            b'(' => Kind::LParen,
            b')' => Kind::RParen,
            b';' => Kind::Semicolon,
            _ => {
                let byte_handlers = self.config.byte_handlers();
                // SAFETY: Caller guarantees to uphold safety invariants.
                return unsafe { byte_handlers[byte as usize](self) };
            }
        };

        // SAFETY: Caller guarantees `byte` is the current source byte, and these are single-byte ASCII characters.
        let next = unsafe { self.source.position().add(1) };
        self.source.set_position(next);
        kind
    }
}

pub type ByteHandler<C> = unsafe fn(&mut Lexer<'_, C>) -> Kind;
pub type ByteHandlers<C> = [ByteHandler<C>; 256];

/// Macro to create a lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/e55a1310ba69a3f5ce2a9a6eef643feced02ac08/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
macro_rules! byte_handlers {
    () => {
        [
        //  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
            ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, ISP, ISP, LIN, ERR, ERR, // 0
            ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
            SPS, EXL, QOD, HAS, IDT, PRC, AMP, QOS, UER, UER, ATR, PLS, COM, MIN, PRD, SLH, // 2
            ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, UER, LSS, EQL, GTR, QST, // 3
            AT_, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
            IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, ESC, BTC, CRT, IDT, // 5
            TPL, KWD, KWD, KWD, KWD, KWD, KWD, KWD, IDT, KWD, IDT, KWD, KWD, KWD, KWD, KWD, // 6
            KWD, IDT, KWD, KWD, KWD, KWD, KWD, KWD, IDT, KWD, IDT, BEO, PIP, BEC, TLD, ERR, // 7
            UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // 8
            UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // 9
            UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // A
            UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // B
            UER, UER, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
            UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
            UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
            UNI, UNI, UNI, UNI, UNI, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // F
        ]
    };
}

pub mod byte_handler_tables {
    use super::*;

    pub static NO_TOKENS: ByteHandlers<NoTokensLexerConfig> = byte_handlers!();
    pub static WITH_TOKENS: ByteHandlers<TokensLexerConfig> = byte_handlers!();
    pub static RUNTIME_TOKENS: ByteHandlers<RuntimeLexerConfig> = byte_handlers!();
}

/// Macro for defining byte handler for an ASCII character.
///
/// Asserts that lexer is not at end of file, and that next char is ASCII.
/// Where the handler is for an ASCII character, these assertions are self-evidently true.
///
/// These assertions produce no runtime code, but hint to the compiler that it can assume that
/// next char is ASCII, and it uses that information to optimize the rest of the handler.
/// e.g. `lexer.consume_char()` becomes just a single assembly instruction.
/// Without the assertions, the compiler is unable to deduce the next char is ASCII, due to
/// the indirection of the byte handlers jump table.
///
/// These assertions are unchecked (i.e. won't panic) and will cause UB if they're incorrect.
///
/// # SAFETY
/// Only use this macro to define byte handlers for ASCII characters.
///
/// ```rust,ignore
/// ascii_byte_handler!(SPS(lexer) {
///     lexer.consume_char();
///     Kind::WhiteSpace
/// });
/// ```
///
/// expands to:
///
/// ```rust,ignore
/// #[expect(non_snake_case)]
/// fn SPS<C: Config>(lexer: &mut Lexer<'_, C>) -> Kind {
///     // SAFETY: This macro is only used for ASCII characters
///     unsafe {
///         assert_unchecked!(!lexer.source.is_eof());
///         assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
///     }
///     {
///         lexer.consume_char();
///         Kind::WhiteSpace
///     }
/// }
/// ```
macro_rules! ascii_byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        #[expect(non_snake_case)]
        fn $id<C: Config>($lex: &mut Lexer<'_, C>) -> Kind {
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                assert_unchecked!(!$lex.source.is_eof());
                assert_unchecked!($lex.source.peek_byte_unchecked() < 128);
            }
            $body
        }
    };
}

// `\0` `\1` etc
ascii_byte_handler!(ERR(lexer) {
    let c = lexer.consume_char();
    lexer.error(diagnostics::invalid_character(c, lexer.unterminated_range()));
    Kind::Undetermined
});

// <SPACE> <TAB> Normal Whitespace
ascii_byte_handler!(SPS(lexer) {
    lexer.whitespace_handler()
});

// <VT> <FF> Irregular Whitespace
ascii_byte_handler!(ISP(lexer) {
    lexer.consume_char();
    lexer.trivia_builder.add_irregular_whitespace(lexer.token_start, lexer.offset());
    Kind::Skip
});

// '\r' '\n'
ascii_byte_handler!(LIN(lexer) {
    lexer.consume_char();
    lexer.line_break_handler()
});

// !
ascii_byte_handler!(EXL(lexer) {
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
        _ => Kind::Bang
    }
});

// "
ascii_byte_handler!(QOD(lexer) {
    // SAFETY: This function is only called for `"`
    unsafe { lexer.read_string_literal_double_quote() }
});

// '
ascii_byte_handler!(QOS(lexer) {
    // SAFETY: This function is only called for `'`
    unsafe { lexer.read_string_literal_single_quote() }
});

// #
ascii_byte_handler!(HAS(lexer) {
    lexer.consume_char();
    lexer.private_identifier()
});

// `A..=Z`, `h`, `j`, `q`, `x`, `z`, `_`, `$`: no keyword starts with these bytes.
#[expect(non_snake_case)]
fn IDT<C: Config>(lexer: &mut Lexer<'_, C>) -> Kind {
    // SAFETY: This handler is only selected for ASCII identifier-start bytes.
    unsafe { lexer.identifier_name_handler::<false>() }
}

// Share one indirect-call target for all first letters which can begin a keyword.
#[expect(non_snake_case)]
fn KWD<C: Config>(lexer: &mut Lexer<'_, C>) -> Kind {
    // SAFETY: This handler is only selected for ASCII identifier-start bytes.
    unsafe { lexer.identifier_name_handler::<true>() }
}

// %
ascii_byte_handler!(PRC(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
});

// &
ascii_byte_handler!(AMP(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'&') {
        if lexer.next_ascii_byte_eq(b'=') {
            Kind::Amp2Eq
        } else {
            Kind::Amp2
        }
    } else if lexer.next_ascii_byte_eq(b'=') {
        Kind::AmpEq
    } else {
        Kind::Amp
    }
});

// *
ascii_byte_handler!(ATR(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'*') {
        if lexer.next_ascii_byte_eq(b'=') {
            Kind::Star2Eq
        } else {
            Kind::Star2
        }
    } else if lexer.next_ascii_byte_eq(b'=') {
        Kind::StarEq
    } else {
        Kind::Star
    }
});

// +
ascii_byte_handler!(PLS(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'+') {
        Kind::Plus2
    } else if lexer.next_ascii_byte_eq(b'=') {
        Kind::PlusEq
    } else {
        Kind::Plus
    }
});

// ,
ascii_byte_handler!(COM(lexer) {
    lexer.consume_char();
    Kind::Comma
});

// -
ascii_byte_handler!(MIN(lexer) {
    lexer.consume_char();
    lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment(CommentKind::HtmlClose))
});

// .
ascii_byte_handler!(PRD(lexer) {
    lexer.consume_char();
    lexer.read_dot()
});

// /
ascii_byte_handler!(SLH(lexer) {
    lexer.consume_char();
    match lexer.peek_byte() {
        Some(b'/') => {
            lexer.consume_char();
            lexer.skip_single_line_comment(CommentKind::Line)
        }
        Some(b'*') => {
            lexer.consume_char();
            lexer.skip_multi_line_comment()
        }
        _ => {
            // regex is handled separately, see `next_regex`
            if lexer.next_ascii_byte_eq(b'=') {
                Kind::SlashEq
            } else {
                Kind::Slash
            }
        }
    }
});

// 0
ascii_byte_handler!(ZER(lexer) {
    lexer.consume_char();
    lexer.read_zero()
});

// 1 to 9
ascii_byte_handler!(DIG(lexer) {
    lexer.consume_char();
    lexer.decimal_literal_after_first_digit()
});

// :
ascii_byte_handler!(COL(lexer) {
    lexer.consume_char();
    Kind::Colon
});

// <
ascii_byte_handler!(LSS(lexer) {
    lexer.consume_char();
    lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment(CommentKind::HtmlOpen))
});

// =
ascii_byte_handler!(EQL(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        if lexer.next_ascii_byte_eq(b'=') {
            Kind::Eq3
        } else {
            Kind::Eq2
        }
    } else if lexer.next_ascii_byte_eq(b'>') {
        Kind::Arrow
    } else {
        Kind::Eq
    }
});

// >
ascii_byte_handler!(GTR(lexer) {
    lexer.consume_char();
    // `>=` is re-lexed with [Lexer::next_jsx_child]
    Kind::RAngle
});

// ?
ascii_byte_handler!(QST(lexer) {
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
});

// @
ascii_byte_handler!(AT_(lexer) {
    lexer.consume_char();
    Kind::At
});

// [
ascii_byte_handler!(BTO(lexer) {
    lexer.consume_char();
    Kind::LBrack
});

// \
ascii_byte_handler!(ESC(lexer) {
    lexer.identifier_backslash_handler()
});

// ]
ascii_byte_handler!(BTC(lexer) {
    lexer.consume_char();
    Kind::RBrack
});

// ^
ascii_byte_handler!(CRT(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::CaretEq
    } else {
        Kind::Caret
    }
});

// `
ascii_byte_handler!(TPL(lexer) {
    lexer.consume_char();
    lexer.read_template_literal(Kind::TemplateHead, Kind::NoSubstitutionTemplate)
});

// {
ascii_byte_handler!(BEO(lexer) {
    lexer.consume_char();
    Kind::LCurly
});

// |
ascii_byte_handler!(PIP(lexer) {
    lexer.consume_char();

    match lexer.peek_byte() {
        Some(b'|') => {
            lexer.consume_char();
            if lexer.next_ascii_byte_eq(b'=') {
                Kind::Pipe2Eq
            } else {
                Kind::Pipe2
            }
        }
        Some(b'=') => {
            lexer.consume_char();
            Kind::PipeEq
        }
        _ => Kind::Pipe
    }
});

// }
ascii_byte_handler!(BEC(lexer) {
    lexer.consume_char();
    Kind::RCurly
});

// ~
ascii_byte_handler!(TLD(lexer) {
    lexer.consume_char();
    Kind::Tilde
});

// Non-ASCII characters.
//
// Note: Must not use `ascii_byte_handler!` macro, as this handler is for non-ASCII chars.
#[expect(non_snake_case)]
fn UNI<C: Config>(lexer: &mut Lexer<'_, C>) -> Kind {
    lexer.unicode_char_handler()
}

// Punctuation dispatched directly in `handle_byte`, and invalid UTF-8 start bytes.
//
// UTF-8 continuation bytes (0x80 - 0xBF) (i.e. middle of a multi-byte UTF-8 sequence)
// + and byte values which are not legal in UTF-8 strings (0xC0, 0xC1, 0xF5 - 0xFF).
// `handle_byte()` should only be called with 1st byte of a valid UTF-8 character,
// so something has gone wrong if we get here.
// https://datatracker.ietf.org/doc/html/rfc3629
//
// Note: Must not use `ascii_byte_handler!` macro, as this handler also covers non-ASCII bytes.
#[expect(non_snake_case)]
fn UER<C: Config>(_lexer: &mut Lexer<'_, C>) -> Kind {
    unreachable!();
}
