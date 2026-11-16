use crate::config::LexerConfig as Config;

use super::{Kind, Lexer, Token, keyword::match_keyword};

impl<C: Config> Lexer<'_, C> {
    /// Re-tokenize `<<`, `<=`, or `<<=` to `<`.
    ///
    /// Called when the parser encounters a compound token starting with `<` (e.g. `<<`)
    /// in a position where it could be the start of type arguments (e.g. `foo<<T>()>`).
    /// The lexer eagerly produces the compound token, but the parser needs a single `<`
    /// to speculatively try parsing type arguments.
    ///
    /// Only `<<` reaches here in code which parses. `<=` and `<<=` arrive only from the type-context
    /// callers in `ts/types.rs` and always go on to fail, for the reason documented on
    /// `parse_type_arguments_in_expression`, which rejects them before reaching here.
    ///
    /// The compound token was pushed to the collected token stream *before* `try_parse`
    /// created its checkpoint (since it was the "current" token at that point), so it is
    /// the last token in the stream. Overwrite it there with the single `<`. The remaining
    /// characters after the first `<` (e.g. the second `<` in `<<`) are lexed as separate
    /// tokens on subsequent `next_token` calls, so the stream ends up with all of them.
    ///
    /// If the speculative parse fails and `try_parse` rewinds, `truncate` leaves this `<`
    /// as the last collected token, and the caller (`expression.rs`) restores the original
    /// compound token over it via `rewrite_last_collected_token`.
    pub(crate) fn re_lex_as_typescript_l_angle(&mut self, offset: u32) -> Token {
        self.token_start = self.offset() - offset;
        self.source.back(offset as usize - 1);
        let token = self.finish_re_lex(Kind::LAngle);
        self.rewrite_last_collected_token(token);
        token
    }

    /// Whether the source after the last token can continue or end a type argument which began
    /// with that token, for a `<` which follows an expression.
    ///
    /// ```text
    /// TypeArguments    : `<` TypeArgumentList `>`
    /// TypeArgumentList : Type (`,` Type)*
    /// ```
    ///
    /// ECMAScript reads `a < b` as a `RelationalExpression`. TypeScript first tries the `<` as
    /// the start of type arguments (`f<T>(x)`), and falls back to the operator when that parse
    /// fails. Nearly every `<` in an expression is a comparison, and the text after its right
    /// operand is enough to tell: `i < n;`, `a < b.c)`, `x < 0 &&`.
    ///
    /// The last token lexed is the first one after `<`, and is a primary type on its own:
    /// a name (`is_name`), a literal or `this`. Such a type is complete unless the next token
    /// continues it, and a complete type argument is followed by `,` or `>`:
    ///
    /// ```text
    /// TypeReference     : TypeName [no LineTerminator here] TypeArguments?
    /// TypeName          : IdentifierReference | TypeName `.` IdentifierName
    /// ArrayType         : PrimaryType [no LineTerminator here] `[` `]`
    /// IndexedAccessType : PrimaryType [no LineTerminator here] `[` Type `]`
    /// IntersectionType  : Type `&` Type
    /// UnionType         : Type `|` Type
    /// ConditionalType   : Type `extends` Type `?` Type `:` Type
    /// TypePredicate     : `this` `is` Type
    /// ```
    ///
    /// plus the JSDoc postfix `!` and `?`, which the type parser accepts in order to report them.
    /// Any other token ends the list early.
    ///
    /// Returns `false` only when the type argument list is certain not to parse, so that the
    /// parser need not attempt it. Anything this does not recognize, including a comment, an
    /// escape, or a non-ASCII character, returns `true`.
    ///
    /// This reads source bytes and leaves the lexer untouched, so it lexes no token which the
    /// parser then has to lex again.
    ///
    /// The checks mirror `parse_non_array_type`, `parse_postfix_type_rest` and their callers in
    /// `ts/types.rs`, and must be kept in step with them.
    pub(crate) fn can_continue_type_argument(&self, is_name: bool) -> bool {
        let mut bytes = Lookahead::new(self.source.remaining().as_bytes());
        can_continue_type_argument(&mut bytes, is_name).unwrap_or(true)
    }

    /// Whether the source after the current `<<` can begin a type parameter list, for a `<<`
    /// which follows an expression.
    ///
    /// ECMAScript reads `a << b` as a `ShiftExpression`. In TypeScript the first `<` may instead
    /// open type arguments, the first of which then starts with the second `<`. That makes it
    /// a generic function type, whose type parameter list opens with a `TypeParameter`:
    ///
    /// ```text
    /// f<<T>(x: T) => T>(g)
    /// TypeParameter : Modifiers? BindingIdentifier (`extends` Type)? (`=` Type)?
    /// ```
    ///
    /// which rules out `1 << 2` at the first token, and `a << b;` at the second.
    ///
    /// Returns `false` only when the type arguments are certain not to parse. Anything this
    /// does not recognize returns `true`.
    ///
    /// The checks mirror `parse_ts_type_parameter` and its callers in `ts/types.rs`, and must be
    /// kept in step with them.
    pub(crate) fn can_start_type_parameters(&self) -> bool {
        let mut bytes = Lookahead::new(self.source.remaining().as_bytes());
        can_start_type_parameters(&mut bytes).unwrap_or(true)
    }

    /// Whether the source after the current `<` can begin the type parameters of a generic arrow
    /// function in a JSX file, for a `<` which starts an expression.
    ///
    /// TypeScript reads `<T>` there as a JSX element, and as type parameters only when the first
    /// one cannot be mistaken for a tag:
    ///
    /// ```text
    /// <T,>(x: T) => x
    /// <T = U>(x: T) => x
    /// <T extends U>(x: T) => x
    /// <const T>(x: T) => x
    /// ```
    ///
    /// Returns `false` only when the `<` is certain not to open type parameters, which is the case
    /// for nearly every element: `<div>`, `<Foo bar />`, `<>`. Otherwise returns `true`, and the
    /// parser decides from the tokens.
    pub(crate) fn can_start_type_parameters_in_jsx(&self) -> bool {
        let mut bytes = Lookahead::new(self.source.remaining().as_bytes());
        can_start_type_parameters_in_jsx(&mut bytes).unwrap_or(true)
    }
}

/// See [`Lexer::can_continue_type_argument`]. `None` means the source could not be classified.
fn can_continue_type_argument(bytes: &mut Lookahead, is_name: bool) -> Option<bool> {
    let mut next = bytes.peek()?;
    while is_name && next == b'.' {
        bytes.bump();
        if !is_identifier_start(bytes.peek()?) {
            return Some(false);
        }
        bytes.word()?;
        next = bytes.peek()?;
    }

    Some(match next {
        b'>' | b',' | b'<' | b'[' => true,
        // Not `!=` or `!==`.
        b'!' => bytes.byte_after() != b'=',
        // Not `&&`, `||`, `&=` or `|=`.
        b'&' | b'|' => {
            let after = bytes.byte_after();
            after != next && after != b'='
        }
        // JSDoc `T?` ends the primary type, so what follows it cannot start another type.
        // This tells it apart from a conditional expression: `a < b ? c : d`.
        b'?' => {
            bytes.bump();
            match bytes.peek()? {
                b'>' | b',' => true,
                byte if is_identifier_start(byte) => bytes.word()? == b"extends",
                _ => false,
            }
        }
        byte if is_identifier_start(byte) => matches!(bytes.word()?, b"extends" | b"is"),
        _ => false,
    })
}

/// See [`Lexer::can_start_type_parameters`]. `None` means the source could not be classified.
fn can_start_type_parameters(bytes: &mut Lookahead) -> Option<bool> {
    // The parser splits `<<` and lexes on from the second `<`, which joins a `<` right after
    // it into another `<<`. That is not the `<` of a type parameter list: `a <<< b`, `a <<<!--`.
    if bytes.first() == b'<' {
        return Some(false);
    }
    let first = bytes.peek()?;
    // An empty list is only a recoverable error: `f<<>() => void>()`.
    if first == b'>' {
        return Some(true);
    }
    if !is_identifier_start(first) {
        return Some(false);
    }
    // Keywords may be modifiers (`const T`) or contextual names (`of`).
    if match_keyword(bytes.word()?) != Kind::Ident {
        return Some(true);
    }
    Some(match bytes.peek()? {
        b'>' | b',' => true,
        // Not `==` or `=>`.
        b'=' => !matches!(bytes.byte_after(), b'=' | b'>'),
        byte if is_identifier_start(byte) => bytes.word()? == b"extends",
        _ => false,
    })
}

/// See [`Lexer::can_start_type_parameters_in_jsx`]. `None` means the source could not be
/// classified.
///
/// The checks mirror `is_parenthesized_arrow_function_expression_worker` in `js/arrow.rs`,
/// and must be kept in step with it.
fn can_start_type_parameters_in_jsx(bytes: &mut Lookahead) -> Option<bool> {
    if !is_identifier_start(bytes.peek()?) {
        return Some(false);
    }
    if bytes.word()? == b"const" {
        return Some(true);
    }
    Some(match bytes.peek()? {
        b',' | b'=' => true,
        byte if is_identifier_start(byte) => bytes.word()? == b"extends",
        _ => false,
    })
}

/// Classes of byte which [`Lookahead`] tells apart.
mod class {
    /// ASCII whitespace or line break.
    pub const WHITESPACE: u8 = 1 << 0;
    /// `A-Z`, `a-z`, `_` or `$`.
    pub const IDENTIFIER_START: u8 = 1 << 1;
    /// `A-Z`, `a-z`, `0-9`, `_` or `$`.
    pub const IDENTIFIER_PART: u8 = 1 << 2;
    /// `/`, `<` or `-`, which may start a comment: `//`, `/*`, `<!--`, `-->`.
    pub const COMMENT_START: u8 = 1 << 3;
    /// A backslash or a non-ASCII byte. Either may belong to the token before it or the token
    /// after it, in a way [`Lookahead`] does not follow: an escaped or non-ASCII identifier,
    /// or a Unicode space or line break.
    ///
    /// [`Lookahead`]: super::Lookahead
    pub const UNCLASSIFIED: u8 = 1 << 4;
}

static CLASSES: [u8; 256] = {
    let mut classes = [0; 256];
    let mut index = 0;
    while index < 256 {
        #[expect(clippy::cast_possible_truncation)]
        let byte = index as u8;
        classes[index] = match byte {
            b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C => class::WHITESPACE,
            b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'$' => {
                class::IDENTIFIER_START | class::IDENTIFIER_PART
            }
            b'0'..=b'9' => class::IDENTIFIER_PART,
            b'/' | b'<' | b'-' => class::COMMENT_START,
            b'\\' | 0x80.. => class::UNCLASSIFIED,
            _ => 0,
        };
        index += 1;
    }
    classes
};

#[inline]
fn is(byte: u8, class: u8) -> bool {
    CLASSES[byte as usize] & class != 0
}

#[inline]
fn is_identifier_start(byte: u8) -> bool {
    is(byte, class::IDENTIFIER_START)
}

/// Read-only view of the source bytes which follow the current token.
///
/// It recognizes only what the checks above need: ASCII whitespace, ASCII identifiers and
/// keywords, and punctuation. Methods return `None` for anything else, since the lexer may then
/// see tokens other than the bytes suggest.
struct Lookahead<'s> {
    bytes: &'s [u8],
}

impl<'s> Lookahead<'s> {
    fn new(bytes: &'s [u8]) -> Self {
        Self { bytes }
    }

    /// Skip whitespace and line breaks, and return the first byte of the next token.
    /// Returns `0` at the end of the source, which starts no token.
    ///
    /// Returns `None` where a comment may start, or at an escape or a non-ASCII character.
    #[inline]
    fn peek(&mut self) -> Option<u8> {
        let mut byte = self.first();
        while is(byte, class::WHITESPACE) {
            self.bump();
            byte = self.first();
        }
        if is(byte, class::COMMENT_START | class::UNCLASSIFIED) {
            return self.peek_unusual(byte);
        }
        Some(byte)
    }

    /// Out of line, to keep [`Self::peek`] small where it is inlined.
    #[cold]
    fn peek_unusual(&self, byte: u8) -> Option<u8> {
        let is_token_start = match byte {
            b'/' => !matches!(self.byte_after(), b'/' | b'*'),
            b'<' => !self.bytes.starts_with(b"<!--"),
            b'-' => !self.bytes.starts_with(b"-->"),
            _ => false,
        };
        if is_token_start { Some(byte) } else { None }
    }

    /// The next byte, or `0` at the end of the source.
    #[inline]
    fn first(&self) -> u8 {
        self.bytes.first().copied().unwrap_or(0)
    }

    /// The byte after the one [`Self::peek`] returned, or `0` at the end of the source.
    #[inline]
    fn byte_after(&self) -> u8 {
        self.bytes.get(1).copied().unwrap_or(0)
    }

    /// Skip the byte [`Self::peek`] returned.
    #[inline]
    fn bump(&mut self) {
        self.bytes = &self.bytes[1..];
    }

    /// Skip and return an ASCII identifier or keyword.
    ///
    /// Returns `None` if an escape or a non-ASCII character may continue it.
    #[inline]
    fn word(&mut self) -> Option<&'s [u8]> {
        let len = self
            .bytes
            .iter()
            .position(|&byte| !is(byte, class::IDENTIFIER_PART))
            .unwrap_or(self.bytes.len());
        let (word, rest) = self.bytes.split_at(len);
        self.bytes = rest;
        if is(self.first(), class::UNCLASSIFIED) { None } else { Some(word) }
    }
}
