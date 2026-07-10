//! TS-mode keyword recognition: the TS set is active only under
//! `LexOptions::ts`, JS mode must be unaffected, and the wider TS hash key
//! must separate the pairs the JS key cannot.
#![cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "bmi2"))]

use oxc_lexer::{Lexer, PAD, default_options, token_kind as k};

/// All 35 TS-mode additions, mirroring `KEYWORDS_TS_EXTRA` (kept literal so
/// a table typo cannot hide behind shared constants).
const TS_WORDS: [(&str, u8); 35] = [
    ("abstract", k::KW_ABSTRACT),
    ("accessor", k::KW_ACCESSOR),
    ("any", k::KW_ANY),
    ("asserts", k::KW_ASSERTS),
    ("bigint", k::KW_BIGINT),
    ("boolean", k::KW_BOOLEAN),
    ("declare", k::KW_DECLARE),
    ("global", k::KW_GLOBAL),
    ("implements", k::KW_IMPLEMENTS),
    ("infer", k::KW_INFER),
    ("interface", k::KW_INTERFACE),
    ("intrinsic", k::KW_INTRINSIC),
    ("is", k::KW_IS),
    ("keyof", k::KW_KEYOF),
    ("module", k::KW_MODULE),
    ("namespace", k::KW_NAMESPACE),
    ("never", k::KW_NEVER),
    ("number", k::KW_NUMBER),
    ("object", k::KW_OBJECT),
    ("out", k::KW_OUT),
    ("override", k::KW_OVERRIDE),
    ("package", k::KW_PACKAGE),
    ("private", k::KW_PRIVATE),
    ("protected", k::KW_PROTECTED),
    ("public", k::KW_PUBLIC),
    ("readonly", k::KW_READONLY),
    ("require", k::KW_REQUIRE),
    ("satisfies", k::KW_SATISFIES),
    ("string", k::KW_STRING),
    ("symbol", k::KW_SYMBOL),
    ("type", k::KW_TYPE),
    ("undefined", k::KW_UNDEFINED),
    ("unique", k::KW_UNIQUE),
    ("unknown", k::KW_UNKNOWN),
    ("using", k::KW_USING),
];

/// Lex and return the non-trivia kinds (EOF dropped).
fn kinds(code: &str, ts: bool, jsx: bool) -> Vec<u8> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.ts = ts;
    opts.jsx = jsx;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    lx.kinds[..count - 1].iter().copied().filter(|&kk| !oxc_lexer::is_trivia(kk)).collect()
}

fn first_kind(code: &str, ts: bool) -> u8 {
    kinds(code, ts, false)[0]
}

#[test]
fn ts_words_resolve_in_ts_mode_only() {
    for (w, tok) in TS_WORDS {
        assert_eq!(first_kind(w, true), tok, "ts mode: {w}");
        assert_eq!(first_kind(w, false), k::IDENT, "js mode: {w}");
    }
}

#[test]
fn js_keywords_identical_in_both_modes() {
    for (w, tok) in [
        ("return", k::KW_RETURN),
        ("instanceof", k::KW_INSTANCEOF),
        ("await", k::KW_AWAIT),
        ("let", k::KW_LET),
        ("static", k::KW_STATIC),
        ("as", k::KW_AS),
        ("of", k::KW_OF),
        ("null", k::KW_NULL),
    ] {
        assert_eq!(first_kind(w, false), tok, "js mode: {w}");
        assert_eq!(first_kind(w, true), tok, "ts mode: {w}");
    }
    // get/set are in-table placeholders and stay IDENT everywhere.
    for w in ["get", "set"] {
        assert_eq!(first_kind(w, false), k::IDENT);
        assert_eq!(first_kind(w, true), k::IDENT);
    }
}

#[test]
fn ts_key_separates_narrow_key_collisions() {
    // These pairs share (c0, c1, len) — the JS key cannot tell them apart,
    // the TS (c0, c1, last, len) key must.
    assert_eq!(first_kind("static", true), k::KW_STATIC);
    assert_eq!(first_kind("string", true), k::KW_STRING);
    assert_eq!(first_kind("declare", true), k::KW_DECLARE);
    assert_eq!(first_kind("default", true), k::KW_DEFAULT);
    assert_eq!(first_kind("interface", true), k::KW_INTERFACE);
    assert_eq!(first_kind("intrinsic", true), k::KW_INTRINSIC);
    // (c0, last, len) and (c1, last, len) degenerate pairs, for key hygiene.
    assert_eq!(first_kind("true", true), k::KW_TRUE);
    assert_eq!(first_kind("type", true), k::KW_TYPE);
    assert_eq!(first_kind("if", true), k::KW_IF);
    assert_eq!(first_kind("of", true), k::KW_OF);
}

#[test]
fn near_misses_stay_ident() {
    for w in [
        // spellchecker:off
        "types",
        "strin",
        "stringg",
        "interfac",
        "interfacee",
        "intrinsics",
        "implementss",
        "undefine",
        "undefinedd",
        "usin",
        "usingg",
        "keyo",
        "arguments",
        "prototype",
        "Number",
        "String",
        // spellchecker:on
    ] {
        assert_eq!(first_kind(w, true), k::IDENT, "ts near-miss: {w}");
        assert_eq!(first_kind(w, false), k::IDENT, "js near-miss: {w}");
    }
}

#[test]
fn member_access_words_stay_ident() {
    // The candidate filter drops words right after a member dot.
    assert_eq!(kinds("a.type", true, false), vec![k::IDENT, k::DOT, k::IDENT]);
    assert_eq!(kinds("a?.string", true, false), vec![k::IDENT, k::OPTIONAL_CHAIN, k::IDENT]);
    assert_eq!(kinds("module.exports", true, false), vec![k::KW_MODULE, k::DOT, k::IDENT]);
}

#[test]
fn ts_statement_shapes() {
    assert_eq!(
        kinds("type X = string;", true, false),
        vec![k::KW_TYPE, k::IDENT, k::EQ, k::KW_STRING, k::SEMI]
    );
    assert_eq!(
        kinds("interface I { readonly x: number }", true, false),
        vec![
            k::KW_INTERFACE,
            k::IDENT,
            k::LBRACE,
            k::KW_READONLY,
            k::IDENT,
            k::COLON,
            k::KW_NUMBER,
            k::RBRACE
        ]
    );
    assert_eq!(
        kinds("declare module 'x';", true, false),
        vec![k::KW_DECLARE, k::KW_MODULE, k::STRING, k::SEMI]
    );
    // Same input in JS mode: every TS spelling is a plain identifier.
    assert_eq!(
        kinds("type X = string;", false, false),
        vec![k::IDENT, k::IDENT, k::EQ, k::IDENT, k::SEMI]
    );
}

#[test]
fn tsx_mode_uses_ts_set() {
    assert_eq!(
        kinds("type P = { x: number };", true, true),
        vec![
            k::KW_TYPE,
            k::IDENT,
            k::EQ,
            k::LBRACE,
            k::IDENT,
            k::COLON,
            k::KW_NUMBER,
            k::RBRACE,
            k::SEMI
        ]
    );
}

#[test]
fn number_adjacency_resolves_with_active_set() {
    // glue_number's cold arm resolves the abutting word inline; it must use
    // the mode's set. (`3in` is invalid input; spans still tokenize.)
    let js = kinds("3in x", false, false);
    assert_eq!(js[0], k::NUMBER);
    assert_eq!(js[1], k::KW_IN);
    let ts = kinds("3is x", true, false);
    assert_eq!(ts[0], k::NUMBER);
    assert_eq!(ts[1], k::KW_IS);
    let js2 = kinds("3is x", false, false);
    assert_eq!(js2[1], k::IDENT);
}

#[test]
fn escape_continuation_demotes_ts_keyword() {
    // `types` is one escaped identifier, not KW_TYPE + escape.
    let got = kinds("type\\u0073 x", true, false);
    assert_eq!(got[0], k::IDENT_ESCAPED);
}

#[test]
fn regex_vs_division_untouched_by_ts_kinds() {
    // Keyword-kind rewriting happens after the regex decision; `type` is
    // not a regex-preceding keyword in either mode.
    let ts = kinds("type/x/g", true, false);
    assert_eq!(ts[0], k::KW_TYPE);
    assert_eq!(ts[1], k::SLASH, "ts: `/` after `type` must be division");
    let js = kinds("type/x/g", false, false);
    assert_eq!(js[1], k::SLASH, "js: `/` after `type` must be division");
    // Control: after a real RX keyword it is a regex in both modes.
    assert_eq!(kinds("return /x/g", true, false)[1], k::REGEXP);
}
