use crate::{Lexer, PAD, options::default_options, token::TokenKind};

fn kinds_of(code: &str, module: bool) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.source_type_module = module;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    lx.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

#[track_caller]
fn regex(code: &str) {
    let ks = kinds_of(code, false);
    assert!(ks.contains(&TokenKind::RegExp), "expected regex in {code:?}: kinds {ks:?}");
}

#[track_caller]
fn division(code: &str) {
    let ks = kinds_of(code, false);
    assert!(!ks.contains(&TokenKind::RegExp), "expected division in {code:?}: kinds {ks:?}");
    assert!(ks.contains(&TokenKind::Slash), "expected a `/` in {code:?}: kinds {ks:?}");
}

#[test]
fn yield_identifier_divides() {
    division("var yield = 1; var r = yield /2/g;");
    division("function f() { var yield = 1; return yield /2/g; }");
    division("function* g() { function h() { var yield = 1; return yield /2/g; } }");
    division("function* g() { const a = () => { var yield = 1; return yield /2/g; }; }");
    division("({ get m() { var yield = 1; return yield /2/g; } });");
}

#[test]
fn yield_keyword_stays_regex() {
    regex("function* g() { yield /re/; }");
    regex("function* g() { if (x) { while (y) { yield /re/; } } }");
    regex("function* g() { const o = { a: yield /re/ }; }");
    regex("class C { *m() { yield /re/; } }");
    regex("({ *m() { yield /re/ } });");
    regex("async function* ag() { yield /re/; }");
    regex("function* g() { const c = class { [yield /re/](){} }; }");
}

#[test]
fn strict_yield_stays_regex() {
    regex("\"use strict\"; var r = yield /2/g;");
    regex("'use strict'\nvar r = yield /2/g;");
    regex("function f() { \"use strict\"; return yield /2/g; }");
    regex("class C { m() { return yield /2/g; } }");
    division("var s = \"use strict\"; var yield = 1; var r = yield /2/g;");
}

#[test]
fn await_identifier_divides() {
    division("var await = 1; var r = await /2/g;");
    division("\"use strict\"; var await = 1; var r = await /2/g;");
    division("async function f() { function g() { var await = 1; return await /2/g; } }");
    division("async function f() { const g = () => { var await = 1; return await /2/g; }; }");
    division("function f() { var await = 1; return await /2/g; }");
}

#[test]
fn await_keyword_stays_regex() {
    regex("async function f() { await /re/; }");
    regex("async function f() { if (x) { await /re/; } }");
    regex("({ async m() { await /re/ } });");
    regex("class C { async m() { await /re/ } }");
    regex("const f = async () => { await /re/ };");
    regex("const f = async x => { await /re/ };");
    regex("x = async () => await /re/.test(s);");
    regex("f(1, async () => await /re/.test(s), 2);");
    regex("class C { static { await /re/ } }");
}

#[test]
fn concise_bodies_pop() {
    division("var await = 1; const g = [async () => await 1, await /2/g];");
    division("var await = 1; const h = (async () => await 1, await /2/g);");
    division("var await = 1; const t = c ? async () => await 1 : await /2/g;");
}

#[test]
fn params_take_their_functions_kind() {
    division("function* g() { function h(a = yield /2/g) {} }");
    division("async function f() { function h(a = await /2/g) {} }");
}

#[test]
fn module_gate_skips_replay() {
    let ks = kinds_of("var r = await /re/.test(x);", true);
    assert!(ks.contains(&TokenKind::RegExp), "module keeps await reserved: {ks:?}");
    let ks = kinds_of("var r = yield /re/;", true);
    assert!(ks.contains(&TokenKind::RegExp), "module keeps yield reserved: {ks:?}");
}

#[test]
fn property_spellings_unaffected() {
    division("x.yield / 2;");
    division("x.await / 2;");
}

#[test]
fn fake_directive_expression_continuation() {
    division("\"use strict\"\n+ 1; var yield = 1; var r = yield /2/g;");
    division("\"use strict\"\n.length; var yield = 1; var r = yield /2/g;");
}

#[test]
fn leading_semicolon_ends_prologue() {
    division("; \"use strict\"; var yield = 1; var r = yield /2/g;");
}

#[test]
fn concise_body_ends_by_asi() {
    division("var await = 1; var f = async () => 0\nvar r = await /2/g;");
}

#[test]
fn concise_body_asi_generator_side() {
    regex("function* gen() { const f = () => 0\nyield /re/ }");
}

#[test]
fn concise_body_ternary_colon_does_not_pop() {
    regex("var await = 1; x = async () => c ? a : await /re/;");
}

#[test]
fn no_asi_pop_after_operator() {
    regex("var f = async () => x +\nawait /2/g;");
}

#[test]
fn template_substitution_pops_concise() {
    division("var await = 1, g = 2; x = `${async () => await 1}${await /2/g}`;");
}

#[test]
fn template_substitution_generator_side() {
    regex("function* g() { var x = `${() => 1}${yield /re/}`; }");
    regex("var f = async () => `${await 1}` + await /re/;");
}

#[test]
fn asi_pop_after_brace_tail() {
    division("var await = 1; var f = async () => y = {a: 1}\nvar r = await /2/g;");
    division("var await = 1; var f = async () => y = function(){}\nvar r = await /2/g;");
    regex("function* gen() { const f = () => o = {a: 1}\nyield /re/ }");
}

#[test]
fn asi_pop_after_postfix_tail() {
    division("var await = 1; var f = async () => x++\nvar r = await /2/g;");
    division("var await = 1; var f = async () => x--\nvar r = await /2/g;");
}

#[test]
fn asi_pop_postfix_only() {
    regex(
        "function* g() { const f = () => o = {a: 1}
yield /re/ }",
    );
    division(
        "function* g() { const f = () => ++
yield /2/g; }",
    );
}

#[test]
fn asi_pop_across_absorbed_ls() {
    division("var await = 1; var f = async () => x\u{2028}var r = await /2/g;");
}

#[test]
fn asi_pop_spaced_prefix_pair() {
    division("var yield = 4, g = 2;\nfunction* gen() { const f = () => 1 + ++\nyield /2/g; }");
}

#[test]
fn method_named_function_keeps_modifiers() {
    regex("var o = { *function() { yield /re/ } };");
    regex("var o = { async function() { await /re/ } };");
}

#[test]
fn computed_method_modifiers() {
    regex("var o = { *['m']() { yield /re/ } };");
    regex("var await = 1; var o = { async ['m']() { await /re/ } };");
    regex("var await = 1; class C { async ['m']() { await /re/ } }");
    regex("var await = 1; class C { static async ['m']() { await /re/ } }");
}

#[test]
fn method_header_matrix() {
    for code in [
        "({ *m() { yield /re/ } });",
        "({ *['m']() { yield /re/ } });",
        "({ *[k]() { yield /re/ } });",
        "({ *'m'() { yield /re/ } });",
        "({ *42() { yield /re/ } });",
        "({ *function() { yield /re/ } });",
        "({ async m() { await /re/ } });",
        "({ async ['m']() { await /re/ } });",
        "({ async function() { await /re/ } });",
        "({ async *m() { await /re/ } });",
        "({ async *[k]() { yield /re/ } });",
        "class C { *m() { yield /re/ } }",
        "class C { *['m']() { yield /re/ } }",
        "class C { static *m() { yield /re/ } }",
        "class C { static async *[k]() { await /re/ } }",
        "class C { async [k]() { await /re/ } }",
        "class C { async #p() { await /re/ } }",
    ] {
        regex(code);
    }
    regex("({ *\\u0066oo() { yield /re/ } });");
    regex("class C { async *#\\u0066() { await /re/ } }");
    regex(
        "class C { x = 1
async *m() { await /re/ } }",
    );
    division("({ a: b * function() { var yield = 1; return yield /2/g; } });");
    division("({ a: b * async function() { var yield = 1; return yield /2/g; } });");
    division("({ get m() { var yield = 1; return yield /2/g; } });");
    division("({ ['m']() { var yield = 1; return yield /2/g; } });");
    division("class C { ['m']() { var await = 1; return await /2/g; } }");
    division("({ a: function() { var yield = 1; return yield /2/g; } });");
    regex("({ a: async function() { await /re/ } });");
}

#[test]
fn mult_star_is_not_a_modifier() {
    division("var o = { a: b * function() { var yield = 1; return yield /2/g; } };");
    division("var o = { a: b * async function() { var yield = 1; return yield /2/g; } };");
}

#[test]
fn bigint_and_escaped_method_names() {
    regex("var o = { *1n() { yield /re/ } };");
    regex("var o = { *\\u0066oo() { yield /re/ } };");
}
