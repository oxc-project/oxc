use oxc_ast::CommentKind;
use oxc_data_structures::assert_unchecked;

use crate::{
    config::{LexerConfig as Config, NoTokensLexerConfig, RuntimeLexerConfig, TokensLexerConfig},
    diagnostics,
};

use super::{FinishTokenMode, Kind, Lexer, SourcePosition, Token, numeric::ends_decimal_integer};

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
        if let Some(kind) = single_byte_token(byte) {
            // SAFETY: Caller guarantees `byte` is the current source byte, and single-byte tokens are ASCII.
            let next = unsafe { self.source.position().add(1) };
            self.source.set_position(next);
            return kind;
        }
        let byte_handlers = self.config.byte_handlers();
        // SAFETY: Caller guarantees to uphold safety invariants.
        unsafe { byte_handlers[byte as usize](self, self.source.position()).0 }
    }
}

/// `Kind` of a token which is always exactly the single byte `byte`, if there is one.
/// These tokens are completed without an indirect call to a byte handler.
#[expect(clippy::inline_always)]
#[inline(always)]
pub(super) fn single_byte_token(byte: u8) -> Option<Kind> {
    let kind = SINGLE_BYTE_TOKENS[byte as usize];
    if kind == Kind::Eof { None } else { Some(kind) }
}

/// `>` is included because `>=`, `>>` etc are produced by re-lexing (see `re_lex_right_angle`).
static SINGLE_BYTE_TOKENS: [Kind; 256] = {
    let mut table = [Kind::Eof; 256];
    table[b'(' as usize] = Kind::LParen;
    table[b')' as usize] = Kind::RParen;
    table[b';' as usize] = Kind::Semicolon;
    table[b',' as usize] = Kind::Comma;
    table[b':' as usize] = Kind::Colon;
    table[b'[' as usize] = Kind::LBrack;
    table[b']' as usize] = Kind::RBrack;
    table[b'{' as usize] = Kind::LCurly;
    table[b'}' as usize] = Kind::RCurly;
    table[b'~' as usize] = Kind::Tilde;
    table[b'@' as usize] = Kind::At;
    table[b'>' as usize] = Kind::RAngle;
    table
};

/// Byte handlers take and return the current position of `lexer.source`, so it stays in a register
/// between tokens instead of being stored and reloaded.
pub type ByteHandler<C> =
    for<'a> unsafe fn(&mut Lexer<'a, C>, SourcePosition<'a>) -> (Kind, SourcePosition<'a>);
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
            SPS, EXL, QOD, HAS, IDT, PRC, AMP, QOS, UER, UER, ATR, PLS, UER, MIN, PRD, SLH, // 2
            ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, UER, UER, LSS, EQL, UER, QST, // 3
            UER, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
            IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, UER, ESC, UER, CRT, IDT, // 5
            TPL, KWD, KWD, KWD, KWD, KWD, KWD, KWD, IDT, KWD, IDT, KWD, KWD, KWD, KWD, KWD, // 6
            KWD, IDT, KWD, KWD, KWD, KWD, KWD, KWD, IDT, KWD, IDT, UER, PIP, UER, UER, ERR, // 7
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

/// Token handlers take the position of the first byte of a token, and return the completed token.
///
/// # SAFETY
/// All token handlers have the same requirements. `Lexer::next_token_impl` upholds them.
/// * `pos` must be the position of a byte which selects the handler in `Lexer::TOKEN_HANDLERS`.
/// * At least `TOKEN_HANDLER_MIN_BYTES` bytes of source must remain from `pos`.
/// * No whitespace or comments must have been seen since the previous token, other than
///   a single space.
type TokenHandler<C> = for<'a> unsafe fn(&mut Lexer<'a, C>, SourcePosition<'a>) -> Token;

/// Token handlers, each named after the byte handler for the same byte.
/// See [`TokenHandler`] for what callers must guarantee.
mod token_handlers {
    use super::{Config, Kind, Lexer, SourcePosition, Token, ends_decimal_integer};

    impl<C: Config> Lexer<'_, C> {
        /// Lookup table mapping any byte to the token handler for tokens which start with that byte.
        ///
        /// Token handlers are generic over the lexer's config, and a `static` cannot be generic,
        /// so the table is an associated constant. `&Self::TOKEN_HANDLERS` is a reference to
        /// static memory, so getting a handler from the table is a single load, as with a `static`.
        ///
        /// Defined in this module so that the names in `byte_handlers!` refer to the token handlers.
        pub(in crate::lexer) const TOKEN_HANDLERS: [super::TokenHandler<C>; 256] = byte_handlers!();
    }

    /// Define token handlers which call the byte handler of the same name, then finish the token.
    ///
    /// * `fresh` is for byte handlers which only ever produce a token, without flags.
    ///   They must not return `Kind::Skip`, nor alter `lexer.pending`.
    /// * `zero` is for the byte handler for `0`. A lone `0` is the most common number which
    ///   starts with it, and is finished without calling the byte handler.
    /// * `string` is for the byte handlers for quotes. A string which contains no escapes has
    ///   no flags, and is finished without calling the byte handler. Other strings are uncommon,
    ///   and are finished from where the first escape in them is found.
    /// * `general` is for all other byte handlers.
    macro_rules! token_handlers {
        (fresh: $($name:ident),+ $(,)?) => {
            $(
                #[expect(non_snake_case)]
                pub unsafe fn $name<'a, C: Config>(
                    lexer: &mut Lexer<'a, C>,
                    pos: SourcePosition<'a>,
                ) -> Token {
                    let (kind, end) = super::$name(lexer, pos);
                    lexer.finish_fresh_token(kind, pos, end)
                }
            )+
        };
        (zero: $($name:ident),+ $(,)?) => {
            $(
                #[expect(non_snake_case)]
                pub unsafe fn $name<'a, C: Config>(
                    lexer: &mut Lexer<'a, C>,
                    pos: SourcePosition<'a>,
                ) -> Token {
                    // SAFETY: Caller guarantees `TOKEN_HANDLER_MIN_BYTES` bytes can be read
                    // from `pos`. A digit is ASCII, so `pos + 1` is on a UTF-8 char boundary.
                    let (after_digit, next_byte) = unsafe {
                        let after_digit = pos.add(1);
                        (after_digit, after_digit.read())
                    };
                    if ends_decimal_integer(next_byte) {
                        lexer.source.set_position(after_digit);
                        return lexer.finish_fresh_token(Kind::Decimal, pos, after_digit);
                    }
                    // Decimal points, other radixes, legacy octal.
                    // SAFETY: Caller guarantees `pos` is the position of a byte which this handler,
                    // and so the byte handler of the same name, is for.
                    unsafe { lexer.token_from_byte_handler(pos, super::$name) }
                }
            )+
        };
        (string: $($name:ident => $read_plain_string:ident, $string_token_rest:ident);+ $(;)?) => {
            $(
                #[expect(non_snake_case)]
                pub unsafe fn $name<'a, C: Config>(
                    lexer: &mut Lexer<'a, C>,
                    pos: SourcePosition<'a>,
                ) -> Token {
                    // SAFETY: Caller guarantees `pos` is the position of the quote
                    // which this handler, and so `$read_plain_string`, is for
                    match unsafe { lexer.$read_plain_string(pos) } {
                        Ok(end) => lexer.finish_fresh_token(Kind::Str, pos, end),
                        // SAFETY: As above, and position of source has not been altered since
                        Err(next_byte) => unsafe { lexer.$string_token_rest(pos, next_byte) },
                    }
                }
            )+
        };
        (general: $($name:ident),+ $(,)?) => {
            $(
                #[expect(non_snake_case)]
                pub unsafe fn $name<'a, C: Config>(
                    lexer: &mut Lexer<'a, C>,
                    pos: SourcePosition<'a>,
                ) -> Token {
                    // SAFETY: Caller guarantees `pos` is the position of a byte which this handler,
                    // and so the byte handler of the same name, is for
                    unsafe { lexer.general_token_handler(pos, super::$name) }
                }
            )+
        };
    }

    token_handlers!(fresh: AMP, ATR, CRT, EXL, PIP, PLS, PRC, QST);

    token_handlers!(zero: ZER);

    token_handlers!(
        string:
        QOD => read_plain_string_literal_double_quote, string_token_rest_double_quote;
        QOS => read_plain_string_literal_single_quote, string_token_rest_single_quote;
    );

    token_handlers!(
        general: ERR, ESC, HAS, ISP, SLH, SPS, TPL, UER, UNI,
    );

    // `A..=Z`, `h`, `j`, `q`, `x`, `z`, `_`, `$`: no keyword starts with these bytes.
    #[expect(non_snake_case)]
    pub unsafe fn IDT<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `pos` is the position of an ASCII identifier-start byte,
        // and that no trivia has been seen, other than one space
        unsafe { lexer.identifier_token::<false, false>(pos) }
    }

    // All first letters which can begin a keyword.
    #[expect(non_snake_case)]
    pub unsafe fn KWD<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `pos` is the position of an ASCII identifier-start byte,
        // and that no trivia has been seen, other than one space
        unsafe { lexer.identifier_token::<true, false>(pos) }
    }

    // `\r` `\n`
    #[expect(non_snake_case)]
    pub unsafe fn LIN<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `pos` is the position of a line break byte,
        // and that no trivia has been seen, other than one space
        unsafe { lexer.line_break_token(pos) }
    }

    // `.`
    //
    // Almost always a property access, where what follows is an identifier.
    #[expect(non_snake_case)]
    pub unsafe fn PRD<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `TOKEN_HANDLER_MIN_BYTES` bytes can be read from `pos`.
        // `.` is ASCII, so `pos + 1` is on a UTF-8 char boundary.
        let (after_dot, next_byte) = unsafe {
            let after_dot = pos.add(1);
            (after_dot, after_dot.read())
        };
        // `.` or a digit means `...` or a number. `/` also takes this branch, which is harmless.
        if next_byte.wrapping_sub(b'.') <= b'9' - b'.' {
            // SAFETY: Caller guarantees `pos` is the position of `.`, which `PRD` is for
            return unsafe { lexer.token_from_byte_handler(pos, super::PRD) };
        }
        lexer.source.set_position(after_dot);
        lexer.finish_fresh_token(Kind::Dot, pos, after_dot)
    }

    // `1` to `9`
    //
    // Most numbers are integers of a few digits, particularly in minified code.
    // They have no flags, and are finished without calling the byte handler.
    #[expect(non_snake_case)]
    pub unsafe fn DIG<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `TOKEN_HANDLER_MIN_BYTES` bytes can be read from `pos`.
        // A digit is ASCII, so `pos + 1` is on a UTF-8 char boundary.
        let (mut end, mut byte) = unsafe {
            let after_digit = pos.add(1);
            (after_digit, after_digit.read())
        };
        while byte.is_ascii_digit() {
            // SAFETY: `byte` is the byte at `end`, and it is ASCII, so `end + 1` is in bounds
            // and on a UTF-8 char boundary
            end = unsafe { end.add(1) };
            if !end.is_not_end_of(&lexer.source) {
                // A number at the very end of the source. Any byte which does not end a number.
                byte = b'.';
                break;
            }
            // SAFETY: `end` is not the end of the source
            byte = unsafe { end.read() };
        }
        if ends_decimal_integer(byte) {
            lexer.source.set_position(end);
            return lexer.finish_fresh_token(Kind::Decimal, pos, end);
        }
        // Decimal points, exponents, separators, BigInts
        // SAFETY: Caller guarantees `pos` is the position of a digit, which `DIG` is for
        unsafe { lexer.token_from_byte_handler(pos, super::DIG) }
    }

    // `<`
    //
    // Usually a comparison, or the start of type arguments or a JSX element.
    #[expect(non_snake_case)]
    pub unsafe fn LSS<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `TOKEN_HANDLER_MIN_BYTES` bytes can be read from `pos`.
        // `<` is ASCII, so `pos + 1` is on a UTF-8 char boundary.
        let (after_angle, next_byte) = unsafe {
            let after_angle = pos.add(1);
            (after_angle, after_angle.read())
        };
        // `<<`, `<=`, or an HTML comment `<!--`
        if matches!(next_byte, b'<' | b'=' | b'!') {
            // SAFETY: Caller guarantees `pos` is the position of `<`, which `LSS` is for
            return unsafe { lexer.token_from_byte_handler(pos, super::LSS) };
        }
        lexer.source.set_position(after_angle);
        lexer.finish_fresh_token(Kind::LAngle, pos, after_angle)
    }

    // `-`
    //
    // Usually a subtraction or a negative number.
    #[expect(non_snake_case)]
    pub unsafe fn MIN<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `TOKEN_HANDLER_MIN_BYTES` bytes can be read from `pos`.
        // `-` is ASCII, so `pos + 1` is on a UTF-8 char boundary.
        let (after_minus, next_byte) = unsafe {
            let after_minus = pos.add(1);
            (after_minus, after_minus.read())
        };
        // `--`, `-=`, or an HTML comment `-->`
        if matches!(next_byte, b'-' | b'=') {
            // SAFETY: Caller guarantees `pos` is the position of `-`, which `MIN` is for
            return unsafe { lexer.token_from_byte_handler(pos, super::MIN) };
        }
        lexer.source.set_position(after_minus);
        lexer.finish_fresh_token(Kind::Minus, pos, after_minus)
    }

    // `=`
    //
    // Almost always an assignment, where what follows is a space or the start of an expression.
    #[expect(non_snake_case)]
    pub unsafe fn EQL<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> Token {
        // SAFETY: Caller guarantees `TOKEN_HANDLER_MIN_BYTES` bytes can be read from `pos`.
        // `=` is ASCII, so `pos + 1` is on a UTF-8 char boundary.
        let (after_eq, next_byte) = unsafe {
            let after_eq = pos.add(1);
            (after_eq, after_eq.read())
        };
        if matches!(next_byte, b'=' | b'>') {
            let (kind, end) = super::EQL(lexer, pos);
            return lexer.finish_fresh_token(kind, pos, end);
        }
        lexer.source.set_position(after_eq);
        lexer.finish_fresh_token(Kind::Eq, pos, after_eq)
    }
}

impl<'a, C: Config> Lexer<'a, C> {
    /// Call a byte handler for the first byte of a token, and finish the token it produces.
    /// If it skips trivia instead, continue until a token is found.
    ///
    /// `pending` must reflect the trivia seen since the previous token.
    ///
    /// # SAFETY
    /// * `pos` must not be the end of the source.
    /// * `byte_handler` must be the byte handler for the byte at `pos`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    unsafe fn general_token_handler(
        &mut self,
        pos: SourcePosition<'a>,
        byte_handler: ByteHandler<C>,
    ) -> Token {
        // The caller may have skipped a space without updating position of source
        self.source.set_position(pos);
        self.token_start = self.source.offset_of(pos);
        // SAFETY: Caller guarantees `byte_handler` is for the byte at `pos`
        let (kind, end) = unsafe { byte_handler(self, pos) };
        if kind == Kind::Skip {
            return self.next_token_after_trivia();
        }
        self.finish_next_inner(kind, end, FinishTokenMode::Push)
    }

    /// Same as `general_token_handler`, but not inlined.
    ///
    /// For token handlers which finish most tokens themselves. They tail call this for the rest,
    /// so the tokens they do finish do not pay for the stack frame which calling a byte handler
    /// requires.
    ///
    /// # SAFETY
    /// Same as `general_token_handler`.
    #[inline(never)]
    unsafe fn token_from_byte_handler(
        &mut self,
        pos: SourcePosition<'a>,
        byte_handler: ByteHandler<C>,
    ) -> Token {
        // SAFETY: Caller guarantees what `general_token_handler` requires
        unsafe { self.general_token_handler(pos, byte_handler) }
    }
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
/// fn SPS<'a, C: Config>(lexer: &mut Lexer<'a, C>, pos: SourcePosition<'a>) -> (Kind, SourcePosition<'a>) {
///     lexer.source.set_position(pos);
///     // SAFETY: This macro is only used for ASCII characters
///     unsafe {
///         assert_unchecked!(!lexer.source.is_eof());
///         assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
///     }
///     let kind = {
///         lexer.consume_char();
///         Kind::WhiteSpace
///     };
///     (kind, lexer.source.position())
/// }
/// ```
macro_rules! ascii_byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        #[expect(non_snake_case)]
        fn $id<'a, C: Config>(
            $lex: &mut Lexer<'a, C>,
            pos: SourcePosition<'a>,
        ) -> (Kind, SourcePosition<'a>) {
            // No-op store, so the compiler uses `pos` from register instead of reloading it
            $lex.source.set_position(pos);
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                assert_unchecked!(!$lex.source.is_eof());
                assert_unchecked!($lex.source.peek_byte_unchecked() < 128);
            }
            let kind = $body;
            (kind, $lex.source.position())
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
#[expect(non_snake_case)]
fn LIN<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    pos: SourcePosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    lexer.source.set_position(pos);
    // SAFETY: The byte-handler table calls this for an in-bounds ASCII line break.
    unsafe {
        assert_unchecked!(!lexer.source.is_eof());
        assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
    }
    lexer.consume_char();
    let byte = lexer.line_break_handler();

    // Read the next token here to avoid restarting the caller's token loop.
    let pos = lexer.source.position();
    lexer.token_start = lexer.source.offset_of(pos);
    let Some(byte) = byte else {
        return (Kind::Eof, pos);
    };
    // JSX attribute values scan quotes differently. Leave them for the caller's token loop.
    if matches!(byte, b'"' | b'\'') {
        return (Kind::Skip, pos);
    }
    if let Some(kind) = single_byte_token(byte) {
        // SAFETY: Single-byte tokens are ASCII and `pos` is in bounds.
        let end = unsafe { pos.add(1) };
        lexer.source.set_position(end);
        return (kind, end);
    }
    let byte_handlers = lexer.config.byte_handlers();
    // SAFETY: `byte` is the current source byte at `pos`.
    unsafe { byte_handlers[byte as usize](lexer, pos) }
}

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
fn IDT<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    pos: SourcePosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    // SAFETY: This handler is only selected for ASCII identifier-start bytes.
    unsafe { lexer.identifier_name_handler::<false>(pos) }
}

// Share one indirect-call target for all first letters which can begin a keyword.
#[expect(non_snake_case)]
fn KWD<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    pos: SourcePosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    // SAFETY: This handler is only selected for ASCII identifier-start bytes.
    unsafe { lexer.identifier_name_handler::<true>(pos) }
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

// \
ascii_byte_handler!(ESC(lexer) {
    lexer.identifier_backslash_handler()
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

// Non-ASCII characters.
//
// Note: Must not use `ascii_byte_handler!` macro, as this handler is for non-ASCII chars.
#[expect(non_snake_case)]
fn UNI<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    _pos: SourcePosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    let kind = lexer.unicode_char_handler();
    (kind, lexer.source.position())
}

// Single-byte tokens dispatched via `SINGLE_BYTE_TOKENS`, and invalid UTF-8 start bytes.
//
// UTF-8 continuation bytes (0x80 - 0xBF) (i.e. middle of a multi-byte UTF-8 sequence)
// + and byte values which are not legal in UTF-8 strings (0xC0, 0xC1, 0xF5 - 0xFF).
// `handle_byte()` should only be called with 1st byte of a valid UTF-8 character,
// so something has gone wrong if we get here.
// https://datatracker.ietf.org/doc/html/rfc3629
//
// Note: Must not use `ascii_byte_handler!` macro, as this handler also covers non-ASCII bytes.
#[expect(non_snake_case)]
fn UER<'a, C: Config>(
    _lexer: &mut Lexer<'a, C>,
    _pos: SourcePosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    unreachable!();
}
