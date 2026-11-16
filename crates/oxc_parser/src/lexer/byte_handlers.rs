use oxc_ast::CommentKind;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{AsciiPosition, Kind, Lexer, SourcePosition};

impl<'a, C: Config> Lexer<'a, C> {
    /// Handle the ASCII byte `byte`, which is the byte at `start`.
    ///
    /// Position of source is set to `start`, then moved past whatever the byte begins.
    /// Returns `Kind::Skip` if that is whitespace or a comment.
    // `#[inline(always)]` to ensure is inlined into `read_next_jsx_attribute_value`
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) fn handle_ascii_byte(&mut self, start: AsciiPosition<'a>, byte: u8) -> Kind {
        if let Some(kind) = single_byte_token(byte) {
            self.source.set_position(start.after());
            return kind;
        }
        Self::byte_handler(byte)(self, start).0
    }

    /// Get the byte handler for an ASCII byte.
    ///
    /// The table has 1 handler for each ASCII byte. `byte` is taken modulo 128, which costs
    /// nothing where the compiler can see `byte` is ASCII, which callers have always checked.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(super) fn byte_handler(byte: u8) -> ByteHandler<C> {
        let byte_handlers: &ByteHandlers<C> = &Self::BYTE_HANDLERS;
        byte_handlers[(byte & 0x7F) as usize]
    }

    /// Byte handler for each ASCII byte.
    ///
    /// The handlers are generic over the lexer's config, so there is one table for each config.
    /// A reference to an associated constant is to static memory, so getting a handler
    /// from the table is a single load.
    const BYTE_HANDLERS: ByteHandlers<C> = ascii_handlers!();
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

/// Byte handlers take the position of an ASCII byte. They set position of `lexer.source` to it,
/// then move past whatever the byte begins.
///
/// They return the `Kind` of the token, or `Kind::Skip` for whitespace and comments,
/// along with the position of `lexer.source`, so it stays in a register between tokens
/// instead of being stored and reloaded.
///
/// Characters which are not ASCII have no byte handler. `Lexer::unicode_char_handler` lexes them.
pub(super) type ByteHandler<C> =
    for<'a> fn(&mut Lexer<'a, C>, AsciiPosition<'a>) -> (Kind, SourcePosition<'a>);
pub(super) type ByteHandlers<C> = [ByteHandler<C>; 128];

/// Macro to create a lookup table mapping each ASCII byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/e55a1310ba69a3f5ce2a9a6eef643feced02ac08/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
macro_rules! ascii_handlers {
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
        ]
    };
}
use ascii_handlers;

/// Macro for defining a byte handler.
///
/// Moves `lexer.source` to the ASCII byte which the handler is given the position of.
/// The compiler then knows that the source is not at its end, and that the next char is ASCII,
/// so e.g. `lexer.consume_char()` becomes just a single assembly instruction.
///
/// ```rust,ignore
/// byte_handler!(SPS(lexer) {
///     lexer.consume_char();
///     Kind::WhiteSpace
/// });
/// ```
///
/// expands to:
///
/// ```rust,ignore
/// #[expect(non_snake_case)]
/// fn SPS<'a, C: Config>(
///     lexer: &mut Lexer<'a, C>,
///     start: AsciiPosition<'a>,
/// ) -> (Kind, SourcePosition<'a>) {
///     lexer.source.set_position_to_ascii(start);
///     let kind = {
///         lexer.consume_char();
///         Kind::WhiteSpace
///     };
///     (kind, lexer.source.position())
/// }
/// ```
macro_rules! byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        #[expect(non_snake_case)]
        fn $id<'a, C: Config>(
            $lex: &mut Lexer<'a, C>,
            start: AsciiPosition<'a>,
        ) -> (Kind, SourcePosition<'a>) {
            $lex.source.set_position_to_ascii(start);
            let kind = $body;
            (kind, $lex.source.position())
        }
    };
}

// `\0` `\1` etc
byte_handler!(ERR(lexer) {
    let c = lexer.consume_char();
    lexer.error(diagnostics::invalid_character(c, lexer.unterminated_range()));
    Kind::Undetermined
});

// <SPACE> <TAB> Normal Whitespace
byte_handler!(SPS(lexer) {
    lexer.whitespace_handler()
});

// <VT> <FF> Irregular Whitespace
byte_handler!(ISP(lexer) {
    lexer.consume_char();
    lexer.trivia_builder.add_irregular_whitespace(lexer.token_start, lexer.offset());
    Kind::Skip
});

// '\r' '\n'
#[expect(non_snake_case)]
fn LIN<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    start: AsciiPosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    lexer.source.set_position(start.after());
    lexer.line_break_handler();

    // Read the next token here to avoid restarting the caller's token loop.
    let pos = lexer.source.position();
    lexer.token_start = lexer.source.offset_of(pos);
    let Some((start, byte)) = lexer.source.ascii_position(pos) else {
        // End of the source, or a Unicode character, which the caller's token loop handles
        let kind = if pos.is_end_of(&lexer.source) { Kind::Eof } else { Kind::Skip };
        return (kind, pos);
    };
    // JSX attribute values scan quotes differently. Leave them for the caller's token loop.
    if matches!(byte, b'"' | b'\'') {
        return (Kind::Skip, pos);
    }
    if let Some(kind) = single_byte_token(byte) {
        let end = start.after();
        lexer.source.set_position(end);
        return (kind, end);
    }
    Lexer::byte_handler(byte)(lexer, start)
}

// !
byte_handler!(EXL(lexer) {
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
#[expect(non_snake_case)]
fn QOD<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    start: AsciiPosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    let kind = lexer.read_string_literal_double_quote(start);
    (kind, lexer.source.position())
}

// '
#[expect(non_snake_case)]
fn QOS<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    start: AsciiPosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    let kind = lexer.read_string_literal_single_quote(start);
    (kind, lexer.source.position())
}

// #
byte_handler!(HAS(lexer) {
    lexer.consume_char();
    lexer.private_identifier()
});

// `A..=Z`, `h`, `j`, `q`, `x`, `z`, `_`, `$`: no keyword starts with these bytes.
#[expect(non_snake_case)]
fn IDT<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    start: AsciiPosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    lexer.identifier_name_handler::<false>(start)
}

// Share one indirect-call target for all first letters which can begin a keyword.
#[expect(non_snake_case)]
fn KWD<'a, C: Config>(
    lexer: &mut Lexer<'a, C>,
    start: AsciiPosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    lexer.identifier_name_handler::<true>(start)
}

// %
byte_handler!(PRC(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
});

// &
byte_handler!(AMP(lexer) {
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
byte_handler!(ATR(lexer) {
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
byte_handler!(PLS(lexer) {
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
byte_handler!(MIN(lexer) {
    lexer.consume_char();
    lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment(CommentKind::HtmlClose))
});

// .
byte_handler!(PRD(lexer) {
    lexer.consume_char();
    lexer.read_dot()
});

// /
byte_handler!(SLH(lexer) {
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
byte_handler!(ZER(lexer) {
    lexer.consume_char();
    lexer.read_zero()
});

// 1 to 9
byte_handler!(DIG(lexer) {
    lexer.consume_char();
    lexer.decimal_literal_after_first_digit()
});

// <
byte_handler!(LSS(lexer) {
    lexer.consume_char();
    lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment(CommentKind::HtmlOpen))
});

// =
byte_handler!(EQL(lexer) {
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
byte_handler!(QST(lexer) {
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
byte_handler!(ESC(lexer) {
    lexer.identifier_backslash_handler()
});

// ^
byte_handler!(CRT(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::CaretEq
    } else {
        Kind::Caret
    }
});

// `
byte_handler!(TPL(lexer) {
    lexer.consume_char();
    lexer.read_template_literal(Kind::TemplateHead, Kind::NoSubstitutionTemplate)
});

// |
byte_handler!(PIP(lexer) {
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

// Single-byte tokens, which `SINGLE_BYTE_TOKENS` contains, so they never reach a handler.
#[expect(non_snake_case)]
fn UER<'a, C: Config>(
    _lexer: &mut Lexer<'a, C>,
    _start: AsciiPosition<'a>,
) -> (Kind, SourcePosition<'a>) {
    unreachable!();
}
