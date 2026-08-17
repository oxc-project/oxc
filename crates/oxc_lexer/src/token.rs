macro_rules! define_token_kind {
    ($( $variant:ident = $value:literal => $name:literal ),+ $(,)?) => {
        /// A lexed token kind.
        ///
        /// The discriminants are load-bearing: `[32, 128)` is reserved for
        /// punctuators and `>= 128` for keywords, so the pipeline can classify
        /// with range checks and SIMD compares. They are *not* dense â€” the
        /// pipeline computes kinds arithmetically and blends them in SIMD
        /// registers, so it works on the raw `u8` and only the crate boundary
        /// is typed.
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum TokenKind {
            $( $variant = $value, )+
        }

        impl TokenKind {
            /// Every declared kind, in discriminant order.
            pub const VARIANTS: &'static [TokenKind] = &[ $( TokenKind::$variant, )+ ];

            /// The kind `byte` denotes, or `None` if it is not a declared discriminant.
            #[inline]
            #[must_use]
            pub const fn from_u8(byte: u8) -> Option<Self> {
                match byte {
                    $( $value => Some(Self::$variant), )+
                    _ => None,
                }
            }

            /// The token's spelling for punctuators and keywords, else the kind's name.
            #[inline]
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $( Self::$variant => $name, )+
                }
            }
        }
    };
}

define_token_kind! {
    Eof = 0 => "EOF",

    Ident = 1 => "IDENT",
    PrivateIdent = 2 => "PRIVATE_IDENT",
    Number = 3 => "NUMBER",
    BigInt = 4 => "BIGINT",
    String = 5 => "STRING",
    RegExp = 6 => "REGEXP",
    TemplateNoSub = 7 => "TEMPLATE_NO_SUB",
    TemplateHead = 8 => "TEMPLATE_HEAD",
    TemplateMiddle = 9 => "TEMPLATE_MIDDLE",
    TemplateTail = 10 => "TEMPLATE_TAIL",

    LineComment = 11 => "LINE_COMMENT",
    BlockComment = 12 => "BLOCK_COMMENT",
    Hashbang = 13 => "HASHBANG",
    Whitespace = 14 => "WHITESPACE",
    LineTerminator = 15 => "LINE_TERMINATOR",

    StringCooked = 16 => "STRING_COOKED",
    IdentEscaped = 17 => "IDENT_ESCAPED",
    PrivateIdentEscaped = 18 => "PRIVATE_IDENT_ESCAPED",
    TemplateNoSubCooked = 19 => "TEMPLATE_NO_SUB_COOKED",
    TemplateHeadCooked = 20 => "TEMPLATE_HEAD_COOKED",
    TemplateMiddleCooked = 21 => "TEMPLATE_MIDDLE_COOKED",
    TemplateTailCooked = 22 => "TEMPLATE_TAIL_COOKED",
    Decimal = 23 => "DECIMAL",
    Float = 24 => "FLOAT",
    Binary = 25 => "BINARY",
    Octal = 26 => "OCTAL",
    Hex = 27 => "HEX",
    JsxText = 28 => "JSX_TEXT",
    JsxTagEnd = 29 => "JSX_TAG_END",
    JsxLt = 30 => "JSX_LT",

    LBrace = 32 => "{",
    RBrace = 33 => "}",
    LParen = 34 => "(",
    RParen = 35 => ")",
    LBracket = 36 => "[",
    RBracket = 37 => "]",
    Dot = 38 => ".",
    Ellipsis = 39 => "...",
    Semi = 40 => ";",
    Comma = 41 => ",",
    Colon = 42 => ":",
    Question = 43 => "?",
    OptionalChain = 44 => "?.",
    Nullish = 45 => "??",
    NullishEq = 46 => "??=",
    Arrow = 47 => "=>",
    Lt = 48 => "<",
    Le = 49 => "<=",
    Gt = 50 => ">",
    Ge = 51 => ">=",
    Eq = 52 => "=",
    EqEq = 53 => "==",
    EqEqEq = 54 => "===",
    Bang = 55 => "!",
    BangEq = 56 => "!=",
    BangEqEq = 57 => "!==",
    Plus = 58 => "+",
    PlusPlus = 59 => "++",
    PlusEq = 60 => "+=",
    Minus = 61 => "-",
    MinusMinus = 62 => "--",
    MinusEq = 63 => "-=",
    Star = 64 => "*",
    StarStar = 65 => "**",
    StarEq = 66 => "*=",
    StarStarEq = 67 => "**=",
    Slash = 68 => "/",
    SlashEq = 69 => "/=",
    Percent = 70 => "%",
    PercentEq = 71 => "%=",
    Amp = 72 => "&",
    AmpAmp = 73 => "&&",
    AmpEq = 74 => "&=",
    AmpAmpEq = 75 => "&&=",
    Pipe = 76 => "|",
    PipePipe = 77 => "||",
    PipeEq = 78 => "|=",
    PipePipeEq = 79 => "||=",
    Caret = 80 => "^",
    CaretEq = 81 => "^=",
    Tilde = 82 => "~",
    LShift = 83 => "<<",
    LShiftEq = 84 => "<<=",
    RShift = 85 => ">>",
    RShiftEq = 86 => ">>=",
    URShift = 87 => ">>>",
    URShiftEq = 88 => ">>>=",
    At = 89 => "@",

    KwBreak = 128 => "break",
    KwCase = 129 => "case",
    KwCatch = 130 => "catch",
    KwClass = 131 => "class",
    KwConst = 132 => "const",
    KwContinue = 133 => "continue",
    KwDebugger = 134 => "debugger",
    KwDefault = 135 => "default",
    KwDelete = 136 => "delete",
    KwDo = 137 => "do",
    KwElse = 138 => "else",
    KwEnum = 139 => "enum",
    KwExport = 140 => "export",
    KwExtends = 141 => "extends",
    KwFalse = 142 => "false",
    KwFinally = 143 => "finally",
    KwFor = 144 => "for",
    KwFunction = 145 => "function",
    KwIf = 146 => "if",
    KwImport = 147 => "import",
    KwIn = 148 => "in",
    KwInstanceof = 149 => "instanceof",
    KwNew = 150 => "new",
    KwNull = 151 => "null",
    KwReturn = 152 => "return",
    KwSuper = 153 => "super",
    KwSwitch = 154 => "switch",
    KwThis = 155 => "this",
    KwThrow = 156 => "throw",
    KwTrue = 157 => "true",
    KwTry = 158 => "try",
    KwTypeof = 159 => "typeof",
    KwVar = 160 => "var",
    KwVoid = 161 => "void",
    KwWhile = 162 => "while",
    KwWith = 163 => "with",
    KwYield = 164 => "yield",
    KwLet = 165 => "let",
    KwStatic = 166 => "static",
    KwAsync = 167 => "async",
    KwAwait = 168 => "await",
    KwOf = 169 => "of",
    KwFrom = 170 => "from",
    KwAs = 171 => "as",

    // TS-mode contextual keywords (`LexOptions::ts`) plus the strict-mode
    // reserved words; JS mode lexes all of these spellings as IDENT.
    // Contiguous after the JS block so `>= KW_BASE` range checks cover both.
    KwAbstract = 172 => "abstract",
    KwAccessor = 173 => "accessor",
    KwAny = 174 => "any",
    KwAsserts = 175 => "asserts",
    KwBigInt = 176 => "bigint",
    KwBoolean = 177 => "boolean",
    KwDeclare = 178 => "declare",
    KwGlobal = 179 => "global",
    KwImplements = 180 => "implements",
    KwInfer = 181 => "infer",
    KwInterface = 182 => "interface",
    KwIntrinsic = 183 => "intrinsic",
    KwIs = 184 => "is",
    KwKeyof = 185 => "keyof",
    KwModule = 186 => "module",
    KwNamespace = 187 => "namespace",
    KwNever = 188 => "never",
    KwNumber = 189 => "number",
    KwObject = 190 => "object",
    KwOut = 191 => "out",
    KwOverride = 192 => "override",
    KwPackage = 193 => "package",
    KwPrivate = 194 => "private",
    KwProtected = 195 => "protected",
    KwPublic = 196 => "public",
    KwReadonly = 197 => "readonly",
    KwRequire = 198 => "require",
    KwSatisfies = 199 => "satisfies",
    KwString = 200 => "string",
    KwSymbol = 201 => "symbol",
    KwType = 202 => "type",
    KwUndefined = 203 => "undefined",
    KwUnique = 204 => "unique",
    KwUnknown = 205 => "unknown",
    KwUsing = 206 => "using",

    Invalid = 255 => "INVALID",
}

/// First keyword kind: every kind `>= KW_BASE` other than [`TokenKind::Invalid`] is a keyword.
pub const KW_BASE: u8 = TokenKind::KwBreak as u8;
const KW_MAX: u8 = TokenKind::KwUsing as u8;

impl TokenKind {
    #[inline]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// # Safety
    ///
    /// `byte` must be a declared discriminant, i.e. `TokenKind::from_u8(byte).is_some()`.
    #[inline]
    #[must_use]
    pub const unsafe fn from_u8_unchecked(byte: u8) -> Self {
        debug_assert!(Self::from_u8(byte).is_some(), "not a declared TokenKind discriminant");
        // SAFETY: the caller guarantees `byte` is a declared discriminant, and
        // `TokenKind` is `#[repr(u8)]`, so it shares `u8`'s layout.
        unsafe { core::mem::transmute::<u8, Self>(byte) }
    }

    #[inline]
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        is_trivia_byte(self as u8)
    }

    #[inline]
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        (self as u8) >= KW_BASE && (self as u8) <= KW_MAX
    }

    #[inline]
    #[must_use]
    pub const fn is_string(self) -> bool {
        matches!(self, Self::String | Self::StringCooked)
    }

    #[inline]
    #[must_use]
    pub const fn is_template_no_sub(self) -> bool {
        matches!(self, Self::TemplateNoSub | Self::TemplateNoSubCooked)
    }

    #[inline]
    #[must_use]
    pub const fn is_template_head(self) -> bool {
        matches!(self, Self::TemplateHead | Self::TemplateHeadCooked)
    }

    #[inline]
    #[must_use]
    pub const fn is_template_middle(self) -> bool {
        matches!(self, Self::TemplateMiddle | Self::TemplateMiddleCooked)
    }

    #[inline]
    #[must_use]
    pub const fn is_template_tail(self) -> bool {
        matches!(self, Self::TemplateTail | Self::TemplateTailCooked)
    }

    #[inline]
    #[must_use]
    pub const fn is_ident(self) -> bool {
        matches!(self, Self::Ident | Self::IdentEscaped)
    }

    #[inline]
    #[must_use]
    pub const fn is_private_ident(self) -> bool {
        matches!(self, Self::PrivateIdent | Self::PrivateIdentEscaped)
    }

    #[inline]
    #[must_use]
    pub const fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::Number | Self::Decimal | Self::Float | Self::Binary | Self::Octal | Self::Hex
        )
    }
}

impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

/// Reinterpret raw kind bytes written by the pipeline as [`TokenKind`]s.
///
/// # Safety
///
/// Every byte in `bytes` must be a declared [`TokenKind`] discriminant. The
/// pipeline only ever writes kinds that came from [`crate::opmap`]'s tables or
/// from the named constants in `pipeline`, so this holds for any range the
/// lexer has written; it does *not* hold for uninitialised arena memory.
#[inline]
pub(crate) const unsafe fn kinds_from_bytes(bytes: &[u8]) -> &[TokenKind] {
    // SAFETY: `TokenKind` is `#[repr(u8)]` so it has the same size and
    // alignment as `u8`, and the caller guarantees every byte is a declared
    // discriminant.
    unsafe { core::slice::from_raw_parts(bytes.as_ptr().cast::<TokenKind>(), bytes.len()) }
}

#[inline]
pub(crate) fn debug_assert_kind_bytes(bytes: &[u8]) {
    debug_assert!(
        bytes.iter().all(|&b| TokenKind::from_u8(b).is_some()),
        "lexer wrote a byte that is not a declared TokenKind discriminant"
    );
}

pub const SPAN_SENTINELS: usize = 8;

pub const TRIVIA_MIN: u8 = TokenKind::LineComment as u8;
pub const TRIVIA_MAX: u8 = TokenKind::LineTerminator as u8;

/// [`TokenKind::is_trivia`] on a raw kind byte, for the pipeline's `u8` lanes.
#[inline]
#[must_use]
pub(crate) const fn is_trivia_byte(kind: u8) -> bool {
    kind.wrapping_sub(TRIVIA_MIN) <= TRIVIA_MAX - TRIVIA_MIN
}

pub mod token_flags {
    pub const LINE_BEFORE: u16 = 1 << 0;
    pub const COMMENT_BEFORE: u16 = 1 << 1;
    pub const HAS_ESCAPE: u16 = 1 << 2;
    pub const HAS_NON_ASCII: u16 = 1 << 3;
    pub const CONTAINS_NEWLINE: u16 = 1 << 4;
    pub const UNTERMINATED: u16 = 1 << 5;
    pub const INVALID: u16 = 1 << 6;
    pub const LEGACY_OCTAL: u16 = 1 << 7;
    pub const ESCAPED_KEYWORD: u16 = 1 << 8;
    pub const REGEXP_VALIDATED: u16 = 1 << 9;
    pub const ASI_RESTRICTED: u16 = 1 << 10;
}

const _: () = assert!(TokenKind::Hashbang as u8 > TokenKind::LineComment as u8);
const _: () = assert!((TokenKind::Hashbang as u8) < TokenKind::LineTerminator as u8);

/// Bit 31 of a `starts` entry: reserved "newline before this token" flag.
/// The lexer does not set it yet, but consumers must still read offsets
/// through [`offset`].
pub const NEWLINE_BEFORE_MASK: u32 = 0x8000_0000;

pub const OFFSET_MASK: u32 = 0x7FFF_FFFF;

/// Maximum lexable source length in bytes, imposed by the 31-bit offset field.
pub const MAX_SOURCE_LEN: u32 = OFFSET_MASK;

#[inline]
pub const fn offset(s: u32) -> u32 {
    s & OFFSET_MASK
}

#[inline]
pub const fn newline(s: u32) -> bool {
    (s & NEWLINE_BEFORE_MASK) != 0
}

#[inline]
pub const fn pack_start(offset: u32, newline_before: bool) -> u32 {
    debug_assert!(offset <= OFFSET_MASK);
    if newline_before { offset | NEWLINE_BEFORE_MASK } else { offset }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct StringSpan {
    pub start: u32,
    pub end_and_flags: u32,
}

#[expect(clippy::inline_always, reason = "cook-path hot: spans are packed per string/template")]
impl StringSpan {
    pub const LONE_SURROGATES_MASK: u32 = 0x8000_0000;
    pub const END_MASK: u32 = 0x7FFF_FFFF;
    /// On `start`, template spans only: the body contained a
    /// `NotEscapeSequence` (bad `\u`/`\x`, octal, `\8`/`\9`). The equivalent
    /// of oxc_parser's `cooked: None`, from which a parser raises the
    /// untagged-template error. Raw `start` reads must mask.
    pub const COOKED_INVALID_MASK: u32 = 0x8000_0000;
    pub const START_MASK: u32 = 0x7FFF_FFFF;

    #[inline(always)]
    #[must_use]
    pub const fn new(start: u32, end: u32, lone_surrogates: bool) -> Self {
        debug_assert!(end <= Self::END_MASK);
        let end_and_flags = if lone_surrogates { end | Self::LONE_SURROGATES_MASK } else { end };
        Self { start, end_and_flags }
    }

    #[inline(always)]
    #[must_use]
    pub const fn with_cooked_invalid(self) -> Self {
        Self { start: self.start | Self::COOKED_INVALID_MASK, end_and_flags: self.end_and_flags }
    }

    #[inline(always)]
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start & Self::START_MASK
    }

    #[inline(always)]
    #[must_use]
    pub const fn cooked_invalid(self) -> bool {
        (self.start & Self::COOKED_INVALID_MASK) != 0
    }

    #[inline(always)]
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end_and_flags & Self::END_MASK
    }

    #[inline(always)]
    #[must_use]
    pub const fn lone_surrogates(self) -> bool {
        (self.end_and_flags & Self::LONE_SURROGATES_MASK) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::{KW_BASE, TRIVIA_MAX, TRIVIA_MIN, TokenKind};
    use crate::{Lexer, PAD, default_options};

    #[test]
    fn from_u8_round_trips_every_variant() {
        for &kind in TokenKind::VARIANTS {
            assert_eq!(TokenKind::from_u8(kind as u8), Some(kind), "{}", kind.name());
        }
    }

    #[test]
    fn from_u8_agrees_with_the_discriminant_for_all_256_bytes() {
        let declared: Vec<u8> = TokenKind::VARIANTS.iter().map(|k| *k as u8).collect();
        for byte in 0..=u8::MAX {
            match TokenKind::from_u8(byte) {
                Some(kind) => {
                    assert_eq!(kind as u8, byte);
                    assert!(declared.contains(&byte));
                }
                None => assert!(!declared.contains(&byte), "byte {byte} is declared but unmapped"),
            }
        }
    }

    #[test]
    fn discriminants_are_unique() {
        let mut seen = [false; 256];
        for &kind in TokenKind::VARIANTS {
            let byte = kind as usize;
            assert!(!seen[byte], "duplicate discriminant {byte}");
            seen[byte] = true;
        }
    }

    /// The pipeline classifies by range, so the punctuator and keyword blocks
    /// must stay where `compress` and `opmap` expect them.
    #[test]
    fn kind_space_layout_holds() {
        for &kind in TokenKind::VARIANTS {
            let byte = kind as u8;
            if kind.is_keyword() {
                assert!(byte >= KW_BASE, "{} below KW_BASE", kind.name());
            }
            if kind.is_trivia() {
                assert!((TRIVIA_MIN..=TRIVIA_MAX).contains(&byte), "{}", kind.name());
            }
        }
        assert!(TokenKind::LBrace as u8 >= 32 && (TokenKind::At as u8) < KW_BASE);
        assert!(!TokenKind::Invalid.is_keyword());
        assert!(TokenKind::Hashbang.is_trivia());
    }

    /// Backs the safety invariant of [`super::kinds_from_bytes`]: the lexer
    /// never emits a byte outside the declared discriminants.
    #[test]
    fn every_emitted_kind_is_declared() {
        const SOURCES: [&str; 6] = [
            "#!/usr/bin/env node\nlet x = 0b1_0 + 0o7 + 0xFF + 1.5e3 + 9n;",
            "class A { #p = 1; static { this.#p ??= 2; } get x() { return `a${1}b${2}c`; } }",
            "a?.b?.[c] ?? d ||= e &&= f >>>= g; /re/gu.test('s\\u00e9'); // line\n/* block */",
            "export default async function* f(...a) { yield* await import('m'); }",
            "type T = { readonly [K in keyof U]?: U[K] extends infer V ? V : never };",
            "const el = <div a='1' {...r}>text {x} <br/></div>;",
        ];
        for src in SOURCES {
            let mut bytes = src.as_bytes().to_vec();
            let n = bytes.len();
            bytes.extend_from_slice(&[0u8; PAD]);
            for (jsx, ts) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut opts = default_options();
                opts.jsx = jsx;
                opts.ts = ts;
                let mut lexer = Lexer::new();
                lexer.lex(&bytes, n, opts);
                for kind in lexer.kinds() {
                    assert_eq!(
                        TokenKind::from_u8(*kind as u8),
                        Some(*kind),
                        "undeclared kind {} from {src:?}",
                        *kind as u8
                    );
                }
            }
        }
    }
}
