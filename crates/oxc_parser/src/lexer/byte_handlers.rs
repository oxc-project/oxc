use super::{AutoCow, Kind, Lexer, LexerContext};
use crate::diagnostics;

#[allow(clippy::unnecessary_safety_comment)]
/// Handle next byte of source.
///
/// SAFETY:
/// * Lexer must not be at end of file.
/// * `byte` must be next byte of source code, corresponding to current position
///   of `lexer.current.chars`.
/// * Only `BYTE_HANDLERS` for ASCII characters may use the `ascii_byte_handler!()` macro.
pub(super) unsafe fn handle_byte(byte: u8, lexer: &mut Lexer) -> Kind {
    BYTE_HANDLERS[byte as usize](lexer)
}

type ByteHandler = unsafe fn(&mut Lexer<'_>) -> Kind;

/// Lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/master/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
static BYTE_HANDLERS: [ByteHandler; 256] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, SPS, SPS, LIN, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    SPS, EXL, QOT, HAS, IDT, PRC, AMP, QOT, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, GTR, QST, // 3
    AT_, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, ESC, BTC, CRT, IDT, // 5
    TPL, L_A, L_B, L_C, L_D, L_E, L_F, L_G, IDT, L_I, IDT, L_K, L_L, L_M, L_N, L_O, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];

#[allow(clippy::unnecessary_safety_comment)]
/// Macro for defining byte handler for an ASCII character.
///
/// In addition to defining a `const` for the handler, it also asserts that lexer
/// is not at end of file, and that next char is ASCII.
/// Where the handler is for an ASCII character, these assertions are self-evidently true.
///
/// These assertions produce no runtime code, but hint to the compiler that it can assume that
/// next char is ASCII, and it uses that information to optimize the rest of the handler.
/// e.g. `lexer.current.chars.next()` becomes just a single assembler instruction.
/// Without the assertions, the compiler is unable to deduce the next char is ASCII, due to
/// the indirection of the `BYTE_HANDLERS` jump table.
///
/// These assertions are unchecked (i.e. won't panic) and will cause UB if they're incorrect.
///
/// SAFETY: Only use this macro to define byte handlers for ASCII characters.
///
/// ```
/// ascii_byte_handler!(SPS(lexer) {
///   lexer.consume_char();
///   Kind::WhiteSpace
/// });
/// ```
///
/// expands to:
///
/// ```
/// const SPS: ByteHandler = |lexer| {
///   unsafe {
///     use assert_unchecked::assert_unchecked;
///     let s = lexer.current.chars.as_str();
///     assert_unchecked!(!s.is_empty());
///     assert_unchecked!(s.as_bytes()[0] < 128);
///   }
///   lexer.consume_char();
///   Kind::WhiteSpace
/// };
/// ```
macro_rules! ascii_byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        const $id: ByteHandler = |$lex| {
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                use assert_unchecked::assert_unchecked;
                let s = $lex.current.chars.as_str();
                assert_unchecked!(!s.is_empty());
                assert_unchecked!(s.as_bytes()[0] < 128);
            }
            $body
        };
    };
}

// `\0` `\1` etc
ascii_byte_handler!(ERR(lexer) {
    let c = lexer.consume_char();
    lexer.error(diagnostics::InvalidCharacter(c, lexer.unterminated_range()));
    Kind::Undetermined
});

// <SPACE> <TAB> <VT> <FF>
ascii_byte_handler!(SPS(lexer) {
    lexer.consume_char();
    Kind::Skip
});

// '\r' '\n'
ascii_byte_handler!(LIN(lexer) {
    lexer.consume_char();
    lexer.current.token.is_on_new_line = true;
    Kind::Skip
});

// !
ascii_byte_handler!(EXL(lexer) {
    lexer.consume_char();
    if lexer.next_eq('=') {
        if lexer.next_eq('=') {
            Kind::Neq2
        } else {
            Kind::Neq
        }
    } else {
        Kind::Bang
    }
});

// ' "
ascii_byte_handler!(QOT(lexer) {
    let c = lexer.consume_char();
    if lexer.context == LexerContext::JsxAttributeValue {
        lexer.read_jsx_string_literal(c)
    } else {
        lexer.read_string_literal(c)
    }
});

// #
ascii_byte_handler!(HAS(lexer) {
    lexer.consume_char();
    // HashbangComment ::
    //     `#!` SingleLineCommentChars?
    if lexer.current.token.start == 0 && lexer.next_eq('!') {
        lexer.read_hashbang_comment()
    } else {
        lexer.private_identifier()
    }
});

// `A..=Z`, `a..=z` (except special cases below), `_`, `$`
ascii_byte_handler!(IDT(lexer) {
    lexer.identifier_name_handler();
    Kind::Ident
});

// %
ascii_byte_handler!(PRC(lexer) {
    lexer.consume_char();
    if lexer.next_eq('=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
});

// &
ascii_byte_handler!(AMP(lexer) {
    lexer.consume_char();
    if lexer.next_eq('&') {
        if lexer.next_eq('=') {
            Kind::Amp2Eq
        } else {
            Kind::Amp2
        }
    } else if lexer.next_eq('=') {
        Kind::AmpEq
    } else {
        Kind::Amp
    }
});

// (
ascii_byte_handler!(PNO(lexer) {
    lexer.consume_char();
    Kind::LParen
});

// )
ascii_byte_handler!(PNC(lexer) {
    lexer.consume_char();
    Kind::RParen
});

// *
ascii_byte_handler!(ATR(lexer) {
    lexer.consume_char();
    if lexer.next_eq('*') {
        if lexer.next_eq('=') {
            Kind::Star2Eq
        } else {
            Kind::Star2
        }
    } else if lexer.next_eq('=') {
        Kind::StarEq
    } else {
        Kind::Star
    }
});

// +
ascii_byte_handler!(PLS(lexer) {
    lexer.consume_char();
    if lexer.next_eq('+') {
        Kind::Plus2
    } else if lexer.next_eq('=') {
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
    lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment())
});

// .
ascii_byte_handler!(PRD(lexer) {
    lexer.consume_char();
    lexer.read_dot()
});

// /
ascii_byte_handler!(SLH(lexer) {
    lexer.consume_char();
    match lexer.peek() {
        Some('/') => {
            lexer.current.chars.next();
            lexer.skip_single_line_comment()
        }
        Some('*') => {
            lexer.current.chars.next();
            lexer.skip_multi_line_comment()
        }
        _ => {
            // regex is handled separately, see `next_regex`
            if lexer.next_eq('=') {
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

// ;
ascii_byte_handler!(SEM(lexer) {
    lexer.consume_char();
    Kind::Semicolon
});

// <
ascii_byte_handler!(LSS(lexer) {
    lexer.consume_char();
    lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment())
});

// =
ascii_byte_handler!(EQL(lexer) {
    lexer.consume_char();
    if lexer.next_eq('=') {
        if lexer.next_eq('=') {
            Kind::Eq3
        } else {
            Kind::Eq2
        }
    } else if lexer.next_eq('>') {
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
    if lexer.next_eq('?') {
        if lexer.next_eq('=') {
            Kind::Question2Eq
        } else {
            Kind::Question2
        }
    } else if lexer.peek() == Some('.') {
        // parse `?.1` as `?` `.1`
        if lexer.peek2().is_some_and(|c| c.is_ascii_digit()) {
            Kind::Question
        } else {
            lexer.current.chars.next();
            Kind::QuestionDot
        }
    } else {
        Kind::Question
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
    let mut builder = AutoCow::new(lexer);
    lexer.consume_char();
    builder.force_allocation_without_current_ascii_char(lexer);
    lexer.identifier_unicode_escape_sequence(&mut builder, true);
    let text = lexer.identifier_name(builder);
    Kind::match_keyword(text)
});

// ]
ascii_byte_handler!(BTC(lexer) {
    lexer.consume_char();
    Kind::RBrack
});

// ^
ascii_byte_handler!(CRT(lexer) {
    lexer.consume_char();
    if lexer.next_eq('=') {
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
    if lexer.next_eq('|') {
        if lexer.next_eq('=') {
            Kind::Pipe2Eq
        } else {
            Kind::Pipe2
        }
    } else if lexer.next_eq('=') {
        Kind::PipeEq
    } else {
        Kind::Pipe
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

ascii_byte_handler!(L_A(lexer) match &lexer.identifier_name_handler()[1..] {
    "wait" => Kind::Await,
    "sync" => Kind::Async,
    "bstract" => Kind::Abstract,
    "ccessor" => Kind::Accessor,
    "ny" => Kind::Any,
    "s" => Kind::As,
    "ssert" => Kind::Assert,
    "sserts" => Kind::Asserts,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_B(lexer) match &lexer.identifier_name_handler()[1..] {
    "reak" => Kind::Break,
    "oolean" => Kind::Boolean,
    "igint" => Kind::BigInt,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_C(lexer) match &lexer.identifier_name_handler()[1..] {
    "onst" => Kind::Const,
    "lass" => Kind::Class,
    "ontinue" => Kind::Continue,
    "atch" => Kind::Catch,
    "ase" => Kind::Case,
    "onstructor" => Kind::Constructor,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_D(lexer) match &lexer.identifier_name_handler()[1..] {
    "o" => Kind::Do,
    "elete" => Kind::Delete,
    "eclare" => Kind::Declare,
    "efault" => Kind::Default,
    "ebugger" => Kind::Debugger,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_E(lexer) match &lexer.identifier_name_handler()[1..] {
    "lse" => Kind::Else,
    "num" => Kind::Enum,
    "xport" => Kind::Export,
    "xtends" => Kind::Extends,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_F(lexer) match &lexer.identifier_name_handler()[1..] {
    "unction" => Kind::Function,
    "alse" => Kind::False,
    "or" => Kind::For,
    "inally" => Kind::Finally,
    "rom" => Kind::From,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_G(lexer) match &lexer.identifier_name_handler()[1..] {
    "et" => Kind::Get,
    "lobal" => Kind::Global,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_I(lexer) match &lexer.identifier_name_handler()[1..] {
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
});

ascii_byte_handler!(L_K(lexer) match &lexer.identifier_name_handler()[1..] {
    "eyof" => Kind::KeyOf,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_L(lexer) match &lexer.identifier_name_handler()[1..] {
    "et" => Kind::Let,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_M(lexer) match &lexer.identifier_name_handler()[1..] {
    "eta" => Kind::Meta,
    "odule" => Kind::Module,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_N(lexer) match &lexer.identifier_name_handler()[1..] {
    "ull" => Kind::Null,
    "ew" => Kind::New,
    "umber" => Kind::Number,
    "amespace" => Kind::Namespace,
    "ever" => Kind::Never,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_O(lexer) match &lexer.identifier_name_handler()[1..] {
    "f" => Kind::Of,
    "bject" => Kind::Object,
    "ut" => Kind::Out,
    "verride" => Kind::Override,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_P(lexer) match &lexer.identifier_name_handler()[1..] {
    "ackage" => Kind::Package,
    "rivate" => Kind::Private,
    "rotected" => Kind::Protected,
    "ublic" => Kind::Public,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_R(lexer) match &lexer.identifier_name_handler()[1..] {
    "eturn" => Kind::Return,
    "equire" => Kind::Require,
    "eadonly" => Kind::Readonly,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_S(lexer) match &lexer.identifier_name_handler()[1..] {
    "et" => Kind::Set,
    "uper" => Kind::Super,
    "witch" => Kind::Switch,
    "tatic" => Kind::Static,
    "ymbol" => Kind::Symbol,
    "tring" => Kind::String,
    "atisfies" => Kind::Satisfies,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_T(lexer) match &lexer.identifier_name_handler()[1..] {
    "his" => Kind::This,
    "rue" => Kind::True,
    "hrow" => Kind::Throw,
    "ry" => Kind::Try,
    "ypeof" => Kind::Typeof,
    "arget" => Kind::Target,
    "ype" => Kind::Type,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_U(lexer) match &lexer.identifier_name_handler()[1..] {
    "ndefined" => Kind::Undefined,
    "sing" => Kind::Using,
    "nique" => Kind::Unique,
    "nknown" => Kind::Unknown,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_V(lexer) match &lexer.identifier_name_handler()[1..] {
    "ar" => Kind::Var,
    "oid" => Kind::Void,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_W(lexer) match &lexer.identifier_name_handler()[1..] {
    "hile" => Kind::While,
    "ith" => Kind::With,
    _ => Kind::Ident,
});

ascii_byte_handler!(L_Y(lexer) match &lexer.identifier_name_handler()[1..] {
    "ield" => Kind::Yield,
    _ => Kind::Ident,
});

// Non-ASCII characters.
// NB: Must not use `ascii_byte_handler!()` macro, as this handler is for non-ASCII chars.
#[allow(clippy::redundant_closure_for_method_calls)]
const UNI: ByteHandler = |lexer| lexer.unicode_char_handler();
