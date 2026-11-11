//! TS-mode keyword recognition: the TS set is active only under
//! `LexOptions::ts`, JS mode must be unaffected, and the wider TS hash key
//! must separate the pairs the JS key cannot.

#![cfg(target_endian = "little")]

use oxc_lexer::{LexOptions, TokenKind, testing::lex_with_cpu};
use oxc_simd::{DynCpu, test_cpus};

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
fn kinds(code: &str, ts: bool, jsx: bool, cpu: DynCpu) -> Vec<TokenKind> {
    let opts = LexOptions { jsx, ts, ..Default::default() };
    let res = lex_with_cpu(code, opts, cpu);
    let count = res.token_count();
    res.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

fn first_kind(code: &str, ts: bool, cpu: DynCpu) -> TokenKind {
    kinds(code, ts, false, cpu)[0]
}

#[test_cpus]
fn ts_words_resolve_in_ts_mode_only(cpu: DynCpu) {
    for (w, tok) in TS_WORDS {
        assert_eq!(first_kind(w, true, cpu), tok, "ts mode: {w}");
        assert_eq!(first_kind(w, false, cpu), TokenKind::Ident, "js mode: {w}");
    }
}

#[test_cpus]
fn js_keywords_identical_in_both_modes(cpu: DynCpu) {
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
        assert_eq!(first_kind(w, false, cpu), tok, "js mode: {w}");
        assert_eq!(first_kind(w, true, cpu), tok, "ts mode: {w}");
    }
    // get/set are in-table placeholders and stay IDENT everywhere.
    for w in ["get", "set"] {
        assert_eq!(first_kind(w, false, cpu), TokenKind::Ident);
        assert_eq!(first_kind(w, true, cpu), TokenKind::Ident);
    }
}

#[test_cpus]
fn ts_key_separates_narrow_key_collisions(cpu: DynCpu) {
    // These pairs share (c0, c1, len) - the JS key cannot tell them apart,
    // the TS (c0, c1, last, len) key must.
    assert_eq!(first_kind("static", true, cpu), TokenKind::KwStatic);
    assert_eq!(first_kind("string", true, cpu), TokenKind::KwString);
    assert_eq!(first_kind("declare", true, cpu), TokenKind::KwDeclare);
    assert_eq!(first_kind("default", true, cpu), TokenKind::KwDefault);
    assert_eq!(first_kind("interface", true, cpu), TokenKind::KwInterface);
    assert_eq!(first_kind("intrinsic", true, cpu), TokenKind::KwIntrinsic);
    // (c0, last, len) and (c1, last, len) degenerate pairs, for key hygiene.
    assert_eq!(first_kind("true", true, cpu), TokenKind::KwTrue);
    assert_eq!(first_kind("type", true, cpu), TokenKind::KwType);
    assert_eq!(first_kind("if", true, cpu), TokenKind::KwIf);
    assert_eq!(first_kind("of", true, cpu), TokenKind::KwOf);
}

#[test_cpus]
fn near_misses_stay_ident(cpu: DynCpu) {
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
        assert_eq!(first_kind(w, true, cpu), TokenKind::Ident, "ts near-miss: {w}");
        assert_eq!(first_kind(w, false, cpu), TokenKind::Ident, "js near-miss: {w}");
    }
}

#[test_cpus]
fn member_access_words_stay_ident(cpu: DynCpu) {
    // The candidate filter drops words right after a member dot.
    assert_eq!(
        kinds("a.type", true, false, cpu),
        [TokenKind::Ident, TokenKind::Dot, TokenKind::Ident]
    );
    assert_eq!(
        kinds("a?.string", true, false, cpu),
        [TokenKind::Ident, TokenKind::OptionalChain, TokenKind::Ident]
    );
    assert_eq!(
        kinds("module.exports", true, false, cpu),
        [TokenKind::KwModule, TokenKind::Dot, TokenKind::Ident]
    );
}

#[test_cpus]
fn ts_statement_shapes(cpu: DynCpu) {
    assert_eq!(
        kinds("type X = string;", true, false, cpu),
        [TokenKind::KwType, TokenKind::Ident, TokenKind::Eq, TokenKind::KwString, TokenKind::Semi]
    );
    assert_eq!(
        kinds("interface I { readonly x: number }", true, false, cpu),
        [
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
        kinds("declare module 'x';", true, false, cpu),
        [TokenKind::KwDeclare, TokenKind::KwModule, TokenKind::String, TokenKind::Semi]
    );
    // Same input in JS mode: every TS spelling is a plain identifier.
    assert_eq!(
        kinds("type X = string;", false, false, cpu),
        [TokenKind::Ident, TokenKind::Ident, TokenKind::Eq, TokenKind::Ident, TokenKind::Semi]
    );
}

#[test_cpus]
fn tsx_mode_uses_ts_set(cpu: DynCpu) {
    assert_eq!(
        kinds("type P = { x: number };", true, true, cpu),
        [
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

#[test_cpus]
fn number_adjacency_resolves_with_active_set(cpu: DynCpu) {
    // glue_number's cold arm resolves the abutting word inline; it must use
    // the mode's set. (`3in` is invalid input; spans still tokenize.)
    let js = kinds("3in x", false, false, cpu);
    assert_eq!(js[0], TokenKind::Number);
    assert_eq!(js[1], TokenKind::KwIn);
    let ts = kinds("3is x", true, false, cpu);
    assert_eq!(ts[0], TokenKind::Number);
    assert_eq!(ts[1], TokenKind::KwIs);
    let js2 = kinds("3is x", false, false, cpu);
    assert_eq!(js2[0], TokenKind::Number);
    assert_eq!(js2[1], TokenKind::Ident);
    // `get` is contextual, so it's an identifier.
    let ts2 = kinds("3get x", true, false, cpu);
    assert_eq!(ts2[0], TokenKind::Number);
    assert_eq!(ts2[1], TokenKind::Ident);
}

#[test_cpus]
fn number_adjacency_long_words(cpu: DynCpu) {
    // A 10-byte keyword abutting a number is resolved too.
    let js = kinds("3instanceof x", false, false, cpu);
    assert_eq!(js[0], TokenKind::Number);
    assert_eq!(js[1], TokenKind::KwInstanceof);
    // A longer word starting with a keyword is not a keyword, even if it's longer than 16 bytes.
    let long = kinds("3instanceofabcdefghijklmnopqrstuvwxyz x", false, false, cpu);
    assert_eq!(long[0], TokenKind::Number);
    assert_eq!(long[1], TokenKind::Ident);
}

#[test_cpus]
fn escape_continuation_demotes_ts_keyword(cpu: DynCpu) {
    // `types` is one escaped identifier, not KW_TYPE + escape.
    let got = kinds("type\\u0073 x", true, false, cpu);
    assert_eq!(got[0], TokenKind::IdentEscaped);
}

#[test_cpus]
fn regex_vs_division_untouched_by_ts_kinds(cpu: DynCpu) {
    // Keyword-kind rewriting happens after the regex decision; `type` is
    // not a regex-preceding keyword in either mode.
    let ts = kinds("type/x/g", true, false, cpu);
    assert_eq!(ts[0], TokenKind::KwType);
    assert_eq!(ts[1], TokenKind::Slash, "ts: `/` after `type` must be division");
    let js = kinds("type/x/g", false, false, cpu);
    assert_eq!(js[1], TokenKind::Slash, "js: `/` after `type` must be division");
    // Control: after a real RX keyword it is a regex in both modes.
    assert_eq!(kinds("return /x/g", true, false, cpu)[1], TokenKind::RegExp);
}
