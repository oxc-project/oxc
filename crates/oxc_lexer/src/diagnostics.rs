use std::borrow::Cow;

use oxc_ast::ast::REGEXP_FLAGS_LIST;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_syntax::identifier::{is_identifier_part_ascii, is_identifier_start};

use crate::error::{DiagCode, DiagSeverity, Diagnostic};

/// The diagnostic's span: the lexer stores `(off, len)`, oxc `(start, end)`.
#[inline]
fn span_of(d: &Diagnostic) -> Span {
    Span::new(d.off, d.off + d.len)
}

/// The source slice the diagnostic covers; empty if out of bounds.
#[inline]
fn lexeme<'a>(source: &'a str, d: &Diagnostic) -> &'a str {
    let (start, end) = (d.off as usize, (d.off + d.len) as usize);
    source.get(start..end).unwrap_or("")
}

/// First char of the span (the offending character), or U+FFFD.
#[inline]
fn offending_char(source: &str, d: &Diagnostic) -> char {
    lexeme(source, d).chars().next().unwrap_or('\u{FFFD}')
}

/// Decode the identifier escape whose text ends at `end`: the offending
/// char of an escaped-non-identifier-char diagnostic exists only decoded,
/// so the lexer emits an empty span after the escape and the bridge
/// recovers the char here.
fn decode_escape_ending_at(source: &str, end: u32) -> Option<char> {
    let text = source.get(..end as usize)?;
    let bs = text.rfind('\\')?;
    let esc = &text[bs..];
    let hex = if let Some(h) = esc.strip_prefix("\\u{") {
        h.strip_suffix('}')?
    } else {
        esc.strip_prefix("\\u")?
    };
    char::from_u32(u32::from_str_radix(hex, 16).ok()?)
}

/// Honor the POD severity (every current code is an error).
#[inline]
fn diag(severity: DiagSeverity, message: impl Into<Cow<'static, str>>) -> OxcDiagnostic {
    if severity == DiagSeverity::Warning {
        OxcDiagnostic::warn(message)
    } else {
        OxcDiagnostic::error(message)
    }
}

/// Is `off` the start of a numeric literal rather than an adjacency anchor
/// inside one? Adjacency diags (`1.5n`, `0b12`) anchor right after a number
/// char, so a number/identifier byte before `off` declines; so does a
/// non-ASCII one (conservative - the fallback rendering applies).
fn at_numeric_literal_start(source: &str, off: u32) -> bool {
    let b = source.as_bytes();
    let i = off as usize;
    let starts = match b.get(i) {
        Some(c) if c.is_ascii_digit() => true,
        Some(b'.') => b.get(i + 1).is_some_and(u8::is_ascii_digit),
        _ => false,
    };
    starts
        && match i.checked_sub(1).and_then(|p| b.get(p)) {
            None => true,
            Some(&p) => {
                p.is_ascii() && !p.is_ascii_alphanumeric() && !matches!(p, b'_' | b'$' | b'.')
            }
        }
}

/// The parser's first numeric error: a required digit was missing (empty
/// span at the char), or identifier chars right after a number.
enum NumErr {
    Unexpected(usize),
    NumberEnd(usize, usize),
}

/// Port of oxc_parser's numeric scanner (`lexer/numeric.rs`), stopping at
/// its first diagnostic. Mirrors the quirks exactly: no separators in
/// legacy-octal-like decimals, a legacy-octal exponent only as lowercase `e`
/// after an `8`/`9` flipped the run to NonOctalDecimal, and `.` after a
/// pure-octal run simply ends the token. Each method notes the parser
/// function it ports.
struct NumericWalk<'a> {
    src: &'a str,
    i: usize,
}

impl NumericWalk<'_> {
    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.i).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }

    fn bump(&mut self) {
        self.i += 1; // ASCII call sites only
    }

    fn walk(&mut self) -> Result<(), NumErr> {
        match self.peek() {
            Some(b'0') => {
                self.bump();
                self.zero()
            }
            Some(c) if c.is_ascii_digit() => {
                self.bump();
                self.decimal_after_first_digit()
            }
            // `.5`-style literal: fraction digits are REQUIRED.
            Some(b'.') => {
                self.bump();
                self.decimal_digits()?;
                self.optional_exponent()?;
                self.check_after()
            }
            _ => Ok(()),
        }
    }

    /// `read_zero`
    fn zero(&mut self) -> Result<(), NumErr> {
        match self.peek() {
            Some(b'b' | b'B') => self.non_decimal(|c| matches!(c, b'0' | b'1')),
            Some(b'o' | b'O') => self.non_decimal(|c| matches!(c, b'0'..=b'7')),
            Some(b'x' | b'X') => self.non_decimal(u8::is_ascii_hexdigit),
            // `0e...`: the parser returns straight out of the exponent read -
            // no trailing-char check on this path.
            Some(b'e' | b'E') => {
                self.bump();
                self.exponent()
            }
            Some(b'.') => {
                self.bump();
                self.optional_fraction_then_exponent()
            }
            Some(b'n') => {
                self.bump();
                self.check_after()
            }
            Some(c) if c.is_ascii_digit() => self.legacy_octal(),
            _ => self.check_after(),
        }
    }

    /// `read_non_decimal`
    fn non_decimal(&mut self, is_digit: impl Fn(&u8) -> bool) -> Result<(), NumErr> {
        self.bump(); // the radix marker
        if self.peek().as_ref().is_some_and(&is_digit) {
            self.bump();
        } else {
            return Err(NumErr::Unexpected(self.i));
        }
        while let Some(c) = self.peek() {
            if c == b'_' {
                self.bump();
                if self.peek().as_ref().is_some_and(&is_digit) {
                    self.bump();
                } else {
                    return Err(NumErr::Unexpected(self.i));
                }
            } else if is_digit(&c) {
                self.bump();
            } else {
                break;
            }
        }
        if self.peek() == Some(b'n') {
            self.bump();
        }
        self.check_after()
    }

    /// `read_legacy_octal`
    fn legacy_octal(&mut self) -> Result<(), NumErr> {
        let mut decimal = false;
        while let Some(c) = self.peek() {
            match c {
                b'0'..=b'7' => self.bump(),
                b'8' | b'9' => {
                    decimal = true;
                    self.bump();
                }
                _ => break,
            }
        }
        match self.peek() {
            Some(b'.') if decimal => {
                self.bump();
                self.optional_fraction_then_exponent()
            }
            // Lowercase only, faithfully: the parser rejects `08E1`.
            Some(b'e') if decimal => {
                self.bump();
                self.exponent()
            }
            _ => self.check_after(),
        }
    }

    /// `decimal_literal_after_first_digit`
    fn decimal_after_first_digit(&mut self) -> Result<(), NumErr> {
        self.decimal_digits_after_first()?;
        if self.peek() == Some(b'.') {
            self.bump();
            return self.optional_fraction_then_exponent();
        }
        if self.peek() == Some(b'n') {
            self.bump();
            return self.check_after();
        }
        self.optional_exponent()?;
        self.check_after()
    }

    /// `decimal_literal_after_decimal_point_after_digits`
    fn optional_fraction_then_exponent(&mut self) -> Result<(), NumErr> {
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.bump();
            self.decimal_digits_after_first()?;
        }
        self.optional_exponent()?;
        self.check_after()
    }

    /// `read_decimal_digits` (first digit required)
    fn decimal_digits(&mut self) -> Result<(), NumErr> {
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.bump();
        } else {
            return Err(NumErr::Unexpected(self.i));
        }
        self.decimal_digits_after_first()
    }

    /// `read_decimal_digits_after_first_digit`
    fn decimal_digits_after_first(&mut self) -> Result<(), NumErr> {
        while let Some(c) = self.peek() {
            if c == b'_' {
                self.bump();
                if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    self.bump();
                } else {
                    return Err(NumErr::Unexpected(self.i));
                }
            } else if c.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }
        Ok(())
    }

    /// `optional_exponent`
    fn optional_exponent(&mut self) -> Result<(), NumErr> {
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.bump();
            return self.exponent();
        }
        Ok(())
    }

    /// `read_decimal_exponent`
    fn exponent(&mut self) -> Result<(), NumErr> {
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.bump();
        }
        self.decimal_digits()
    }

    /// `check_after_numeric_literal`: an IdentifierStart or digit directly
    /// after the literal, spanning it plus the identifier-start run after.
    fn check_after(&mut self) -> Result<(), NumErr> {
        let bad = match self.peek_char() {
            None => false,
            Some(c) if c.is_ascii() => is_identifier_part_ascii(c),
            Some(c) => is_identifier_start(c),
        };
        if !bad {
            return Ok(());
        }
        let start = self.i;
        if let Some(c) = self.peek_char() {
            self.i += c.len_utf8();
        }
        while let Some(c) = self.peek_char() {
            if is_identifier_start(c) {
                self.i += c.len_utf8();
            } else {
                break;
            }
        }
        Err(NumErr::NumberEnd(start, self.i))
    }
}

/// Re-walk the numeric literal at `off` and reproduce oxc_parser's first
/// diagnostic for it; `None` when the walk finds nothing wrong.
fn parser_numeric_first_error(source: &str, off: u32, sev: DiagSeverity) -> Option<OxcDiagnostic> {
    let mut walk = NumericWalk { src: source, i: off as usize };

    #[expect(
        clippy::cast_possible_truncation,
        reason = "source offsets are bounded by MAX_SOURCE_LEN"
    )]
    match walk.walk() {
        Ok(()) => None,
        Err(NumErr::Unexpected(at)) => {
            let span = Span::new(at as u32, at as u32);
            Some(match source[at..].chars().next() {
                Some(c) => diag(sev, format!("Invalid Character `{c}`")).with_label(span),
                None => diag(sev, "Unexpected end of file").with_label(span),
            })
        }
        Err(NumErr::NumberEnd(start, end)) => Some(
            diag(sev, "Invalid characters after number")
                .with_label(Span::new(start as u32, end as u32)),
        ),
    }
}

/// Convert one lexer [`Diagnostic`] into an [`OxcDiagnostic`] with
/// oxc_parser's message and span for that error class. `source` is the
/// original text, needed to recover offending characters.
pub fn to_oxc_diagnostic(d: &Diagnostic, source: &str) -> OxcDiagnostic {
    let span = span_of(d);
    let sev = d.severity;

    // Numeric diags anchored at the literal re-walk it for the parser's
    // exact first error; adjacency-anchored spans and clean walks fall
    // through to the static arms.
    if matches!(
        d.code,
        DiagCode::InvalidNumericSeparator
            | DiagCode::InvalidBigint
            | DiagCode::InvalidNumericLiteral
    ) && at_numeric_literal_start(source, d.off)
        && let Some(exact) = parser_numeric_first_error(source, d.off, sev)
    {
        return exact;
    }

    #[expect(
        clippy::match_same_arms,
        reason = "distinct codes deliberately collapse to the parser's message (see module docs)"
    )]
    match d.code {
        // Exact 1:1 with oxc_parser/src/diagnostics.rs.
        DiagCode::UnterminatedString => diag(sev, "Unterminated string").with_label(span),
        DiagCode::UnterminatedBlockComment => {
            diag(sev, "Unterminated multiline comment").with_label(span)
        }
        DiagCode::UnterminatedRegexp => diag(sev, "Unterminated regular expression").with_label(span),
        // The parser's string-escape message; "Invalid Unicode escape
        // sequence" belongs to identifier escapes below.
        DiagCode::InvalidUnicodeEscape => diag(sev, "Invalid escape sequence").with_label(span),
        DiagCode::UnexpectedCharacter => {
            // The len-0 shape is an escaped non-identifier char: the message
            // embeds the decoded char, recovered from the escape text ending
            // at `off`.
            let ch = if d.len == 0 {
                decode_escape_ending_at(source, d.off).unwrap_or('\u{FFFD}')
            } else {
                offending_char(source, d)
            };
            diag(sev, format!("Invalid Character `{ch}`")).with_label(span)
        }
        DiagCode::InvalidRegexpFlag => diag(
            sev,
            format!("Unexpected flag {} in regular expression literal", offending_char(source, d)),
        )
        .with_label(span)
        .with_help(format!("The allowed flags are `{REGEXP_FLAGS_LIST}`")),
        DiagCode::DuplicateRegexpFlag => diag(
            sev,
            format!(
                "Flag {} is mentioned twice in regular expression literal",
                offending_char(source, d)
            ),
        )
        .with_label(span)
        .with_help("Remove the duplicated flag here"),
        // A misplaced bigint suffix (`1.5n`) also reaches oxc_parser's
        // `invalid_number_end` (its float path never consumes the `n`), so
        // BIGINT collapses to the same message and span.
        DiagCode::InvalidNumericLiteral | DiagCode::InvalidBigint => {
            diag(sev, "Invalid characters after number").with_label(span)
        }

        // The parser does not distinguish unterminated template from string,
        // and reports newline-in-literal as "unterminated".
        DiagCode::UnterminatedTemplate => diag(sev, "Unterminated string").with_label(span),
        DiagCode::LineTerminatorInString => diag(sev, "Unterminated string").with_label(span),
        DiagCode::LineTerminatorInRegexp => {
            diag(sev, "Unterminated regular expression").with_label(span)
        }
        // The parser-side `invalid_number` message.
        DiagCode::InvalidNumericSeparator => {
            diag(sev, format!("Invalid Number {}", lexeme(source, d))).with_label(span)
        }
        // The parser's identifier-escape message, distinct from the
        // string-escape one above.
        DiagCode::InvalidIdentifierEscape => {
            diag(sev, "Invalid Unicode escape sequence").with_label(span)
        }

        // The parser consumes valid &str, so its closest analog to bad UTF-8
        // is the binary-file error, which carries no label.
        DiagCode::InvalidUtf8 => diag(sev, "File appears to be binary.").with_error_code("TS", "1490"),
        DiagCode::InvalidHashbangPosition => {
            diag(sev, format!("Invalid Character `{}`", offending_char(source, d))).with_label(span)
        }
        DiagCode::InvalidRegexpGrammar => diag(sev, "Invalid regular expression").with_label(span),
        DiagCode::HtmlCommentInModule => {
            diag(sev, "HTML comments are not allowed in modules").with_label(span)
        }
        DiagCode::UnterminatedJsxElement => {
            diag(sev, "JSX element has no corresponding closing tag")
                .with_label(span)
                .with_help("In a `.tsx` file `<T>(...) =>` opens a JSX element; write `<T,>(...) =>` for a generic arrow function")
        }
        DiagCode::JsxTextInvalidCharacter => {
            let (ch, entity) = if lexeme(source, d) == "}" { ('}', "rbrace") } else { ('>', "gt") };
            diag(sev, format!("Unexpected token. Did you mean `{{'{ch}'}}` or `&{entity};`?"))
                .with_label(span)
        }
        DiagCode::UnterminatedJsxTag => diag(sev, "Unterminated JSX tag").with_label(span),
        DiagCode::UnterminatedJsxContainer => {
            diag(sev, "Unterminated JSX expression container").with_label(span)
        }
        DiagCode::JsxClosingTagMismatch => diag(
            sev,
            format!("Expected corresponding JSX closing tag, found `{}`", lexeme(source, d)),
        )
        .with_label(span),
        DiagCode::OracleDepthExceeded => diag(sev, "Nesting depth limit exceeded").with_label(span),
        DiagCode::AllocationLimitExceeded => diag(sev, "Source length exceeds 4 GiB limit"),

        // `Ok` never appears in the buffer; unknown codes get a fallback.
        DiagCode::Ok => diag(sev, "Lexer error").with_label(span),
    }
}

/// Convert a slice of lexer diagnostics into the `Vec<OxcDiagnostic>` shape
/// oxc_parser collects.
#[must_use]
pub fn to_oxc_diagnostics(diags: &[Diagnostic], source: &str) -> Vec<OxcDiagnostic> {
    diags.iter().map(|d| to_oxc_diagnostic(d, source)).collect()
}

#[cfg(test)]
mod tests {
    use crate::error::DiagCode;

    use super::*;

    fn d(code: DiagCode, off: u32, len: u32) -> Diagnostic {
        Diagnostic { off, len, code, severity: DiagSeverity::Error }
    }

    #[test]
    fn reproduces_parser_messages_and_spans() {
        let src = "let x = 'abc";
        let g = to_oxc_diagnostic(&d(DiagCode::UnterminatedString, 8, 4), src);
        assert_eq!(g.message.as_ref(), "Unterminated string");

        // message embeds the offending char from `source`
        let src = "a @ b";
        let g = to_oxc_diagnostic(&d(DiagCode::UnexpectedCharacter, 2, 1), src);
        assert_eq!(g.message.as_ref(), "Invalid Character `@`");

        let src = "/a/z";
        let g = to_oxc_diagnostic(&d(DiagCode::InvalidRegexpFlag, 3, 1), src);
        assert_eq!(g.message.as_ref(), "Unexpected flag z in regular expression literal");
        assert_eq!(g.help.as_deref(), Some("The allowed flags are `gimsuydv`"));
    }

    #[test]
    fn finer_codes_collapse_to_parser_text() {
        let src = "`abc";
        let g = to_oxc_diagnostic(&d(DiagCode::UnterminatedTemplate, 0, 4), src);
        assert_eq!(g.message.as_ref(), "Unterminated string");

        // separator errors re-walk the literal: `1_000_` at EOF is the
        // parser's unexpected-end, not "Invalid Number ..."
        let src = "1_000_";
        let g = to_oxc_diagnostic(&d(DiagCode::InvalidNumericSeparator, 0, 6), src);
        assert_eq!(g.message.as_ref(), "Unexpected end of file");
    }

    fn label(g: &OxcDiagnostic) -> (u32, u32) {
        let l = g.labels.as_ref().first().expect("expected a labeled span");
        (l.offset(), l.len())
    }

    #[test]
    fn numeric_diags_reproduce_parser_walk() {
        // (source, POD code, POD off, POD len, parser message, parser label)
        type Case = (&'static str, DiagCode, u32, u32, &'static str, (u32, u32));
        let cases: &[Case] = &[
            ("x = 0b2;", DiagCode::InvalidNumericLiteral, 4, 2, "Invalid Character `2`", (6, 0)),
            ("0b0_n;", DiagCode::InvalidNumericSeparator, 0, 5, "Invalid Character `n`", (4, 0)),
            ("1__03;", DiagCode::InvalidNumericSeparator, 0, 5, "Invalid Character `_`", (2, 0)),
            ("1_000_", DiagCode::InvalidNumericSeparator, 0, 6, "Unexpected end of file", (6, 0)),
            (
                "0_0;",
                DiagCode::InvalidNumericSeparator,
                0,
                3,
                "Invalid characters after number",
                (1, 1),
            ),
            ("00n;", DiagCode::InvalidBigint, 0, 3, "Invalid characters after number", (2, 1)),
            (
                "10._e1;",
                DiagCode::InvalidNumericSeparator,
                0,
                6,
                "Invalid characters after number",
                (3, 2),
            ),
            (
                "00e1;",
                DiagCode::InvalidNumericLiteral,
                0,
                4,
                "Invalid characters after number",
                (2, 1),
            ),
        ];
        for &(src, code, off, len, msg, lab) in cases {
            let g = to_oxc_diagnostic(&d(code, off, len), src);
            assert_eq!(g.message.as_ref(), msg, "message for {src:?}");
            assert_eq!(label(&g), lab, "label for {src:?}");
        }

        // adjacency-anchored spans bypass the walk and keep their POD span
        let g = to_oxc_diagnostic(&d(DiagCode::InvalidNumericLiteral, 1, 2), "3in");
        assert_eq!(g.message.as_ref(), "Invalid characters after number");
        assert_eq!(label(&g), (1, 2));
    }
}
