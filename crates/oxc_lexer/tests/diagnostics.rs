//! Diagnostic coverage for every code class. Each positive test has a
//! `*_never_flagged` counterpart: valid input must emit nothing.
#![cfg(target_endian = "little")]
#![expect(
    clippy::cast_possible_truncation,
    clippy::unreadable_literal,
    reason = "test helpers: lengths fit u32; the fuzz PRNG uses raw constants"
)]

use oxc_lexer::{Diagnostic, PAD, default_options, diag_code, lex_utf8};

fn diags(code: &str) -> Vec<Diagnostic> {
    let mut buf = code.as_bytes().to_vec();
    let len = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0); // lexer over-reads up to PAD bytes past `len`
    let (res, _arena) = lex_utf8(&buf, len, default_options());
    res.diagnostics().to_vec()
}

fn codes(code: &str) -> Vec<u16> {
    diags(code).iter().map(|d| d.code).collect()
}

fn codes_jsx(code: &str) -> Vec<u16> {
    let mut buf = code.as_bytes().to_vec();
    let len = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let mut opts = default_options();
    opts.jsx = true;
    let (res, _arena) = lex_utf8(&buf, len, opts);
    res.diagnostics().iter().map(|d| d.code).collect()
}

fn codes_tsx(code: &str) -> Vec<u16> {
    let mut buf = code.as_bytes().to_vec();
    let len = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let mut opts = default_options();
    opts.jsx = true;
    opts.ts = true;
    let (res, _arena) = lex_utf8(&buf, len, opts);
    res.diagnostics().iter().map(|d| d.code).collect()
}

#[test]
fn unterminated_string() {
    // opener at 4 to EOF: span (4, 4)
    let d = diags("x = 'abc");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, diag_code::UNTERMINATED_STRING);
    assert_eq!((d[0].off, d[0].len), (4, 4));
}

#[test]
fn unterminated_template() {
    assert_eq!(codes("x = `abc"), vec![diag_code::UNTERMINATED_TEMPLATE]);
    assert_eq!(codes("x = `a${b}c"), vec![diag_code::UNTERMINATED_TEMPLATE]);
    // An open substitution at EOF is a parser-level error; oxc_parser's
    // lexer is also silent here.
    assert!(codes("x = `a${b").is_empty());
}

#[test]
fn unterminated_block_comment() {
    assert_eq!(codes("x /* abc"), vec![diag_code::UNTERMINATED_BLOCK_COMMENT]);
}

#[test]
fn unterminated_regex() {
    assert_eq!(codes("x = /abc"), vec![diag_code::UNTERMINATED_REGEXP]);
}

#[test]
fn unterminated_in_jsx_path() {
    // carve_jsx must detect the same unterminated literals as carve
    assert_eq!(codes_jsx("const x = 'abc"), vec![diag_code::UNTERMINATED_STRING]);
    assert_eq!(codes_jsx("const x = `abc"), vec![diag_code::UNTERMINATED_TEMPLATE]);
    assert_eq!(codes_jsx("const x = 1 /* abc"), vec![diag_code::UNTERMINATED_BLOCK_COMMENT]);
    assert_eq!(codes_jsx("const x = /abc"), vec![diag_code::UNTERMINATED_REGEXP]);
    assert!(codes_jsx("const el = <div className='x'>hi</div>;").is_empty());
}

#[test]
fn numeric_separator_and_empty_radix() {
    use diag_code as D;
    assert_eq!(codes("x = 1_000_"), vec![D::INVALID_NUMERIC_SEPARATOR]); // trailing
    assert_eq!(codes("x = 1__2"), vec![D::INVALID_NUMERIC_SEPARATOR]); // double
    assert_eq!(codes("x = 0x_1"), vec![D::INVALID_NUMERIC_SEPARATOR]); // after prefix
    assert_eq!(codes("x = 0xAB_"), vec![D::INVALID_NUMERIC_SEPARATOR]); // trailing in hex
    assert_eq!(codes("x = 0x;"), vec![D::INVALID_NUMERIC_LITERAL]); // empty radix
}

#[test]
fn legacy_octal_like_decimal() {
    use diag_code as D;
    // Leading `0` + digit: no separators, no bigint suffix, and an exponent
    // only as lowercase `e` after an 8/9 (oxc_parser's read_legacy_octal
    // quirk).
    assert_eq!(codes("x = 0_0;"), vec![D::INVALID_NUMERIC_SEPARATOR]);
    assert_eq!(codes("x = 00_0;"), vec![D::INVALID_NUMERIC_SEPARATOR]);
    assert_eq!(codes("x = 08_0;"), vec![D::INVALID_NUMERIC_SEPARATOR]);
    assert_eq!(codes("x = 00n;"), vec![D::INVALID_BIGINT]);
    assert_eq!(codes("x = 08n;"), vec![D::INVALID_BIGINT]);
    assert_eq!(codes("x = 0008n;"), vec![D::INVALID_BIGINT]);
    assert_eq!(codes("x = 00e1;"), vec![D::INVALID_NUMERIC_LITERAL]);
    assert_eq!(codes("x = 08E1;"), vec![D::INVALID_NUMERIC_LITERAL]);
}

#[test]
fn valid_numbers_never_flagged() {
    for n in [
        "123",
        "1_000",
        "1_000_000",
        "3.14",
        "1e10",
        "1E3",
        "1e1_0",
        "1.0e-1_0",
        "0xFF",
        "0xDEAD_BEEF",
        "0o17",
        "0b1010",
        "123n",
        "1_000n",
        "0xFFn",
        ".5",
        "5.",
        "0.0",
        "0",
        // legacy-octal-like decimals that are valid in sloppy mode
        "00",
        "0123",
        "08",
        "09",
        "0008",
        "08.5",
        "09e1",
        "08e-1",
    ] {
        assert!(codes(&format!("x = {n};")).is_empty(), "false positive on valid `{n}`");
    }
}

#[test]
fn numeric_adjacency() {
    use diag_code as D;
    // misplaced bigint suffix: span = the trailing `n`
    let d = diags("x = 1.5n;");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, D::INVALID_BIGINT);
    assert_eq!((d[0].off, d[0].len), (7, 1));
    assert_eq!(codes("x = 1e3n;"), vec![D::INVALID_BIGINT]);
    // digit invalid for the radix: span = that digit only (a digit is not an
    // identifier start, so the parser's run stops after one char)
    let d = diags("x = 0b12;");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, D::INVALID_NUMERIC_LITERAL);
    assert_eq!((d[0].off, d[0].len), (7, 1));
    assert_eq!(codes("x = 0o18;"), vec![D::INVALID_NUMERIC_LITERAL]);
    // number followed by an identifier: span covers the ident run
    let d = diags("x = 3in y;");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, D::INVALID_NUMERIC_LITERAL);
    assert_eq!((d[0].off, d[0].len), (5, 2)); // `in`
    let d = diags("x = 123abc;");
    assert_eq!((d[0].off, d[0].len), (7, 3)); // `abc`
    assert_eq!(codes("x = 0xFFg;"), vec![D::INVALID_NUMERIC_LITERAL]);
    // coalesce is shared by all four modes, so the jsx path fires too
    assert_eq!(codes_jsx("x = 1.5n;"), vec![D::INVALID_BIGINT]);
}

#[test]
fn valid_numeric_adjacency_never_flagged() {
    for src in [
        "x = 0x1Fn;",          // hex bigint: `n` consumed by the scanner
        "x = 123n;",           // decimal bigint
        "x = 1_000n;",         // bigint with separators
        "x = 0b101;",          // valid binary
        "x = 1..toString();",  // number `1.` then `.` property access — dot is legal
        "x = 1.e3;",           // exponent directly after the dot
        "x = 1.5.toString();", // float then member access
        "let abc123def = 1;",  // digits inside an identifier never glue as numbers
        "x = x1;",             // trailing digit in identifier
        "x = .5;",             // dot-led float
        "x = 5.;",             // trailing-dot float
        "for (var i in x) {}", // spaced `in` operator
        "x = 1 in y;",         // spaced `in` after number
        "x = a instanceof b;",
        "t = `${1}n`;",        // template text after a substitution
        "s = '1.5n';",         // string interior
        "c = /* 1.5n */ 1;",   // comment interior
        "r = /1.5n/;",         // regex interior
        "x = 1\u{a0}+ 2;",     // NBSP after number: Unicode WHITESPACE, not ident
        "x = 1\u{2028}y = 2;", // LS after number: a line terminator, valid
        "x = 1\u{3000}+ 2;",   // ideographic space after number
    ] {
        assert!(codes(src).is_empty(), "false positive on `{src}`");
    }
    // JSX text and expression containers must not glue text as numbers.
    assert!(codes_jsx("const el = <div>1.5n</div>;").is_empty());
    assert!(codes_jsx("const el = <div a=\"3in\">{1.5}</div>;").is_empty());
    // The content-blind TSX type-arg skip must not leak numeric diagnostics
    // from digit-letter adjacency inside a string type-arg.
    assert!(codes_tsx("const el = <Foo<\"3px\"> x={1}/>;").is_empty());
    assert!(codes_tsx("const el = <Foo<1.5, \"2xl\"> y/>;").is_empty());
}

#[test]
fn regex_flags() {
    use diag_code as D;
    assert_eq!(codes("r = /x/q;"), vec![D::INVALID_REGEXP_FLAG]); // unknown flag
    assert_eq!(codes("r = /x/gg;"), vec![D::DUPLICATE_REGEXP_FLAG]); // duplicate
    assert!(codes("r = /x/gimsuydv;").is_empty()); // all valid flags, once each
    assert!(codes("r = /x/;").is_empty()); // no flags
}

#[test]
fn unexpected_character() {
    use diag_code as D;
    assert_eq!(codes("a\u{1}b"), vec![D::UNEXPECTED_CHARACTER]);
    assert_eq!(codes("x = 1\u{7} 2"), vec![D::UNEXPECTED_CHARACTER]);
    assert_eq!(codes("\u{1}\u{2}"), vec![D::UNEXPECTED_CHARACTER, D::UNEXPECTED_CHARACTER]);
}

#[test]
fn line_terminator_in_string() {
    use diag_code as D;
    assert_eq!(codes("x = 'a\nb';"), vec![D::LINE_TERMINATOR_IN_STRING]);
    assert_eq!(codes("x = 'a\r\nb';"), vec![D::LINE_TERMINATOR_IN_STRING]);
    // escaped line terminators are legal continuations, CRLF as one sequence
    assert!(codes("x = 'a\\\nb';").is_empty());
    assert!(codes("x = 'a\\\rb';").is_empty());
    assert!(codes("x = 'a\\\r\nb';").is_empty());
    // escaped backslash followed by a raw terminator is invalid
    assert_eq!(codes("x = 'a\\\\\r\nb';"), vec![D::LINE_TERMINATOR_IN_STRING]);

    // Span parity: opener through the first unescaped terminator; for CRLF
    // only the CR (the parser consumes one char, then reports).
    let d = diags("x = 'a\nb';");
    assert_eq!((d[0].off, d[0].len), (4, 3));
    let d = diags("x = 'a\r\nb';");
    assert_eq!((d[0].off, d[0].len), (4, 3));
    let d = diags("x = 'a\\\\\r\nb';");
    assert_eq!((d[0].off, d[0].len), (4, 5));
}

#[test]
fn line_terminator_in_regexp() {
    use diag_code as D;
    // Our token still runs to the closing `/`; the diagnostic reproduces the
    // parser's span, opener to just past the first terminator.
    let d = diags("x = /a\nb/;");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, D::LINE_TERMINATOR_IN_REGEXP);
    assert_eq!((d[0].off, d[0].len), (4, 3)); // `/a<LF>`
    let d = diags("x = /a\r\nb/;");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, D::LINE_TERMINATOR_IN_REGEXP);
    assert_eq!((d[0].off, d[0].len), (4, 3)); // `/a<CR>`: one diag, span past the CR
    // escaped terminators and terminators inside `[...]` are also invalid
    assert_eq!(codes("x = /a\\\nb/;"), vec![D::LINE_TERMINATOR_IN_REGEXP]);
    assert_eq!(codes("x = /a\\\r\nb/;"), vec![D::LINE_TERMINATOR_IN_REGEXP]);
    // backslash at EOF: the escape peek must not read pad bytes and
    // fabricate a terminator
    assert_eq!(codes("x = /a\\"), vec![D::UNTERMINATED_REGEXP]);
    assert_eq!(codes("x = /a[\n]b/;"), vec![D::LINE_TERMINATOR_IN_REGEXP]);
    // newline then EOF: the parser stops at the terminator first
    assert_eq!(codes("x = /abc\n"), vec![D::LINE_TERMINATOR_IN_REGEXP]);
    assert_eq!(codes_jsx("x = /a\nb/;"), vec![D::LINE_TERMINATOR_IN_REGEXP]);
    assert!(codes("x = /a\\nb/g;").is_empty()); // `\n` as two chars
    assert!(codes("x = a / b\nc / d;").is_empty()); // division across lines
    assert!(codes("x = /a[b-z]+/;").is_empty());
}

#[test]
fn invalid_unicode_escape_in_string() {
    use diag_code as D;
    // Spans run from the backslash to just past the last byte the parser's
    // escape scanner consumed. (`x = ` puts the `\` at offset 5.)
    let d = diags(r#"x = "\u{110000}";"#); // out of range: span `\u{110000` (excl `}`)
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 9, D::INVALID_UNICODE_ESCAPE));
    let d = diags(r#"x = "\uZZZZ";"#); // bad hex: span `\u`
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 2, D::INVALID_UNICODE_ESCAPE));
    let d = diags(r#"x = "\u1ZZZ";"#); // one leading valid digit: span `\u1`
    assert_eq!((d[0].off, d[0].len), (5, 3));
    let d = diags(r#"x = "\xGG";"#); // bad hex \x: span `\x`
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 2, D::INVALID_UNICODE_ESCAPE));
    let d = diags(r#"x = "\x4G";"#); // partial hex: span `\x4`
    assert_eq!((d[0].off, d[0].len), (5, 3));
    let d = diags(r#"x = "\u{}";"#); // empty braces: span `\u{`
    assert_eq!((d[0].off, d[0].len), (5, 3));
    let d = diags(r#"x = "\u{12";"#); // unterminated braces: `\u{12`
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len), (5, 5));
    // the parser stops at the digit that crosses 0x10FFFF
    let d = diags(r#"x = "\u{1100000}";"#);
    assert_eq!((d[0].off, d[0].len), (5, 9)); // `\u{110000`
    // u32 wrap-around: `\u{100000041}` wraps back into range
    assert_eq!(codes(r#"x = "\u{100000041}";"#), vec![D::INVALID_UNICODE_ESCAPE]);
    let d = diags(r#"x = "\u{0041ZZZ}";"#); // non-hex break, in-range value
    assert_eq!((d[0].off, d[0].len), (5, 7)); // `\u{0041`
    let d = diags(r#"x = "\u004";"#); // truncated at the closing quote
    assert_eq!((d[0].off, d[0].len), (5, 5)); // `\u004`
    // one error per bad escape, anchored at each backslash
    assert_eq!(codes(r#"x = "\xG \xG";"#).len(), 2);
    let d = diags(r#"x = "\uD800\uZZZZ";"#); // 2nd escape bad
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len), (11, 2));
}

#[test]
fn valid_escapes_never_flagged() {
    for s in [
        r#""\u0041""#,
        r#""\u{1F600}""#,
        r#""\u{0}""#,
        r#""\u{10FFFF}""#,     // boundary: strictly-greater check
        r#""\u{0000000041}""#, // leading zeros never cross the limit
        r#""\z""#,             // NonEscapeCharacter — valid, cooks to `z`
        r#""\\n""#,            // escaped backslash + literal n
        r#""\0""#,
        r#""\01""#, // legacy octal: parser/strict-mode territory
        r#""\7""#,
        r#""\8""#,           // NonOctalDecimalEscape: parser territory
        r#""\uD800""#,       // lone surrogate: valid in strings
        r#""\uD83D\uDE00""#, // surrogate-pair escapes (two valid escapes)
        r#""😀""#,           // raw multi-byte content, no escapes
        r#""\x41""#,
    ] {
        assert!(codes(&format!("x = {s};")).is_empty(), "false positive on `{s}`");
    }
    assert!(codes("x = 'a\\\nb';").is_empty()); // LF line continuation
    assert!(codes("x = 'a\\\r\nb';").is_empty()); // CRLF line continuation
}

#[test]
fn template_invalid_escapes_stay_silent() {
    // An invalid escape in a tagged template is legal, and tagged-ness is
    // parser context; oxc_parser's lexer is also silent here.
    assert!(codes(r"t = `\uZZZZ`;").is_empty());
    assert!(codes(r"t = `\u{110000}`;").is_empty());
    assert!(codes(r"t = `\xGG`;").is_empty());
    assert!(codes(r"t = tag`\uZZ`;").is_empty());
    assert!(codes(r"t = `a${b}\uZZ`;").is_empty()); // middle/tail segments
}

#[test]
fn invalid_unicode_escape_jsx_path() {
    use diag_code as D;
    assert_eq!(codes_jsx(r#"const x = "\uZZZZ";"#), vec![D::INVALID_UNICODE_ESCAPE]);
    // JSX attribute strings have no escapes — the value is verbatim source.
    assert!(codes_jsx(r#"const el = <div a="\uZZ">x</div>;"#).is_empty());
}

#[test]
fn valid_input_emits_nothing() {
    assert!(codes("let x = 'abc';").is_empty());
    assert!(codes("const t = `a${b}c`; /* ok */ // ok\nlet r = /x/g;").is_empty());
    assert!(codes("function f(){ return 1 / 2; }").is_empty());
}

#[test]
fn invalid_unicode_character() {
    use diag_code as D;
    // Chars that are neither whitespace nor ID_Start/ID_Continue at code
    // level: U+2E2F vertical tilde, U+180E Mongolian vowel separator, emoji.
    let d = diags("var x\u{2E2F} = 1;");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 3, D::UNEXPECTED_CHARACTER));
    assert_eq!(codes("var \u{180E}x = 1;"), vec![D::UNEXPECTED_CHARACTER]);
    assert_eq!(codes("let a = \u{1F600};"), vec![D::UNEXPECTED_CHARACTER]);
    // A raw U+FFFD at code level is oxc_parser's binary-file error, distinct
    // from opt-in UTF-8 validation.
    assert_eq!(codes("var \u{FFFD} = 1;"), vec![D::INVALID_UTF8]);
}

#[test]
fn valid_unicode_never_flagged() {
    // identifier chars, whitespace, and anything inside literal content
    for src in [
        // spellchecker:disable-next-line
        "var caf\u{E9} = 1;",                   // é: ID_Continue
        "var \u{4F60}\u{597D} = 1;",            // CJK idents
        "let \u{3C0} = 3.14;",                  // π
        "x\u{00A0}= 1;",                        // NBSP whitespace
        "let s = '\u{1F600}\u{2E2F}\u{180E}';", // anything in a string
        "let t = `\u{1F600} ${x} \u{2E2F}`;",   // anything in a template
        "// \u{2E2F}\u{1F600}\nlet u = 1;",     // anything in a comment
        "/* \u{180E} */ let v = 1;",            // block comment
        "x = /\u{1F600}/u;",                    // anything in a regex body
    ] {
        assert!(codes(src).is_empty(), "false positive on valid {src:?}");
    }
}

fn diags_bytes(code: &[u8]) -> Vec<Diagnostic> {
    let mut buf = code.to_vec();
    let len = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let mut opts = default_options();
    opts.validate_utf8 = true;
    let (res, _arena) = lex_utf8(&buf, len, opts);
    res.diagnostics().to_vec()
}

#[test]
fn invalid_utf8() {
    use diag_code as D;
    // span = the maximal invalid subpart; one diagnostic per file
    let d = diags_bytes(b"x = \xFF;"); // lone invalid byte
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (4, 1, D::INVALID_UTF8));
    assert_eq!(diags_bytes(b"x = \xC3;")[0].code, D::INVALID_UTF8); // truncated 2-byte
    assert_eq!(diags_bytes(b"x = \xC3").len(), 1); // truncated at EOF
    assert_eq!(diags_bytes(b"x = \x80;")[0].code, D::INVALID_UTF8); // stray continuation
    // Overlong / surrogate / beyond-U+10FFFF: byte 2 is range-checked per
    // lead, so the invalid subpart is the lead alone.
    let d = diags_bytes(b"x = '\xE0\x80\x80';");
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 1, D::INVALID_UTF8));
    let d = diags_bytes(b"x = '\xED\xA0\x80';");
    assert_eq!((d[0].off, d[0].len), (5, 1));
    let d = diags_bytes(b"x = '\xF4\x90\x80\x80';");
    assert_eq!((d[0].off, d[0].len), (5, 1));
    // truncated 4-byte at EOF: lead + two range-valid continuations
    let d = diags_bytes(b"x = \xF0\x9F\x98");
    assert_eq!((d[0].off, d[0].len), (4, 3));
    assert_eq!(diags_bytes(b"x = \xFF + \xFF;").len(), 1); // first subpart only
    // context-free: fires inside a string body too
    let d = diags_bytes(b"x = 'a\xFFb';");
    assert_eq!((d[0].off, d[0].code), (6, D::INVALID_UTF8));
}

#[test]
fn valid_utf8_never_flagged() {
    let codes_v =
        |s: &str| -> Vec<u16> { diags_bytes(s.as_bytes()).iter().map(|d| d.code).collect() };
    for src in [
        // spellchecker:disable-next-line
        "let caf\u{e9} = 1;",        // 2-byte
        "x = '\u{1F600}';",          // 4-byte emoji in string
        "a\u{a0}b",                  // NBSP (ws-reclass path preserved)
        "let \u{4e2d}\u{6587} = 1;", // CJK identifier (3-byte)
        "x\u{2028}y = 1;",           // LS
        "s = '\u{FEFF}';",           // ZWNBSP mid-file
    ] {
        assert!(
            !codes_v(src).contains(&diag_code::INVALID_UTF8),
            "false positive on valid UTF-8 `{src}`"
        );
    }
}

#[test]
fn utf8_fuzz_against_std() {
    use diag_code as D;
    // Deterministic LCG fuzz against std::str::from_utf8: a diag iff std
    // errors, offset == valid_up_to(), len == error_len() when std has one.
    let mut s: u64 = 0x243F_6A88_85A3_08D3;
    for _ in 0..4000 {
        let mut buf = vec![b' ']; // keep offset 0 clear of hashbang/BOM handling
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let len = 3 + (s >> 59) as usize;
        for _ in 0..len {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r = (s >> 33) as u8;
            buf.push(if r & 1 == 0 { 0x40 + (r >> 3) } else { r });
        }
        let d = diags_bytes(&buf);
        let first6 = d.iter().find(|d| d.code == D::INVALID_UTF8);
        match core::str::from_utf8(&buf) {
            Ok(_) => assert!(first6.is_none(), "false positive on valid UTF-8 {buf:?}"),
            Err(e) => {
                let d6 = first6.unwrap_or_else(|| panic!("missed invalid UTF-8 in {buf:?}"));
                assert_eq!(d6.off as usize, e.valid_up_to(), "offset mismatch on {buf:?}");
                if let Some(l) = e.error_len() {
                    assert_eq!(d6.len as usize, l, "subpart len mismatch on {buf:?}");
                }
            }
        }
    }
}

#[test]
fn invalid_identifier_escape_bare() {
    use diag_code as D;
    // bare `\`: span = the one char after it, backslash excluded
    let d = diags(r"x = \A");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 1, D::INVALID_IDENTIFIER_ESCAPE));
    // backslash at EOF: empty span at n
    let d = diags(r"x = \");
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 0, D::INVALID_IDENTIFIER_ESCAPE));
    // control bytes keep their own code
    assert_eq!(codes("a\u{1}b"), vec![D::UNEXPECTED_CHARACTER]);
}

#[test]
fn invalid_identifier_escape_payload() {
    use diag_code as D;
    // spans start after the backslash, end where the parser scanner stopped
    let d = diags(r"var \uZZ = 1");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 1, D::INVALID_IDENTIFIER_ESCAPE));
    let d = diags(r"var \u004 = 1"); // 3 leading valid hex
    assert_eq!((d[0].off, d[0].len), (5, 4));
    let d = diags(r"var \u{} = 1"); // no digits; closing brace NOT consumed
    assert_eq!((d[0].off, d[0].len), (5, 2));
    let d = diags(r"var \u{110000} = 1"); // stops right after the crossing digit
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len), (5, 8));
    let d = diags(r"var \u{FFFFFFFFFF} = 1"); // ditto, incremental check
    assert_eq!((d[0].off, d[0].len), (5, 8));
    let d = diags(r"var \u{12 = 1"); // missing closing brace
    assert_eq!((d[0].off, d[0].len), (5, 4));
    // surrogates: lone (both forms) and pairs are invalid in identifiers
    let d = diags(r"var \uD800 = 1");
    assert_eq!((d[0].off, d[0].len), (5, 5));
    let d = diags(r"var \u{D800} = 1");
    assert_eq!((d[0].off, d[0].len), (5, 7));
    // a well-formed pair: one diagnostic spanning both escapes
    let src = ["var ", r"\uD800", r"\uDC00", " = 1"].concat();
    let d = diags(&src);
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len), (5, 11));
    // high + invalid low: the parser rewinds — two independent diagnostics
    let src = ["var ", r"\uD800", r"\uD800", " = 1"].concat();
    let d = diags(&src);
    assert_eq!(d.len(), 2);
    assert_eq!((d[0].off, d[0].len), (5, 5));
    assert_eq!((d[1].off, d[1].len), (11, 5));
}

#[test]
fn invalid_identifier_escape_jsx_path() {
    use diag_code as D;
    assert!(codes_jsx(r"const \u{110000} = 1").contains(&D::INVALID_IDENTIFIER_ESCAPE));
    assert!(codes_jsx(r"const el = <div>a\b</div>;").is_empty()); // JSX text
}

#[test]
fn valid_identifier_escapes_never_flagged() {
    use diag_code as D;
    for src in [
        r"let \u0041 = 1;",        // 4-digit form
        r"let \u{41} = 1;",        // brace form
        r"let \u{10FFFF} = 1;",    // boundary code point (ident-char check deferred)
        r"a\u0041b",               // mid-identifier escape (misc_post merge)
        r"n\u0065w",               // keyword lookalike via escape
        r"class A{ #\u0061 = 1 }", // private name with escape
        r"x = '\n';",              // string escapes: not this class
        r"r = /\d+/g;",            // regex escapes: interiors carved
        r"// \A",                  // comment interior
        "t = `a\nb`;",             // template interior
    ] {
        assert!(!codes(src).contains(&D::INVALID_IDENTIFIER_ESCAPE), "false positive on `{src}`");
    }
    // Out-of-range escape in a STRING is code 7's business, never code 8.
    assert!(!codes(r"s = '\u{110000}';").contains(&D::INVALID_IDENTIFIER_ESCAPE));
}

#[test]
fn empty_exponent() {
    use diag_code as D;
    // `1e` / `1e+`: the marker and sign were consumed but no digits followed
    let d = diags("x = 1e;");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (4, 2, D::INVALID_NUMERIC_LITERAL));
    let d = diags("x = 1e+;");
    assert_eq!((d[0].off, d[0].len, d[0].code), (4, 3, D::INVALID_NUMERIC_LITERAL));
    assert_eq!(codes("x = .5e;"), vec![D::INVALID_NUMERIC_LITERAL]);
    assert_eq!(codes("x = 0e;"), vec![D::INVALID_NUMERIC_LITERAL]);
    assert_eq!(codes("x = 08e;"), vec![D::INVALID_NUMERIC_LITERAL]);
    // `1en`: empty exponent and misplaced bigint suffix both fire — unlike
    // the parser, we don't abort at the first error
    let cs = codes("x = 1en;");
    assert!(cs.contains(&D::INVALID_NUMERIC_LITERAL));
    assert!(cs.contains(&D::INVALID_BIGINT));
    for n in ["1e3", "1E+5", "1e-1", "1.e3", "1e1_0", ".5e2", "0e0", "0xe", "0xEn"] {
        assert!(codes(&format!("x = {n};")).is_empty(), "false positive on `{n}`");
    }
}

#[test]
fn unicode_ident_after_number() {
    use diag_code as D;
    // same adjacency error as ASCII; span = the identifier-start run
    let d = diags("x = 1π;");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 2, D::INVALID_NUMERIC_LITERAL));
    let d = diags("x = 123abcπ;");
    assert_eq!((d[0].off, d[0].len), (7, 5)); // `abc` + 2-byte pi
    assert_eq!(codes("x = 1.5π;"), vec![D::INVALID_NUMERIC_LITERAL]);
    // unicode whitespace after a number stays clean
    assert!(codes("x = 1\u{a0}+ 2;").is_empty());
    assert!(codes("x = 1\u{2028}y = 2;").is_empty());
}

#[test]
fn escaped_char_not_identifier() {
    use diag_code as D;
    // A well-formed escape decoding to a non-identifier char: empty span
    // right after the escape (the bridge decodes backward for the message).
    let d = diags(r"var \u0020 = 1");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (10, 0, D::UNEXPECTED_CHARACTER));
    let d = diags(r"var \u{30}x = 1");
    assert_eq!((d[0].off, d[0].len, d[0].code), (10, 0, D::UNEXPECTED_CHARACTER));
    // mid-identifier escapes use is_identifier_part: digits fine there,
    assert!(codes(r"let a\u{30}b = 1;").is_empty());
    // whitespace not
    assert_eq!(codes(r"let a\u0020b = 1;"), vec![D::UNEXPECTED_CHARACTER]);
    for src in [r"let \u0041 = 1;", r"let \u{24} = 1;", r"let a\u{5F}b = 1;"] {
        assert!(codes(src).is_empty(), "false positive on `{src}`");
    }
}

#[test]
fn line_separator_in_regexp() {
    use diag_code as D;
    // LS/PS are LineTerminators: invalid in regex bodies, raw or escaped;
    // span ends past the whole 3-byte char
    let d = diags("x = /a\u{2028}b/;");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (4, 5, D::LINE_TERMINATOR_IN_REGEXP));
    assert_eq!(codes("x = /a\u{2029}b/;"), vec![D::LINE_TERMINATOR_IN_REGEXP]);
    let d = diags("x = /a\\\u{2028}b/;"); // escaped LS: still invalid
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len), (4, 6));
    assert!(codes("x = /a\u{2014}b/;").is_empty()); // other E2-led chars are fine
    assert!(codes("x = /a\u{2713}b/u;").is_empty());
    assert!(codes(r"x = /a\u2028b/;").is_empty()); // escape text, no raw LS
    assert!(codes("s = '\u{2028}';").is_empty()); // legal in strings/templates
    assert!(codes("t = `\u{2028}`;").is_empty());
}

#[test]
fn misplaced_hash() {
    use diag_code as D;
    // A `#` that opens neither a hashbang nor a private name: the parser
    // consumes the char after `#` and reports on it.
    let d = diags("\n#!/bin/sh");
    let h = d.iter().find(|x| x.code == D::UNEXPECTED_CHARACTER).unwrap();
    assert_eq!((h.off, h.len), (2, 1)); // later garbage tokens may add diags
    let d = diags("x = #;");
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].off, d[0].len, d[0].code), (5, 1, D::UNEXPECTED_CHARACTER));
    let d = diags("x = #"); // `#` at EOF: empty span at n
    assert_eq!((d[0].off, d[0].len), (5, 0));
    assert!(codes("#!/usr/bin/env node\nlet x = 1;").is_empty());
    assert!(codes("class A { #x = 1; m() { return this.#x; } }").is_empty());
}

#[test]
fn template_cooked_invalid_marker() {
    // A NotEscapeSequence makes the cooked value None; legality depends on
    // tagged-ness, so the span carries a marker bit, not a diagnostic.
    let get = |code: &str| -> Vec<bool> {
        let mut buf = code.as_bytes().to_vec();
        let len = buf.len() as u32;
        buf.resize(buf.len() + PAD, 0);
        let (res, arena) = lex_utf8(&buf, len, default_options());
        res.templates(&arena).iter().map(|t| t.cooked_invalid()).collect()
    };
    assert_eq!(get(r"t = `\uZZ`;"), vec![true]); // bad unicode escape
    assert_eq!(get(r"t = `\xG`;"), vec![true]); // bad hex escape
    assert_eq!(get(r"t = `\u{110000}`;"), vec![true]); // out of range
    assert_eq!(get(r"t = `\101`;"), vec![true]); // octal
    assert_eq!(get(r"t = `\8`;"), vec![true]); // NonOctalDecimalEscape
    assert_eq!(get(r"t = `\n`;"), vec![false]); // ordinary escape
    assert_eq!(get(r"t = `\0`;"), vec![false]); // NUL, no digit after
    assert_eq!(get("t = `abc`;"), vec![false]); // no escapes
    assert_eq!(get(r"t = `a${b}c`;"), vec![false, false]); // head + tail
    // still zero diagnostics, matching oxc_parser's silent lexer
    assert!(codes(r"t = `\101`;").is_empty());
    assert!(codes(r"t = `\uZZ`;").is_empty());
    // strings are untouched by the marker path
    assert!(codes(r#"s = "\8";"#).is_empty());
}
