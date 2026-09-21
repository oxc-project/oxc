use crate::token::TokenKind;

use crate::pipeline::disambiguate::tests::{FileType, division, kinds_of, regex};

// Reduce repeated boilerplate in tests below.
// Can reference `ScriptJS` directly, instead of `FileType::ScriptJS`.
use FileType::*;

#[test]
fn yield_identifier_divides() {
    division("var yield = 1; var r = yield /2/g;", ScriptJS);
    division("function f() { var yield = 1; return yield /2/g; }", ScriptJS);
    division("function* g() { function h() { var yield = 1; return yield /2/g; } }", ScriptJS);
    division("function* g() { const a = () => { var yield = 1; return yield /2/g; }; }", ScriptJS);
    division("({ get m() { var yield = 1; return yield /2/g; } });", ScriptJS);
}

#[test]
fn yield_keyword_stays_regex() {
    regex("function* g() { yield /re/; }", ScriptJS);
    regex("function* g() { if (x) { while (y) { yield /re/; } } }", ScriptJS);
    regex("function* g() { const o = { a: yield /re/ }; }", ScriptJS);
    regex("class C { *m() { yield /re/; } }", ScriptJS);
    regex("({ *m() { yield /re/ } });", ScriptJS);
    regex("async function* ag() { yield /re/; }", ScriptJS);
    regex("function* g() { const c = class { [yield /re/](){} }; }", ScriptJS);
}

#[test]
fn strict_yield_stays_regex() {
    regex("\"use strict\"; var r = yield /2/g;", ScriptJS);
    regex("'use strict'\nvar r = yield /2/g;", ScriptJS);
    regex("function f() { \"use strict\"; return yield /2/g; }", ScriptJS);
    regex("class C { m() { return yield /2/g; } }", ScriptJS);
    division("var s = \"use strict\"; var yield = 1; var r = yield /2/g;", ScriptJS);
}

#[test]
fn await_identifier_divides() {
    division("var await = 1; var r = await /2/g;", ScriptJS);
    division("\"use strict\"; var await = 1; var r = await /2/g;", ScriptJS);
    division("async function f() { function g() { var await = 1; return await /2/g; } }", ScriptJS);
    division(
        "async function f() { const g = () => { var await = 1; return await /2/g; }; }",
        ScriptJS,
    );
    division("function f() { var await = 1; return await /2/g; }", ScriptJS);
}

#[test]
fn await_keyword_stays_regex() {
    regex("async function f() { await /re/; }", ScriptJS);
    regex("async function f() { if (x) { await /re/; } }", ScriptJS);
    regex("({ async m() { await /re/ } });", ScriptJS);
    regex("class C { async m() { await /re/ } }", ScriptJS);
    regex("const f = async () => { await /re/ };", ScriptJS);
    regex("const f = async x => { await /re/ };", ScriptJS);
    regex("x = async () => await /re/.test(s);", ScriptJS);
    regex("f(1, async () => await /re/.test(s), 2);", ScriptJS);
    regex("class C { static { await /re/ } }", ScriptJS);
}

#[test]
fn concise_bodies_pop() {
    division("var await = 1; const g = [async () => await 1, await /2/g];", ScriptJS);
    division("var await = 1; const h = (async () => await 1, await /2/g);", ScriptJS);
    division("var await = 1; const t = c ? async () => await 1 : await /2/g;", ScriptJS);
}

#[test]
fn params_take_their_functions_kind() {
    division("function* g() { function h(a = yield /2/g) {} }", ScriptJS);
    division("async function f() { function h(a = await /2/g) {} }", ScriptJS);
}

#[test]
fn module_goal_keeps_yield_and_await_reserved() {
    let ks = kinds_of("var r = await /re/.test(x);", ModuleJS);
    assert!(ks.contains(&TokenKind::RegExp), "module keeps await reserved: {ks:?}");
    let ks = kinds_of("var r = yield /re/;", ModuleJS);
    assert!(ks.contains(&TokenKind::RegExp), "module keeps yield reserved: {ks:?}");
}

#[test]
fn property_spellings_unaffected() {
    division("x.yield / 2;", ScriptJS);
    division("x.await / 2;", ScriptJS);
}

#[test]
fn fake_directive_expression_continuation() {
    division("\"use strict\"\n+ 1; var yield = 1; var r = yield /2/g;", ScriptJS);
    division("\"use strict\"\n.length; var yield = 1; var r = yield /2/g;", ScriptJS);
}

#[test]
fn leading_semicolon_ends_prologue() {
    division("; \"use strict\"; var yield = 1; var r = yield /2/g;", ScriptJS);
}

#[test]
fn concise_body_ends_by_asi() {
    division("var await = 1; var f = async () => 0\nvar r = await /2/g;", ScriptJS);
}

#[test]
fn concise_body_asi_generator_side() {
    regex("function* gen() { const f = () => 0\nyield /re/ }", ScriptJS);
}

#[test]
fn concise_body_ternary_colon_does_not_pop() {
    regex("var await = 1; x = async () => c ? a : await /re/;", ScriptJS);
}

#[test]
fn no_asi_pop_after_operator() {
    regex("var f = async () => x +\nawait /2/g;", ScriptJS);
}

#[test]
fn template_substitution_pops_concise() {
    division("var await = 1, g = 2; x = `${async () => await 1}${await /2/g}`;", ScriptJS);
}

#[test]
fn template_substitution_generator_side() {
    regex("function* g() { var x = `${() => 1}${yield /re/}`; }", ScriptJS);
    regex("var f = async () => `${await 1}` + await /re/;", ScriptJS);
}

#[test]
fn asi_pop_after_brace_tail() {
    division("var await = 1; var f = async () => y = {a: 1}\nvar r = await /2/g;", ScriptJS);
    division("var await = 1; var f = async () => y = function(){}\nvar r = await /2/g;", ScriptJS);
    regex("function* gen() { const f = () => o = {a: 1}\nyield /re/ }", ScriptJS);
}

#[test]
fn asi_pop_after_postfix_tail() {
    division("var await = 1; var f = async () => x++\nvar r = await /2/g;", ScriptJS);
    division("var await = 1; var f = async () => x--\nvar r = await /2/g;", ScriptJS);
}

#[test]
fn asi_pop_postfix_only() {
    regex(
        "function* g() { const f = () => o = {a: 1}
yield /re/ }",
        ScriptJS,
    );
    division(
        "function* g() { const f = () => ++
yield /2/g; }",
        ScriptJS,
    );
}

#[test]
fn asi_pop_across_absorbed_ls() {
    division("var await = 1; var f = async () => x\u{2028}var r = await /2/g;", ScriptJS);
}

#[test]
fn asi_pop_spaced_prefix_pair() {
    division(
        "var yield = 4, g = 2;\nfunction* gen() { const f = () => 1 + ++\nyield /2/g; }",
        ScriptJS,
    );
}

#[test]
fn method_named_function_keeps_modifiers() {
    regex("var o = { *function() { yield /re/ } };", ScriptJS);
    regex("var o = { async function() { await /re/ } };", ScriptJS);
}

#[test]
fn computed_method_modifiers() {
    regex("var o = { *['m']() { yield /re/ } };", ScriptJS);
    regex("var await = 1; var o = { async ['m']() { await /re/ } };", ScriptJS);
    regex("var await = 1; class C { async ['m']() { await /re/ } }", ScriptJS);
    regex("var await = 1; class C { static async ['m']() { await /re/ } }", ScriptJS);
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
        regex(code, ScriptJS);
    }
    regex("({ *\\u0066oo() { yield /re/ } });", ScriptJS);
    regex("class C { async *#\\u0066() { await /re/ } }", ScriptJS);
    regex(
        "class C { x = 1
async *m() { await /re/ } }",
        ScriptJS,
    );
    division("({ a: b * function() { var yield = 1; return yield /2/g; } });", ScriptJS);
    division("({ a: b * async function() { var yield = 1; return yield /2/g; } });", ScriptJS);
    division("({ get m() { var yield = 1; return yield /2/g; } });", ScriptJS);
    division("({ ['m']() { var yield = 1; return yield /2/g; } });", ScriptJS);
    division("class C { ['m']() { var await = 1; return await /2/g; } }", ScriptJS);
    division("({ a: function() { var yield = 1; return yield /2/g; } });", ScriptJS);
    regex("({ a: async function() { await /re/ } });", ScriptJS);
}

#[test]
fn mult_star_is_not_a_modifier() {
    division("var o = { a: b * function() { var yield = 1; return yield /2/g; } };", ScriptJS);
    division(
        "var o = { a: b * async function() { var yield = 1; return yield /2/g; } };",
        ScriptJS,
    );
}

#[test]
fn bigint_and_escaped_method_names() {
    regex("var o = { *1n() { yield /re/ } };", ScriptJS);
    regex("var o = { *\\u0066oo() { yield /re/ } };", ScriptJS);
}

#[test]
fn jsx_after_yield_and_await() {
    let jsx = |code: &str| kinds_of(code, ScriptJSX);
    let ks = jsx("function* items(d) { for (const x of d) yield <li id={x}/>; }");
    assert!(ks.contains(&TokenKind::JsxLt), "yielded JSX element must frame: {ks:?}");
    let ks = jsx("var await = 1, max = 10;\nif (await < max) done();");
    assert!(!ks.contains(&TokenKind::JsxLt), "await < max is a comparison: {ks:?}");
    assert!(ks.contains(&TokenKind::Lt), "expected a plain `<`: {ks:?}");
    let ks = jsx("async function f() { return await <Spinner/>; }");
    assert!(ks.contains(&TokenKind::JsxLt), "awaited JSX element must frame: {ks:?}");
    let ks = jsx("var await = 1, g = 2;\nvar el = <a b={async () => await 1} c={await /2/g}/>;");
    assert!(!ks.contains(&TokenKind::RegExp), "container leak, expected division: {ks:?}");
}

#[test]
fn walk_crosses_return_types_and_type_parameters() {
    regex("x = async (): T => { await /re/; };", ScriptTS);
    regex("x = async (): typeof cb => { await /re/; };", ScriptTS);
    regex("var $: <baz>() => 1n | T = async (): typeof cb => { await /<div>/ };", ScriptTS);
    division("x = (): T => { var await = 1; return await /2/g; };", ScriptTS);
    regex("x = async function f(): T { await /re/; };", ScriptTS);
    regex("x = async function (): Promise<T> { await /re/; };", ScriptTS);
    regex("x = async (): Promise<T> => { await /re/; };", ScriptTS);
    regex("x = function* <T>(): Generator<T> { yield /re/; };", ScriptTS);
    regex("class C { async m(): Promise<T> { await /re/; } }", ScriptTS);
    regex("switch (async function f(): typeof import('m') { await /}/; }) {}", ScriptTS);
    division("x = function (): T { var await = 1; return await /2/g; };", ScriptTS);
    division("x = (): T => { var await = 1; return await /2/g; };", ScriptTS);
}
