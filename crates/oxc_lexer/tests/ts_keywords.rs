//! TS-mode keyword recognition: the TS set is active only under
//! `LexOptions::ts`, JS mode must be unaffected, and the wider TS hash key
//! must separate the pairs the JS key cannot.
#![cfg(target_endian = "little")]

use oxc_lexer::{Lexer, PAD, TokenKind, default_options};

/// All 35 TS-mode additions, mirroring `KEYWORDS_TS_EXTRA` (kept literal so
/// a table typo cannot hide behind shared constants).
const TS_WORDS: [(&str, TokenKind); 35] = [
    ("abstract", TokenKind::KwAbstract),
    ("accessor", TokenKind::KwAccessor),
    ("any", TokenKind::KwAny),
    ("asserts", TokenKind::KwAsserts),
    ("bigint", TokenKind::KwBigInt),
    ("boolean", TokenKind::KwBoolean),
    ("declare", TokenKind::KwDeclare),
    ("global", TokenKind::KwGlobal),
    ("implements", TokenKind::KwImplements),
    ("infer", TokenKind::KwInfer),
    ("interface", TokenKind::KwInterface),
    ("intrinsic", TokenKind::KwIntrinsic),
    ("is", TokenKind::KwIs),
    ("keyof", TokenKind::KwKeyof),
    ("module", TokenKind::KwModule),
    ("namespace", TokenKind::KwNamespace),
    ("never", TokenKind::KwNever),
    ("number", TokenKind::KwNumber),
    ("object", TokenKind::KwObject),
    ("out", TokenKind::KwOut),
    ("override", TokenKind::KwOverride),
    ("package", TokenKind::KwPackage),
    ("private", TokenKind::KwPrivate),
    ("protected", TokenKind::KwProtected),
    ("public", TokenKind::KwPublic),
    ("readonly", TokenKind::KwReadonly),
    ("require", TokenKind::KwRequire),
    ("satisfies", TokenKind::KwSatisfies),
    ("string", TokenKind::KwString),
    ("symbol", TokenKind::KwSymbol),
    ("type", TokenKind::KwType),
    ("undefined", TokenKind::KwUndefined),
    ("unique", TokenKind::KwUnique),
    ("unknown", TokenKind::KwUnknown),
    ("using", TokenKind::KwUsing),
];

/// Lex and return the non-trivia kinds (EOF dropped).
fn kinds(code: &str, ts: bool, jsx: bool) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.ts = ts;
    opts.jsx = jsx;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    lx.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

fn first_kind(code: &str, ts: bool) -> TokenKind {
    kinds(code, ts, false)[0]
}

#[test]
fn ts_words_resolve_in_ts_mode_only() {
    for (w, tok) in TS_WORDS {
        assert_eq!(first_kind(w, true), tok, "ts mode: {w}");
        assert_eq!(first_kind(w, false), TokenKind::Ident, "js mode: {w}");
    }
}

#[test]
fn js_keywords_identical_in_both_modes() {
    for (w, tok) in [
        ("return", TokenKind::KwReturn),
        ("instanceof", TokenKind::KwInstanceof),
        ("await", TokenKind::KwAwait),
        ("let", TokenKind::KwLet),
        ("static", TokenKind::KwStatic),
        ("as", TokenKind::KwAs),
        ("of", TokenKind::KwOf),
        ("null", TokenKind::KwNull),
    ] {
        assert_eq!(first_kind(w, false), tok, "js mode: {w}");
        assert_eq!(first_kind(w, true), tok, "ts mode: {w}");
    }
    // get/set are in-table placeholders and stay IDENT everywhere.
    for w in ["get", "set"] {
        assert_eq!(first_kind(w, false), TokenKind::Ident);
        assert_eq!(first_kind(w, true), TokenKind::Ident);
    }
}

#[test]
fn ts_key_separates_narrow_key_collisions() {
    // These pairs share (c0, c1, len) — the JS key cannot tell them apart,
    // the TS (c0, c1, last, len) key must.
    assert_eq!(first_kind("static", true), TokenKind::KwStatic);
    assert_eq!(first_kind("string", true), TokenKind::KwString);
    assert_eq!(first_kind("declare", true), TokenKind::KwDeclare);
    assert_eq!(first_kind("default", true), TokenKind::KwDefault);
    assert_eq!(first_kind("interface", true), TokenKind::KwInterface);
    assert_eq!(first_kind("intrinsic", true), TokenKind::KwIntrinsic);
    // (c0, last, len) and (c1, last, len) degenerate pairs, for key hygiene.
    assert_eq!(first_kind("true", true), TokenKind::KwTrue);
    assert_eq!(first_kind("type", true), TokenKind::KwType);
    assert_eq!(first_kind("if", true), TokenKind::KwIf);
    assert_eq!(first_kind("of", true), TokenKind::KwOf);
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
        assert_eq!(first_kind(w, true), TokenKind::Ident, "ts near-miss: {w}");
        assert_eq!(first_kind(w, false), TokenKind::Ident, "js near-miss: {w}");
    }
}

#[test]
fn member_access_words_stay_ident() {
    // The candidate filter drops words right after a member dot.
    assert_eq!(
        kinds("a.type", true, false),
        vec![TokenKind::Ident, TokenKind::Dot, TokenKind::Ident]
    );
    assert_eq!(
        kinds("a?.string", true, false),
        vec![TokenKind::Ident, TokenKind::OptionalChain, TokenKind::Ident]
    );
    assert_eq!(
        kinds("module.exports", true, false),
        vec![TokenKind::KwModule, TokenKind::Dot, TokenKind::Ident]
    );
}

#[test]
fn ts_statement_shapes() {
    assert_eq!(
        kinds("type X = string;", true, false),
        vec![
            TokenKind::KwType,
            TokenKind::Ident,
            TokenKind::Eq,
            TokenKind::KwString,
            TokenKind::Semi
        ]
    );
    assert_eq!(
        kinds("interface I { readonly x: number }", true, false),
        vec![
            TokenKind::KwInterface,
            TokenKind::Ident,
            TokenKind::LBrace,
            TokenKind::KwReadonly,
            TokenKind::Ident,
            TokenKind::Colon,
            TokenKind::KwNumber,
            TokenKind::RBrace
        ]
    );
    assert_eq!(
        kinds("declare module 'x';", true, false),
        vec![TokenKind::KwDeclare, TokenKind::KwModule, TokenKind::String, TokenKind::Semi]
    );
    // Same input in JS mode: every TS spelling is a plain identifier.
    assert_eq!(
        kinds("type X = string;", false, false),
        vec![TokenKind::Ident, TokenKind::Ident, TokenKind::Eq, TokenKind::Ident, TokenKind::Semi]
    );
}

#[test]
fn tsx_mode_uses_ts_set() {
    assert_eq!(
        kinds("type P = { x: number };", true, true),
        vec![
            TokenKind::KwType,
            TokenKind::Ident,
            TokenKind::Eq,
            TokenKind::LBrace,
            TokenKind::Ident,
            TokenKind::Colon,
            TokenKind::KwNumber,
            TokenKind::RBrace,
            TokenKind::Semi
        ]
    );
}

#[test]
fn number_adjacency_resolves_with_active_set() {
    // glue_number's cold arm resolves the abutting word inline; it must use
    // the mode's set. (`3in` is invalid input; spans still tokenize.)
    let js = kinds("3in x", false, false);
    assert_eq!(js[0], TokenKind::Number);
    assert_eq!(js[1], TokenKind::KwIn);
    let ts = kinds("3is x", true, false);
    assert_eq!(ts[0], TokenKind::Number);
    assert_eq!(ts[1], TokenKind::KwIs);
    let js2 = kinds("3is x", false, false);
    assert_eq!(js2[1], TokenKind::Ident);
}

#[test]
fn escape_continuation_demotes_ts_keyword() {
    // `types` is one escaped identifier, not KW_TYPE + escape.
    let got = kinds("type\\u0073 x", true, false);
    assert_eq!(got[0], TokenKind::IdentEscaped);
}

#[test]
fn regex_vs_division_untouched_by_ts_kinds() {
    // Keyword-kind rewriting happens after the regex decision; `type` is
    // not a regex-preceding keyword in either mode.
    let ts = kinds("type/x/g", true, false);
    assert_eq!(ts[0], TokenKind::KwType);
    assert_eq!(ts[1], TokenKind::Slash, "ts: `/` after `type` must be division");
    let js = kinds("type/x/g", false, false);
    assert_eq!(js[1], TokenKind::Slash, "js: `/` after `type` must be division");
    // Control: after a real RX keyword it is a regex in both modes.
    assert_eq!(kinds("return /x/g", true, false)[1], TokenKind::RegExp);
}
