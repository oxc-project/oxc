use oxc_ast::CommentKind;
use oxc_data_structures::assert_unchecked;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{Kind, Lexer, ScanState};

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
    pub(super) unsafe fn handle_byte(&mut self, state: &mut ScanState, byte: u8) -> Kind {
        match byte {
            b'(' | b')' | b';' => self.read_fixed_punctuation(byte),
            _ => Self::STATEFUL_HANDLERS[byte as usize](self, state),
        }
    }

    #[inline]
    fn read_fixed_punctuation(&mut self, byte: u8) -> Kind {
        let kind = match byte {
            b'(' => Kind::LParen,
            b')' => Kind::RParen,
            b';' => Kind::Semicolon,
            _ => unreachable!(),
        };
        // SAFETY: The caller classified a single-byte ASCII punctuation token.
        self.source.set_position(unsafe { self.source.position().add(1) });
        kind
    }
}

/// Macro to create a lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/e55a1310ba69a3f5ce2a9a6eef643feced02ac08/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
macro_rules! byte_handlers {
    ($adapter:ident) => {
        $adapter!([
        //  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
            ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, ISP, ISP, LIN, ERR, ERR, // 0
            ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
            SPS, EXL, QOD, HAS, IDT, PRC, AMP, QOS, UER, UER, ATR, PLS, COM, MIN, PRD, SLH, // 2
            ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, UER, LSS, EQL, GTR, QST, // 3
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
        ])
    };
}

macro_rules! stateful_handler {
    ([$($handler:ident),* $(,)?]) => { [$(|lexer, state| $handler(lexer, state)),*] };
}

impl<C: Config> Lexer<'_, C> {
    const STATEFUL_HANDLERS: [fn(&mut Self, &mut ScanState) -> Kind; 256] =
        byte_handlers!(stateful_handler);
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
/// ascii_byte_handler!(SPS(lexer, state) {
///     lexer.consume_char();
///     Kind::WhiteSpace
/// });
/// ```
///
/// expands to:
///
/// ```rust,ignore
/// #[expect(non_snake_case)]
/// fn SPS<C: Config>(lexer: &mut Lexer<'_, C>, state: &mut ScanState) -> Kind {
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
    ($id:ident($lex:ident, $state:ident) $body:expr) => {
        #[expect(non_snake_case)]
        fn $id<C: Config>($lex: &mut Lexer<'_, C>, $state: &mut ScanState) -> Kind {
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                assert_unchecked!(!$lex.source.is_eof());
                assert_unchecked!($lex.source.peek_byte_unchecked() < 128);
            }
            $body
        }
    };
}

/// Macro for defining byte handler for an ASCII character which is start of an identifier
/// (`a`-`z`, `A`-`Z`, `$` or `_`).
///
/// Macro calls `Lexer::identifier_name_handler` to get the text of the identifier,
/// minus its first character.
///
/// `Lexer::identifier_name_handler` is an unsafe function, but if byte being consumed is ASCII,
/// its requirements are met.
///
/// # SAFETY
/// Only use this macro to define byte handlers for ASCII characters.
///
/// ```rust,ignore
/// ascii_identifier_handler!(L_G(id_without_first_char, state) match id_without_first_char.as_bytes() {
///     b"et" => Kind::Get,
///     b"lobal" => Kind::Global,
///     _ => Kind::Ident,
/// });
/// ```
///
/// expands to:
///
/// ```rust,ignore
/// #[expect(non_snake_case)]
/// fn L_G<C: Config>(lexer: &mut Lexer<'_, C>, state: &mut ScanState) -> Kind {
///     // SAFETY: This macro is only used for ASCII characters
///     let id_without_first_char = unsafe { lexer.identifier_name_handler(state) };
///     match id_without_first_char.as_bytes() {
///         b"et" => Kind::Get,
///         b"lobal" => Kind::Global,
///         _ => Kind::Ident,
///     }
/// }
/// ```
macro_rules! ascii_identifier_handler {
    ($id:ident($str:ident, $state:ident) $body:expr) => {
        #[expect(non_snake_case)]
        #[inline]
        fn $id<C: Config>(lexer: &mut Lexer<'_, C>, $state: &mut ScanState) -> Kind {
            // SAFETY: The table only calls identifier handlers for ASCII identifier starts.
            let $str = unsafe { lexer.identifier_name_handler($state) };
            $body
        }
    };
}

// `\0` `\1` etc
ascii_byte_handler!(ERR(lexer, state) {
    let c = lexer.consume_char();
    lexer.error(diagnostics::invalid_character(c, lexer.unterminated_range(*state)));
    Kind::Undetermined
});

// <SPACE> <TAB> Normal Whitespace
ascii_byte_handler!(SPS(lexer, _state) {
    lexer.whitespace_handler()
});

// <VT> <FF> Irregular Whitespace
ascii_byte_handler!(ISP(lexer, state) {
    lexer.consume_char();
    lexer.trivia_builder.add_irregular_whitespace(state.start(), lexer.offset());
    Kind::Skip
});

// '\r' '\n'
ascii_byte_handler!(LIN(lexer, state) {
    lexer.consume_char();
    lexer.line_break_handler(state)
});

// !
ascii_byte_handler!(EXL(lexer, _state) {
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
ascii_byte_handler!(QOD(lexer, state) {
    // SAFETY: This function is only called for `"`
    unsafe { lexer.read_string_literal_double_quote(state) }
});

// '
ascii_byte_handler!(QOS(lexer, state) {
    // SAFETY: This function is only called for `'`
    unsafe { lexer.read_string_literal_single_quote(state) }
});

// #
ascii_byte_handler!(HAS(lexer, state) {
    lexer.consume_char();
    lexer.private_identifier(state)
});

// `A..=Z`, `a..=z` (except special cases below), `_`, `$`
ascii_byte_handler!(IDT(lexer, state) {
    // SAFETY: This handler is only called for ASCII identifier starts.
    unsafe { lexer.identifier_name_handler(state) };
    Kind::Ident
});

// %
ascii_byte_handler!(PRC(lexer, _state) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
});

// &
ascii_byte_handler!(AMP(lexer, _state) {
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
ascii_byte_handler!(ATR(lexer, _state) {
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
ascii_byte_handler!(PLS(lexer, _state) {
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
ascii_byte_handler!(COM(lexer, _state) {
    lexer.consume_char();
    Kind::Comma
});

// -
ascii_byte_handler!(MIN(lexer, state) {
    lexer.consume_char();
    lexer.read_minus(*state).unwrap_or_else(|| { lexer.skip_single_line_comment(state, CommentKind::HtmlClose) })
});

// .
ascii_byte_handler!(PRD(lexer, state) {
    lexer.consume_char();
    lexer.read_dot(state)
});

// /
ascii_byte_handler!(SLH(lexer, state) {
    lexer.consume_char();
    match lexer.peek_byte() {
        Some(b'/') => {
            lexer.consume_char();
            lexer.skip_single_line_comment(state, CommentKind::Line)
        }
        Some(b'*') => {
            lexer.consume_char();
            lexer.skip_multi_line_comment(state)
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
ascii_byte_handler!(ZER(lexer, state) {
    lexer.consume_char();
    lexer.read_zero(state)
});

// 1 to 9
ascii_byte_handler!(DIG(lexer, state) {
    lexer.consume_char();
    lexer.decimal_literal_after_first_digit(state)
});

// :
ascii_byte_handler!(COL(lexer, _state) {
    lexer.consume_char();
    Kind::Colon
});

// <
ascii_byte_handler!(LSS(lexer, state) {
    lexer.consume_char();
    lexer.read_left_angle(*state).unwrap_or_else(|| { lexer.skip_single_line_comment(state, CommentKind::HtmlOpen) })
});

// =
ascii_byte_handler!(EQL(lexer, _state) {
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
ascii_byte_handler!(GTR(lexer, _state) {
    lexer.consume_char();
    // `>=` is re-lexed with [Lexer::next_jsx_child]
    Kind::RAngle
});

// ?
ascii_byte_handler!(QST(lexer, _state) {
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
ascii_byte_handler!(AT_(lexer, _state) {
    lexer.consume_char();
    Kind::At
});

// [
ascii_byte_handler!(BTO(lexer, _state) {
    lexer.consume_char();
    Kind::LBrack
});

// \
ascii_byte_handler!(ESC(lexer, state) {
    lexer.identifier_backslash_handler(state)
});

// ]
ascii_byte_handler!(BTC(lexer, _state) {
    lexer.consume_char();
    Kind::RBrack
});

// ^
ascii_byte_handler!(CRT(lexer, _state) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::CaretEq
    } else {
        Kind::Caret
    }
});

// `
ascii_byte_handler!(TPL(lexer, state) {
    lexer.consume_char();
    lexer.read_template_literal(state, Kind::TemplateHead, Kind::NoSubstitutionTemplate)
});

// {
ascii_byte_handler!(BEO(lexer, _state) {
    lexer.consume_char();
    Kind::LCurly
});

// |
ascii_byte_handler!(PIP(lexer, _state) {
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
ascii_byte_handler!(BEC(lexer, _state) {
    lexer.consume_char();
    Kind::RCurly
});

// ~
ascii_byte_handler!(TLD(lexer, _state) {
    lexer.consume_char();
    Kind::Tilde
});

// Byte patterns let the compiler compare fixed-size chunks without calling a string comparison
// routine for keyword lengths that are not a power of two.
ascii_identifier_handler!(L_A(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"wait" => Kind::Await,
    b"sync" => Kind::Async,
    b"bstract" => Kind::Abstract,
    b"ccessor" => Kind::Accessor,
    b"ny" => Kind::Any,
    b"s" => Kind::As,
    b"ssert" => Kind::Assert,
    b"sserts" => Kind::Asserts,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_B(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"reak" => Kind::Break,
    b"oolean" => Kind::Boolean,
    b"igint" => Kind::BigInt,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_C(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"onst" => Kind::Const,
    b"lass" => Kind::Class,
    b"ontinue" => Kind::Continue,
    b"atch" => Kind::Catch,
    b"ase" => Kind::Case,
    b"onstructor" => Kind::Constructor,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_D(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"o" => Kind::Do,
    b"elete" => Kind::Delete,
    b"eclare" => Kind::Declare,
    b"efault" => Kind::Default,
    b"ebugger" => Kind::Debugger,
    b"efer" => Kind::Defer,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_E(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"lse" => Kind::Else,
    b"num" => Kind::Enum,
    b"xport" => Kind::Export,
    b"xtends" => Kind::Extends,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_F(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"unction" => Kind::Function,
    b"alse" => Kind::False,
    b"or" => Kind::For,
    b"inally" => Kind::Finally,
    b"rom" => Kind::From,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_G(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"et" => Kind::Get,
    b"lobal" => Kind::Global,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_I(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"f" => Kind::If,
    b"nstanceof" => Kind::Instanceof,
    b"n" => Kind::In,
    b"mplements" => Kind::Implements,
    b"mport" => Kind::Import,
    b"nfer" => Kind::Infer,
    b"nterface" => Kind::Interface,
    b"ntrinsic" => Kind::Intrinsic,
    b"s" => Kind::Is,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_K(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"eyof" => Kind::KeyOf,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_L(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"et" => Kind::Let,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_M(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"eta" => Kind::Meta,
    b"odule" => Kind::Module,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_N(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"ull" => Kind::Null,
    b"ew" => Kind::New,
    b"umber" => Kind::Number,
    b"amespace" => Kind::Namespace,
    b"ever" => Kind::Never,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_O(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"f" => Kind::Of,
    b"bject" => Kind::Object,
    b"ut" => Kind::Out,
    b"verride" => Kind::Override,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_P(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"ackage" => Kind::Package,
    b"rivate" => Kind::Private,
    b"rotected" => Kind::Protected,
    b"ublic" => Kind::Public,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_R(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"eturn" => Kind::Return,
    b"equire" => Kind::Require,
    b"eadonly" => Kind::Readonly,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_S(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"et" => Kind::Set,
    b"uper" => Kind::Super,
    b"witch" => Kind::Switch,
    b"tatic" => Kind::Static,
    b"ymbol" => Kind::Symbol,
    b"tring" => Kind::String,
    b"atisfies" => Kind::Satisfies,
    b"ource" => Kind::Source,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_T(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"his" => Kind::This,
    b"rue" => Kind::True,
    b"hrow" => Kind::Throw,
    b"ry" => Kind::Try,
    b"ypeof" => Kind::Typeof,
    b"arget" => Kind::Target,
    b"ype" => Kind::Type,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_U(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"ndefined" => Kind::Undefined,
    b"sing" => Kind::Using,
    b"nique" => Kind::Unique,
    b"nknown" => Kind::Unknown,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_V(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"ar" => Kind::Var,
    b"oid" => Kind::Void,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_W(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"hile" => Kind::While,
    b"ith" => Kind::With,
    _ => Kind::Ident,
});

ascii_identifier_handler!(L_Y(id_without_first_char, state) match id_without_first_char.as_bytes() {
    b"ield" => Kind::Yield,
    _ => Kind::Ident,
});

// Non-ASCII characters.
//
// Note: Must not use `ascii_byte_handler!` macro, as this handler is for non-ASCII chars.
#[expect(non_snake_case)]
fn UNI<C: Config>(lexer: &mut Lexer<'_, C>, state: &mut ScanState) -> Kind {
    lexer.unicode_char_handler(state)
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
fn UER<C: Config>(_lexer: &mut Lexer<'_, C>, _state: &mut ScanState) -> Kind {
    unreachable!();
}

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;
    use oxc_span::SourceType;

    use super::*;
    use crate::{
        Parser, UniquePromise,
        config::{ParserConfig, RuntimeParserConfig, TokensLexerConfig, TokensParserConfig},
    };

    #[derive(Clone, Copy, Default)]
    struct CustomConfig(bool);

    impl Config for CustomConfig {
        const TOKENS_METHOD_IS_STATIC: bool = false;

        fn tokens(&self) -> bool {
            self.0
        }
    }

    impl ParserConfig for CustomConfig {
        type LexerConfig = Self;
        fn lexer_config(&self) -> Self {
            *self
        }
    }

    #[test]
    fn parser_configs_agree_on_tokens_flags_diagnostics_and_comments() {
        // Exercise the configuration boundary and retokenization, including failed speculation.
        let cases = [
            ("/* @__PURE__ */ f(); g(); (a /* trailing */ + b);", SourceType::mjs()),
            ("function f(){return/*\n*/ /[[]/g;} a / b;", SourceType::mjs()),
            ("let a\\u0062 = 'x\\n'; let π = 1_000; let z = 3;", SourceType::mjs()),
            ("const x = `a${b}c${d}e`; tag`\\u{}`;", SourceType::mjs()),
            ("type X = A<B<C>>; f<<T>() => T>(); a << b; a >>> b;", SourceType::ts()),
            (
                "const f = <T,>(x:T) => x; const x = <x-y a=\"\\q\">text{v}</x-y>;",
                SourceType::tsx(),
            ),
            ("async (a /* comment */ + b); (a /* comment */ + b);", SourceType::ts()),
            ("<!-- comment\nlet x = 1;", SourceType::default().with_unambiguous(true)),
            ("let\u{85}x = 1; let\u{200b}y = 2;", SourceType::mjs()),
            ("const x = 3in y;", SourceType::mjs()),
            ("const x = 'unterminated", SourceType::mjs()),
            ("`bad${1", SourceType::mjs()),
            ("", SourceType::mjs()),
            ("#!/usr/bin/env node\n/* tail */", SourceType::mjs()),
        ];
        for (source, source_type) in cases {
            let expected_allocator = Allocator::default();
            let expected = Parser::new(&expected_allocator, source, source_type)
                .with_config(TokensParserConfig)
                .parse();
            for tokens in [false, true] {
                let custom_allocator = Allocator::default();
                let custom = Parser::new(&custom_allocator, source, source_type)
                    .with_config(CustomConfig(tokens))
                    .parse();
                let runtime_allocator = Allocator::default();
                let runtime = Parser::new(&runtime_allocator, source, source_type)
                    .with_config(RuntimeParserConfig::new(tokens))
                    .parse();
                for result in [&custom, &runtime] {
                    assert_eq!(
                        format!("{:?}", result.program),
                        format!("{:?}", expected.program),
                        "{source}"
                    );
                    assert_eq!(
                        format!("{:?}", result.diagnostics),
                        format!("{:?}", expected.diagnostics),
                        "{source}"
                    );
                    assert_eq!(
                        result.irregular_whitespaces, expected.irregular_whitespaces,
                        "{source}"
                    );
                    assert_eq!(result.fatal_error, expected.fatal_error, "{source}");
                    if tokens {
                        assert_eq!(result.tokens, expected.tokens, "{source}");
                    } else {
                        assert!(result.tokens.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn checkpoint_restores_replaced_token() {
        let allocator = Allocator::default();
        let mut lexer = Lexer::new(
            &allocator,
            "a << T /* annotation */ > c",
            SourceType::ts(),
            TokensLexerConfig,
            UniquePromise::new_for_tests_and_benchmarks(),
        );
        lexer.first_token();
        let shift = lexer.next_token();
        let checkpoint = lexer.checkpoint();
        assert_eq!(lexer.re_lex_as_typescript_l_angle(shift, 2).kind(), Kind::LAngle);
        lexer.next_token();
        lexer.next_token();
        lexer.next_token();
        lexer.rewind(checkpoint);
        assert_eq!(lexer.tokens.last().copied(), Some(shift));
        assert_eq!(lexer.peek_token().kind(), Kind::Ident);
        assert_eq!(lexer.next_token().kind(), Kind::Ident);
    }

    #[test]
    fn rescans_preserve_line_breaks_without_appending_tokens() {
        let allocator = Allocator::default();
        let mut lexer = Lexer::new(
            &allocator,
            "\n/abc/g",
            SourceType::mjs(),
            TokensLexerConfig,
            UniquePromise::new_for_tests_and_benchmarks(),
        );
        let slash = lexer.first_token();
        let (regexp, _, _) = lexer.next_regex(slash);
        assert!(regexp.is_on_new_line());
        assert_eq!(lexer.tokens.as_slice(), &[regexp]);
    }
}
