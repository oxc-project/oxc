use crate::{
    cpu::{DynCpu, testing::test_cpus},
    token::TokenKind,
};

use crate::pipeline::disambiguate::tests::{FileType, division, kinds_of, regex};

// Reduce repeated boilerplate in tests below.
// Can reference `ScriptJS` directly, instead of `FileType::ScriptJS`.
use FileType::*;

#[test_cpus]
fn yield_identifier_divides(cpu: DynCpu) {
    division("var yield = 1; var r = yield /2/g;", ScriptJS, cpu);
    division("function f() { var yield = 1; return yield /2/g; }", ScriptJS, cpu);
    division("function* g() { function h() { var yield = 1; return yield /2/g; } }", ScriptJS, cpu);
    division(
        "function* g() { const a = () => { var yield = 1; return yield /2/g; }; }",
        ScriptJS,
        cpu,
    );
    division("({ get m() { var yield = 1; return yield /2/g; } });", ScriptJS, cpu);
}

#[test_cpus]
fn yield_keyword_stays_regex(cpu: DynCpu) {
    regex("function* g() { yield /re/; }", ScriptJS, cpu);
    regex("function* g() { if (x) { while (y) { yield /re/; } } }", ScriptJS, cpu);
    regex("function* g() { const o = { a: yield /re/ }; }", ScriptJS, cpu);
    regex("class C { *m() { yield /re/; } }", ScriptJS, cpu);
    regex("({ *m() { yield /re/ } });", ScriptJS, cpu);
    regex("async function* ag() { yield /re/; }", ScriptJS, cpu);
    regex("function* g() { const c = class { [yield /re/](){} }; }", ScriptJS, cpu);
}

#[test_cpus]
fn strict_yield_outside_a_generator_divides(cpu: DynCpu) {
    // Strict mode makes yield a reserved word, an early error; the grammar still reads a name.
    division("\"use strict\"; var r = yield /2/g;", ScriptJS, cpu);
    division("'use strict'\nvar r = yield /2/g;", ScriptJS, cpu);
    division("function f() { \"use strict\"; return yield /2/g; }", ScriptJS, cpu);
    division("class C { m() { return yield /2/g; } }", ScriptJS, cpu);
    division("class C { static { yield /2/g; } }", ScriptJS, cpu);
    division("var s = \"use strict\"; var yield = 1; var r = yield /2/g;", ScriptJS, cpu);
}

#[test_cpus]
fn await_identifier_divides(cpu: DynCpu) {
    division("var await = 1; var r = await /2/g;", ScriptJS, cpu);
    division("\"use strict\"; var await = 1; var r = await /2/g;", ScriptJS, cpu);
    division(
        "async function f() { function g() { var await = 1; return await /2/g; } }",
        ScriptJS,
        cpu,
    );
    division(
        "async function f() { const g = () => { var await = 1; return await /2/g; }; }",
        ScriptJS,
        cpu,
    );
    division("function f() { var await = 1; return await /2/g; }", ScriptJS, cpu);
}

#[test_cpus]
fn await_keyword_stays_regex(cpu: DynCpu) {
    regex("async function f() { await /re/; }", ScriptJS, cpu);
    regex("async function f() { if (x) { await /re/; } }", ScriptJS, cpu);
    regex("({ async m() { await /re/ } });", ScriptJS, cpu);
    regex("class C { async m() { await /re/ } }", ScriptJS, cpu);
    regex("const f = async () => { await /re/ };", ScriptJS, cpu);
    regex("const f = async x => { await /re/ };", ScriptJS, cpu);
    regex("x = async () => await /re/.test(s);", ScriptJS, cpu);
    regex("f(1, async () => await /re/.test(s), 2);", ScriptJS, cpu);
    regex("class C { static { await /re/ } }", ScriptJS, cpu);
}

#[test_cpus]
fn concise_bodies_pop(cpu: DynCpu) {
    division("var await = 1; const g = [async () => await 1, await /2/g];", ScriptJS, cpu);
    division("var await = 1; const h = (async () => await 1, await /2/g);", ScriptJS, cpu);
    division("var await = 1; const t = c ? async () => await 1 : await /2/g;", ScriptJS, cpu);
}

#[test_cpus]
fn params_take_their_functions_kind(cpu: DynCpu) {
    division("function* g() { function h(a = yield /2/g) {} }", ScriptJS, cpu);
    division("async function f() { function h(a = await /2/g) {} }", ScriptJS, cpu);
}

#[test_cpus]
fn module_goal_keeps_yield_and_await_reserved(cpu: DynCpu) {
    let ks = kinds_of("var r = await /re/.test(x);", ModuleJS, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "module keeps await reserved: {ks:?}");
    let ks = kinds_of("var r = yield /re/;", ModuleJS, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "module keeps yield reserved: {ks:?}");
}

#[test_cpus]
fn property_spellings_unaffected(cpu: DynCpu) {
    division("x.yield / 2;", ScriptJS, cpu);
    division("x.await / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn concise_body_ends_by_asi(cpu: DynCpu) {
    division("var await = 1; var f = async () => 0\nvar r = await /2/g;", ScriptJS, cpu);
}

#[test_cpus]
fn concise_body_asi_generator_side(cpu: DynCpu) {
    regex("function* gen() { const f = () => 0\nyield /re/ }", ScriptJS, cpu);
}

#[test_cpus]
fn concise_body_ternary_colon_does_not_pop(cpu: DynCpu) {
    regex("var await = 1; x = async () => c ? a : await /re/;", ScriptJS, cpu);
}

#[test_cpus]
fn no_asi_pop_after_operator(cpu: DynCpu) {
    regex("var f = async () => x +\nawait /2/g;", ScriptJS, cpu);
}

#[test_cpus]
fn template_substitution_pops_concise(cpu: DynCpu) {
    division("var await = 1, g = 2; x = `${async () => await 1}${await /2/g}`;", ScriptJS, cpu);
}

#[test_cpus]
fn template_substitution_generator_side(cpu: DynCpu) {
    regex("function* g() { var x = `${() => 1}${yield /re/}`; }", ScriptJS, cpu);
    regex("var f = async () => `${await 1}` + await /re/;", ScriptJS, cpu);
}

#[test_cpus]
fn asi_pop_after_brace_tail(cpu: DynCpu) {
    division("var await = 1; var f = async () => y = {a: 1}\nvar r = await /2/g;", ScriptJS, cpu);
    division(
        "var await = 1; var f = async () => y = function(){}\nvar r = await /2/g;",
        ScriptJS,
        cpu,
    );
    regex("function* gen() { const f = () => o = {a: 1}\nyield /re/ }", ScriptJS, cpu);
}

#[test_cpus]
fn asi_pop_after_postfix_tail(cpu: DynCpu) {
    division("var await = 1; var f = async () => x++\nvar r = await /2/g;", ScriptJS, cpu);
    division("var await = 1; var f = async () => x--\nvar r = await /2/g;", ScriptJS, cpu);
}

#[test_cpus]
fn asi_pop_postfix_only(cpu: DynCpu) {
    regex(
        "function* g() { const f = () => o = {a: 1}
yield /re/ }",
        ScriptJS,
        cpu,
    );
    division(
        "function* g() { const f = () => ++
yield /2/g; }",
        ScriptJS,
        cpu,
    );
}

#[test_cpus]
fn asi_pop_across_absorbed_ls(cpu: DynCpu) {
    division("var await = 1; var f = async () => x\u{2028}var r = await /2/g;", ScriptJS, cpu);
}

#[test_cpus]
fn asi_pop_spaced_prefix_pair(cpu: DynCpu) {
    division(
        "var yield = 4, g = 2;\nfunction* gen() { const f = () => 1 + ++\nyield /2/g; }",
        ScriptJS,
        cpu,
    );
}

#[test_cpus]
fn method_named_function_keeps_modifiers(cpu: DynCpu) {
    regex("var o = { *function() { yield /re/ } };", ScriptJS, cpu);
    regex("var o = { async function() { await /re/ } };", ScriptJS, cpu);
}

#[test_cpus]
fn computed_method_modifiers(cpu: DynCpu) {
    regex("var o = { *['m']() { yield /re/ } };", ScriptJS, cpu);
    regex("var await = 1; var o = { async ['m']() { await /re/ } };", ScriptJS, cpu);
    regex("var await = 1; class C { async ['m']() { await /re/ } }", ScriptJS, cpu);
    regex("var await = 1; class C { static async ['m']() { await /re/ } }", ScriptJS, cpu);
}

#[test_cpus]
fn method_header_matrix(cpu: DynCpu) {
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
        regex(code, ScriptJS, cpu);
    }
    regex("({ *\\u0066oo() { yield /re/ } });", ScriptJS, cpu);
    regex("class C { async *#\\u0066() { await /re/ } }", ScriptJS, cpu);
    regex(
        "class C { x = 1
async *m() { await /re/ } }",
        ScriptJS,
        cpu,
    );
    division("({ a: b * function() { var yield = 1; return yield /2/g; } });", ScriptJS, cpu);
    division("({ a: b * async function() { var yield = 1; return yield /2/g; } });", ScriptJS, cpu);
    division("({ get m() { var yield = 1; return yield /2/g; } });", ScriptJS, cpu);
    division("({ ['m']() { var yield = 1; return yield /2/g; } });", ScriptJS, cpu);
    division("class C { ['m']() { var await = 1; return await /2/g; } }", ScriptJS, cpu);
    division("({ a: function() { var yield = 1; return yield /2/g; } });", ScriptJS, cpu);
    regex("({ a: async function() { await /re/ } });", ScriptJS, cpu);
}

#[test_cpus]
fn mult_star_is_not_a_modifier(cpu: DynCpu) {
    division("var o = { a: b * function() { var yield = 1; return yield /2/g; } };", ScriptJS, cpu);
    division(
        "var o = { a: b * async function() { var yield = 1; return yield /2/g; } };",
        ScriptJS,
        cpu,
    );
}

#[test_cpus]
fn bigint_and_escaped_method_names(cpu: DynCpu) {
    regex("var o = { *1n() { yield /re/ } };", ScriptJS, cpu);
    regex("var o = { *\\u0066oo() { yield /re/ } };", ScriptJS, cpu);
}

#[test_cpus]
fn jsx_after_yield_and_await(cpu: DynCpu) {
    let jsx = |code: &str| kinds_of(code, ScriptJSX, cpu);
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

#[test_cpus]
fn continued_walk_leaves_the_group_it_stopped_in(cpu: DynCpu) {
    // The second slash continues the walk of the first, which stopped inside a group now closed.
    for code in [
        "o = { a: f({} / 2), b: {} / 3 };",
        "o = { a: function() {} / 2, b: {} / 3 };",
        "o = { a: class { m() { x = {} / 2 } }, b: {} / 3 };",
    ] {
        division(code, ScriptJS, cpu);
    }
    regex("async function f() { if (a) {} /x/ }\ng();\nx = await\n{} /re/g", ScriptJS, cpu);
}

#[test_cpus]
fn walk_crosses_return_types_and_type_parameters(cpu: DynCpu) {
    regex("x = async (): T => { await /re/; };", ScriptTS, cpu);
    regex("x = async (): typeof cb => { await /re/; };", ScriptTS, cpu);
    regex("var $: <baz>() => 1n | T = async (): typeof cb => { await /<div>/ };", ScriptTS, cpu);
    division("x = (): T => { var await = 1; return await /2/g; };", ScriptTS, cpu);
    regex("x = async function f(): T { await /re/; };", ScriptTS, cpu);
    regex("x = async function (): Promise<T> { await /re/; };", ScriptTS, cpu);
    regex("x = async (): Promise<T> => { await /re/; };", ScriptTS, cpu);
    regex("x = function* <T>(): Generator<T> { yield /re/; };", ScriptTS, cpu);
    regex("class C { async m(): Promise<T> { await /re/; } }", ScriptTS, cpu);
    regex("switch (async function f(): typeof import('m') { await /}/; }) {}", ScriptTS, cpu);
    division("x = function (): T { var await = 1; return await /2/g; };", ScriptTS, cpu);
    division("x = (): T => { var await = 1; return await /2/g; };", ScriptTS, cpu);
}
