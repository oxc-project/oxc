use crate::{
    cpu::{DynCpu, testing::test_cpus},
    token::TokenKind,
};

use crate::pipeline::disambiguate::tests::{
    FileType, diag_codes_of, division, kinds_of, regex, stream,
};

// Reduce repeated boilerplate in tests below.
// Can reference `ScriptJS` directly, instead of `FileType::ScriptJS`.
use FileType::*;

#[test_cpus]
fn debugger_precedes_regex(cpu: DynCpu) {
    regex("debugger\n/re/.test(x);", ScriptJS, cpu);
    division("x.debugger / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn for_of_head_precedes_regex(cpu: DynCpu) {
    regex("for (x of /re/) ;", ScriptJS, cpu);
    regex("for ([a, b] of /re/) ;", ScriptJS, cpu);
    regex("for ({a} of /re/) ;", ScriptJS, cpu);
    regex("for (const x of /re/) ;", ScriptJS, cpu);
    regex("for await (x of /re/) ;", ScriptJS, cpu);
    regex("for ([a, of] of /re/) ;", ScriptJS, cpu);
    division("var of = 1; of / 2;", ScriptJS, cpu);
    division("instance/of/g;", ScriptJS, cpu);
    division("for (of / 2;;) ;", ScriptJS, cpu);
    division("for (of of of / 2) ;", ScriptJS, cpu);
    division("f(x, of / 2);", ScriptJS, cpu);
}

#[test_cpus]
fn unicode_ident_postfix_is_division(cpu: DynCpu) {
    division("\u{53d8}\u{91cf}++ / b;", ScriptJS, cpu);
    division("\u{53d8}\u{91cf} ++ / b;", ScriptJS, cpu);
    regex("a\u{2028}++/re/.lastIndex;", ScriptJS, cpu);
    regex("a\u{2029}++/re/.lastIndex;", ScriptJS, cpu);
    division("a\u{00a0}++ / b;", ScriptJS, cpu);
    division("a\u{200a}++ / b;", ScriptJS, cpu);
    division("a\u{feff}++ / b;", ScriptJS, cpu);
    division("a\u{200d}b++ / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn operand_keywords_before_value_braces(cpu: DynCpu) {
    division("x = typeof {} / 2;", ScriptJS, cpu);
    division("x = void {} / 2;", ScriptJS, cpu);
    division("f(new class {} / 2);", ScriptJS, cpu);
    division("return function(){} / 2;", ScriptJS, cpu);
    division("throw {} / 2;", ScriptJS, cpu);
    division("if (k in {} / 2) ;", ScriptJS, cpu);
    division("switch (x) { case class {} / 2: break; }", ScriptJS, cpu);
    regex("return\nclass C {} /re/.test(x);", ScriptJS, cpu);
    regex("export default class C {} /re/.test(x);", ScriptJS, cpu);
}

#[test_cpus]
fn object_literal_heritage(cpu: DynCpu) {
    division("(class C extends {valueOf(){}} {} / 2);", ScriptJS, cpu);
    division("(class extends {a:1}.constructor {} / 2);", ScriptJS, cpu);
    regex("x = class {}\n{} /re/.test(s);", ScriptJS, cpu);
    regex("x = class C extends {a:1} {}\n{} /re/.test(s);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_implements_heritage(cpu: DynCpu) {
    division("(class C implements I, J {} / 2);", ScriptTS, cpu);
    division("(class C extends B implements I, J {} / 2);", ScriptTS, cpu);
    regex("class C implements I, J {} /re/.test(x);", ScriptTS, cpu);
    regex("x = class A {}, y\n{} /re/.test(s);", ScriptTS, cpu);
}

#[test_cpus]
fn decorated_class_expression(cpu: DynCpu) {
    division("x = @dec class {} / 2;", ScriptTS, cpu);
    division("x = @ns.dec() class {} / 2;", ScriptTS, cpu);
    division("f(@a @b(1) class {} / 2);", ScriptTS, cpu);
    regex("@dec class C {} /re/.test(x);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_angle_before_brace(cpu: DynCpu) {
    division("(class C<T> {} / 2);", ScriptTS, cpu);
    division("(class C extends B<T> {} / 2);", ScriptTS, cpu);
    division("(class C<T extends {a: 1}> {} / 2);", ScriptTS, cpu);
    division("(class C<T = X> {} / 2);", ScriptTS, cpu);
    division("x = f < T > {} / re / g;", ScriptTS, cpu);
    division("x = a < b + c > {} / 2;", ScriptTS, cpu);
    division("(a > {} / 2);", ScriptTS, cpu);
    regex("class C<T> {} /re/.test(x);", ScriptTS, cpu);
    regex("interface I<T> {} /re/.test(x);", ScriptTS, cpu);
    regex("declare class C<T> {} /re/.test(x);", ScriptTS, cpu);
    regex("class C extends B<T> {}\n/re/.test(x);", ScriptTS, cpu);
}

#[test_cpus]
fn brace_tail_before_as(cpu: DynCpu) {
    division("let v = {a: 1} as T / y;", ScriptTS, cpu);
    division("let v = {} as A<B> / y;", ScriptTS, cpu);
    division("let v = {} satisfies T / y;", ScriptTS, cpu);
    division("let v = ({} as T) / y;", ScriptTS, cpu);
}

#[test_cpus]
fn template_literal_types_cross(cpu: DynCpu) {
    division("(class C<T extends `a${X}`> {} / 2);", ScriptTS, cpu);
    division("(class C<T extends `a${`b${Y}`}`> {} / 2);", ScriptTS, cpu);
    division("let v = x as A<`a${B}`> / y;", ScriptTS, cpu);
    regex("class C<T extends `a${X}`> {} /re/.test(x);", ScriptTS, cpu);
}

#[test_cpus]
fn hidden_trivia_segments(cpu: DynCpu) {
    division("a\u{00a0}++ / b;", ScriptJS, cpu);
    division("a\u{200a}++ / b;", ScriptJS, cpu);
    division("a\u{feff}++ / b;", ScriptJS, cpu);
    division("x = a \u{00a0}++ / b;", ScriptJS, cpu);
    regex("return\u{00a0}++/re/.lastIndex;", ScriptJS, cpu);
    division("x.\u{00a0}return++ / 2;", ScriptJS, cpu);
    regex("a\u{2028}++/re/.lastIndex;", ScriptJS, cpu);
    regex("f(a,\u{00a0}++/re/.lastIndex);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_as_type_brace(cpu: DynCpu) {
    division("let v = x as {} / y;", ScriptTS, cpu);
    division("let v = x as {a: 1} / y;", ScriptTS, cpu);
    division("let v = f() as {} / y;", ScriptTS, cpu);
    division("let v = x satisfies {} / y;", ScriptTS, cpu);
    regex("f();\nas: {} /re/.test(s);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_as_angle_form(cpu: DynCpu) {
    division("let v = x as A<B> / y;", ScriptTS, cpu);
    division("let v = x as a.b.C<D> / y;", ScriptTS, cpu);
    division("let v = f() as A<B<C>> / y;", ScriptTS, cpu);
    division("let v = x satisfies A<B> / y;", ScriptTS, cpu);
    regex("let q = a > /re/.source.length;", ScriptTS, cpu);
    regex("let q = (x < y) > /re/.source;", ScriptTS, cpu);
    regex("g(x => /re/.test(x));", ScriptTS, cpu);
}

#[test_cpus]
fn ts_return_type_function_expr(cpu: DynCpu) {
    division("x = function(): T {} / 2;", ScriptTS, cpu);
    division("x = function(): a.b.T {} / 2;", ScriptTS, cpu);
    division("x = function(): Map<A, B> {} / 2;", ScriptTS, cpu);
    division("x = async function(): T {} / 2;", ScriptTS, cpu);
    regex("function f(): T {} /re/.test(x);", ScriptTS, cpu);
    regex("function f(): Map<A, B> {} /re/.test(x);", ScriptTS, cpu);
    regex("L: {} /re/.test(x);", ScriptTS, cpu);
    regex("switch (x) { case f(y): {} /re/.test(s); }", ScriptTS, cpu);
}

#[test_cpus]
fn async_function_expression_value(cpu: DynCpu) {
    division("x = async function(){} / 2;", ScriptJS, cpu);
    regex("async function f(){} /re/.test(x);", ScriptJS, cpu);
}

#[test_cpus]
fn slash_dense_chains(cpu: DynCpu) {
    let count = |code: &str, want_re: usize, want_slash: usize| {
        let ks = kinds_of(code, ScriptJS, cpu);
        let re = ks.iter().filter(|&&kk| kk == TokenKind::RegExp).count();
        let sl = ks.iter().filter(|&&kk| kk == TokenKind::Slash).count();
        assert_eq!((re, sl), (want_re, want_slash), "{code:?}: kinds {ks:?} (regexps, slashes)");
    };
    count("x = /a/g / /b/g;", 2, 1);
    count("x = /a/ / /b/;", 2, 1);
    count("x = a / /b/ / c;", 1, 2);
    count("x = /a/ / b / c;", 1, 2);
    count("x = /a/.lastIndex / 2;", 1, 1);
    count("x /= /re/.source.length;", 1, 0);
    count("x = /a/ instanceof /b/ ? 1 : 2;", 2, 0);
    count("x = /a/ in /b/ ? 1 : 2;", 2, 0);
    count("f(/a/, /b/, a / b);", 2, 1);
    count("x = [/a/, /b/][i] / 2;", 2, 1);
    count("x = `${/a/.source}` / 2;", 1, 1);
}

#[test_cpus]
fn colon_member_and_ternary_values(cpu: DynCpu) {
    division("({a: function(){} / 2, b: 1});", ScriptJS, cpu);
    division("({a: {} / 2});", ScriptJS, cpu);
    division("({a: {b: {} / 2}});", ScriptJS, cpu);
    division("({a: class {} / 2});", ScriptJS, cpu);
    division("x = c ? y : {} / 2;", ScriptJS, cpu);
    division("x = c ? d ? a : b : {} / 2;", ScriptJS, cpu);
    regex("L: {} /re/.test(x);", ScriptJS, cpu);
    regex("{ L: function f(){} /re/ }", ScriptJS, cpu);
    regex("switch (x) { case a: function f(){} /re/ }", ScriptJS, cpu);
    regex("switch (x) { case f(y), z: {} /re/.test(s); }", ScriptJS, cpu);
    regex("c ? a : b\nL: {} /re/.test(s);", ScriptJS, cpu);
    regex("if (c) {} L: {} /re/.test(s);", ScriptJS, cpu);
}

#[test_cpus]
fn postfix_incdec_then_slash_is_division(cpu: DynCpu) {
    division("a++ / b;", ScriptJS, cpu);
    division("a-- / b;", ScriptJS, cpu);
    division("a ++ / b;", ScriptJS, cpu);
    division("x[i]++ / n;", ScriptJS, cpu);
    division("f(x)++ / n;", ScriptJS, cpu);
    division("a.return++ / 2;", ScriptJS, cpu);
    division("obj.#f++ / 2;", ScriptJS, cpu);
    division("a = b++/c/g;", ScriptJS, cpu);
    division("a++\n/ b / c;", ScriptJS, cpu);
}

#[test_cpus]
fn prefix_incdec_then_slash_is_regex(cpu: DynCpu) {
    regex("++/re/.lastIndex;", ScriptJS, cpu);
    regex("x = ++/re/.lastIndex;", ScriptJS, cpu);
    regex("(a, ++/re/.lastIndex);", ScriptJS, cpu);
    regex("f(++/re/.lastIndex);", ScriptJS, cpu);
    regex("return++/re/.lastIndex;", ScriptJS, cpu);
    regex("a + ++/re/.lastIndex;", ScriptJS, cpu);
    regex("a ** ++/re/.lastIndex;", ScriptJS, cpu);
    regex("if (a) ++/re/.lastIndex;", ScriptJS, cpu);
    regex("while (a) --/re/.lastIndex;", ScriptJS, cpu);
}

#[test_cpus]
fn line_terminator_forces_prefix(cpu: DynCpu) {
    regex("a\n++/re/.lastIndex;", ScriptJS, cpu);
    regex("a\r\n++/re/.lastIndex;", ScriptJS, cpu);
    regex("a\u{2028}++/re/.lastIndex;", ScriptJS, cpu);
    regex("a\u{2029}++/re/.lastIndex;", ScriptJS, cpu);
    regex("a /* x\ny */ ++/re/.lastIndex;", ScriptJS, cpu);
    division("a /* xy */ ++ / b;", ScriptJS, cpu);
}

#[test_cpus]
fn incdec_runs_keep_maximal_munch(cpu: DynCpu) {
    regex("a+++/re/;", ScriptJS, cpu);
    regex("a---/re/;", ScriptJS, cpu);
    division("a++/b/;", ScriptJS, cpu);
}

#[test_cpus]
fn default_and_extends_precede_regex(cpu: DynCpu) {
    regex("export default /^x$/;", ScriptJS, cpu);
    regex("export default /re/.source;", ScriptJS, cpu);
    regex("class C extends /re/.constructor {}", ScriptJS, cpu);
    division("x.default / 2;", ScriptJS, cpu);
    division("x.extends / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn statement_head_paren_then_regex(cpu: DynCpu) {
    regex("if (x) /re/.test(y);", ScriptJS, cpu);
    regex("if (f(x)) /re/.test(y);", ScriptJS, cpu);
    regex("while (x) /re/.exec(y);", ScriptJS, cpu);
    regex("for (;;) /re/.test(x);", ScriptJS, cpu);
    regex("with (o) /re/.test(x);", ScriptJS, cpu);
    regex("for await (x of y) /re/.test(x);", ScriptJS, cpu);
    regex("do x; while (y) /re/.test(z);", ScriptJS, cpu);
    regex("if (a) while (b) /re/.test(c);", ScriptJS, cpu);
}

#[test_cpus]
fn value_paren_then_slash_stays_division(cpu: DynCpu) {
    division("f(x) / 2;", ScriptJS, cpu);
    division("(a + b) / 2;", ScriptJS, cpu);
    division("x.if(a) / 2;", ScriptJS, cpu);
    division("x?.while(a) / 2;", ScriptJS, cpu);
    division("if (a) (b) / c / d;", ScriptJS, cpu);
    division("await (x) / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn class_expression_brace_then_slash_is_division(cpu: DynCpu) {
    division("(class {} / 2);", ScriptJS, cpu);
    division("(class C {} / 2);", ScriptJS, cpu);
    division("(class extends B {} / 2);", ScriptJS, cpu);
    division("(class C extends B {} / 2);", ScriptJS, cpu);
    division("(class C extends f(B) {} / 2);", ScriptJS, cpu);
    division("(class C extends a.b[0] {} / 2);", ScriptJS, cpu);
    division("x = class {} / 2;", ScriptJS, cpu);
    division("f(class {m(){}} / 2);", ScriptJS, cpu);
    division("`${class {} / 2}`;", ScriptJS, cpu);
}

#[test_cpus]
fn class_declaration_brace_then_slash_is_regex(cpu: DynCpu) {
    regex("class C {} /re/.test(x);", ScriptJS, cpu);
    regex("class C extends B {}\n/re/.test(x);", ScriptJS, cpu);
    regex("{ class C {} } /re/.test(x);", ScriptJS, cpu);
}

#[test_cpus]
fn block_braces_keep_the_regex_answer(cpu: DynCpu) {
    regex("{} /re/.test(x);", ScriptJS, cpu);
    regex(";{} /re/.test(x);", ScriptJS, cpu);
    regex("L: {} /re/.test(x);", ScriptJS, cpu);
    regex("if(a){}else{} /'/.test(s);\nconst t='x';", ScriptJS, cpu);
    regex("x = () => {}\n/re/.test(s);", ScriptJS, cpu);
}

#[test_cpus]
fn value_braces_keep_the_division_answer(cpu: DynCpu) {
    division("f({} / 2);", ScriptJS, cpu);
    division("x = function(){} / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn plain_contexts_unchanged(cpu: DynCpu) {
    division("a / b;", ScriptJS, cpu);
    division("1n / 2;", ScriptJS, cpu);
    regex("a + /re/g;", ScriptJS, cpu);
    regex("x = /re/;", ScriptJS, cpu);
    regex("f(/re/);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_postfix_bang_unchanged(cpu: DynCpu) {
    let ks = kinds_of("x! / 2;", ScriptTS, cpu);
    assert!(!ks.contains(&TokenKind::RegExp), "x! / 2 must stay division: {ks:?}");
}

#[test_cpus]
fn bare_gt_object_rhs_is_division(cpu: DynCpu) {
    division("x = f < T > {} / re / g;", ScriptJS, cpu);
    division("x = a > {} / 2;", ScriptJS, cpu);
    division("x = a >> {} / 2;", ScriptJS, cpu);
    division("x = a >>> {} / 2;", ScriptJS, cpu);
    division("x = a-- > {} / 2;", ScriptJS, cpu);
    division("x = a >\n{} / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn arrow_block_bodies_still_regex(cpu: DynCpu) {
    regex("x = y => {}\n/re/.test(s);", ScriptJS, cpu);
    regex("x = async () => {}\n/re/.test(s);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_angle_close_resolved(cpu: DynCpu) {
    let ks = kinds_of("class C<T> {} /re/.test(x);", ScriptTS, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "TS class decl with type params: {ks:?}");
    let ks = kinds_of("x = f < T > {} / re / g;", ScriptTS, cpu);
    assert!(!ks.contains(&TokenKind::RegExp), "TS relational re-read must divide: {ks:?}");
}

#[test_cpus]
fn unicode_ident_tail_resolved(cpu: DynCpu) {
    division("\u{53d8}\u{91cf}++ / b;", ScriptJS, cpu);
    regex("a\u{2028}++/re/.lastIndex;", ScriptJS, cpu);
}

#[test_cpus]
fn of_trade_resolved(cpu: DynCpu) {
    regex("for (x of /re/) ;", ScriptJS, cpu);
    division("var of = 1; of / 2;", ScriptJS, cpu);
    division("instance/of/g;", ScriptJS, cpu);
}

#[test_cpus]
fn jsx_operand_positions(cpu: DynCpu) {
    let jsx = |code: &str| kinds_of(code, ScriptJSX, cpu);
    let ks = jsx("export default <App/>;");
    assert!(ks.contains(&TokenKind::JsxLt), "export default <App/> must open JSX: {ks:?}");
    let ks = jsx("if (x) <App/>;");
    assert!(ks.contains(&TokenKind::JsxLt), "if (x) <App/> must open JSX: {ks:?}");
    let ks = jsx("x = a < b;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a < b is a comparison: {ks:?}");
    let ks = jsx("f(x) < y;");
    assert!(!ks.contains(&TokenKind::JsxLt), "f(x) < y is a comparison: {ks:?}");
    let ks = jsx("a++ < b;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a++ < b is a comparison: {ks:?}");
    let ks = jsx("x = a > {} < b;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a > {{}} < b is a comparison chain: {ks:?}");
}

#[test_cpus]
fn ts_declarator_type_annotation_forces_asi(cpu: DynCpu) {
    regex("let x: T\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: string\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: number\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: A | B\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: T[]\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: (T)\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: typeof y\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: a.b.C\n/re/.exec(s);", ScriptTS, cpu);
    regex("var x: T\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: T\n/re/gimsuy.exec(s);", ScriptTS, cpu);
    regex("let a = 1, b: T\n/re/.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_declarator_type_annotation_already_resolved(cpu: DynCpu) {
    regex("let x: Array<T>\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: {a: T}\n/re/.exec(s);", ScriptTS, cpu);
    regex("interface I { a: T }\n/re/.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_initialised_declarator_stays_division(cpu: DynCpu) {
    division("let x= T\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: number = 1\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: T = y\n/re/g.exec(s);", ScriptTS, cpu);
    division("let a = b\n/hi/g.exec(c);", ScriptJS, cpu);
    division("let a = b, c = d\n/hi/g.exec(e);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_type_alias_forces_asi(cpu: DynCpu) {
    regex("type A = T\n/re/.exec(s);", ScriptTS, cpu);
    regex("type A = B | C\n/re/.exec(s);", ScriptTS, cpu);
    regex("declare function f(): T\n/re/.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn declarator_without_initializer_forces_asi(cpu: DynCpu) {
    regex("let x\n/re/.exec(s);", ScriptJS, cpu);
    regex("var a\n/re/.test(b);", ScriptJS, cpu);
    regex("let x\n/re/.exec(s);", ScriptTS, cpu);
    regex("let a, b\n/re/.test(c);", ScriptJS, cpu);
    regex("let a = 1, b\n/re/.test(c);", ScriptJS, cpu);
}

#[test_cpus]
fn module_specifier_forces_asi(cpu: DynCpu) {
    regex("import y from 'y'\n/re/.exec(s);", ScriptJS, cpu);
    regex("import 'y'\n/re/.exec(s);", ScriptJS, cpu);
    regex("export * from 'y'\n/re/.exec(s);", ScriptJS, cpu);
    regex("export {a} from 'y'\n/re/.exec(s);", ScriptJS, cpu);
    division("x = 'y' / 2;", ScriptJS, cpu);
    division("x = f('y') / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn restricted_production_keywords_precede_regex(cpu: DynCpu) {
    regex("for(;;){ break\n/re/.test(b); }", ScriptJS, cpu);
    regex("for(;;){ continue\n/re/.test(b); }", ScriptJS, cpu);
    division("x.break / 2;", ScriptJS, cpu);
    division("x.continue / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn ts_return_type_keyword_name_is_not_operand(cpu: DynCpu) {
    regex("function f(): void {}\n/re/.test(x);", ScriptTS, cpu);
    regex("function f(): void {} /re/.test(x);", ScriptTS, cpu);
    division("x = function(): void {} / 2;", ScriptTS, cpu);
    division("x = void {} / 2;", ScriptJS, cpu);
    division("x = typeof {} / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn ts_composite_type_forms_force_asi(cpu: DynCpu) {
    regex("let x: A extends B ? C : D\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: (a: T) => U\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: 'lit'\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: [A, B]\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: readonly A[]\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: keyof T\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: Map<A, B<C>>\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: T | null\n/re/.exec(s);", ScriptTS, cpu);
    regex("declare const x: T\n/re/.exec(s);", ScriptTS, cpu);
    regex("export const x: T\n/re/.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_pathological_type_annotation_forces_asi(cpu: DynCpu) {
    regex(
        r#"let x: {
  readonly [K in keyof T as `get${Capitalize<K & string>}`]-?:
    T[K] extends infer U extends (...a: [x: string, ...r: number[]]) => infer R
      ? new (m: typeof import("./m").default) => U extends { a: infer V } ? V : R
      : { [k: string]: readonly string[] }
} & (abstract new () => void) & T
/re/g.exec(s)"#,
        ScriptTS,
        cpu,
    );
    regex("let x: {[K in keyof T as `g${K & string}`]-?: T[K]}\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: T extends infer U extends F ? A : B\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: typeof import(\"./m\").default\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: abstract new () => void\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: (...a: [x: string, ...r: number[]]) => R\n/re/g.exec(s);", ScriptTS, cpu);
    division(
        r#"let x = y as {
  readonly [K in keyof T as `get${Capitalize<K & string>}`]-?: T[K]
} & (abstract new () => void) & T
/re/g.exec(s)"#,
        ScriptTS,
        cpu,
    );
}

#[test_cpus]
fn ts_type_literal_brace_forces_asi(cpu: DynCpu) {
    regex("type A = { a: T }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("type A = {}\n/re/g.exec(s);", ScriptTS, cpu);
    regex("type A<T> = { a: T }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("type A = { [K in keyof T]: T[K] }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("type A = { (): T }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: A | { a: T }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: A & { a: T }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: A extends B ? C : { a: T }\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: A extends B ? { a: T } : C\n/re/g.exec(s);", ScriptTS, cpu);
    regex("declare function f(): A | { a: T }\n/re/g.exec(s);", ScriptTS, cpu);
    division("x = {a: 1}\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = {a: 1}\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: T = {a: 1}\n/re/g.exec(s);", ScriptTS, cpu);
    division("x = class {}\n/re/g.exec(s);", ScriptTS, cpu);
    division("let v = {} as T\n/re/g.exec(s);", ScriptTS, cpu);
    division("f({} / 2);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_as_void_is_division(cpu: DynCpu) {
    division("let x = y as void\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y as void / 2;", ScriptTS, cpu);
    division("let x = y satisfies void\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: T = y as void\n/re/g.exec(s);", ScriptTS, cpu);
    regex("x = void /re/.source;", ScriptTS, cpu);
    regex("let x: void\n/re/g.exec(s);", ScriptTS, cpu);
    regex("function f(): void {}\n/re/.test(x);", ScriptTS, cpu);
    division("x.as / 2;", ScriptTS, cpu);
}

#[test_cpus]
fn unicode_line_separator_after_token(cpu: DynCpu) {
    division("a++\u{2028}/re/g.exec(s);", ScriptJS, cpu);
    division("a++\u{2029}/re/g.exec(s);", ScriptJS, cpu);
    division("a--\u{2028}/re/g.exec(s);", ScriptJS, cpu);
    division("a++\u{00a0}/re/g.exec(s);", ScriptJS, cpu);
    division("let x = y as void\u{2028}/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y as void\u{2029}/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y as void\u{00a0}/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y! as void\u{2028}/re/g.exec(s);", ScriptTS, cpu);
    regex("a\u{2028}++/re/.lastIndex;", ScriptJS, cpu);
    regex("a\u{2029}++/re/.lastIndex;", ScriptJS, cpu);
    regex("return\u{00a0}++/re/.lastIndex;", ScriptJS, cpu);
    regex("let x: T\u{2028}/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: {a: T}\u{2028}/re/g.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_non_null_before_as_is_division(cpu: DynCpu) {
    division("let x = y! as {a: T}\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y! as {}\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y! as Array<T>\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y! as a.b.C<D>\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y! as import('m').T<U>\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y!! as {a: T} / 2;", ScriptTS, cpu);
    division("let x = f(y)! as {a: T} / 2;", ScriptTS, cpu);
    division("let x = y! satisfies {a: T} / 2;", ScriptTS, cpu);
    division("let x = !y\n/re/g.test(s);", ScriptTS, cpu);
    regex("x = !/re/.test(s);", ScriptTS, cpu);
    regex("if (!a) /re/.test(s);", ScriptTS, cpu);
}

#[test_cpus]
fn ts_import_type_chain_head(cpu: DynCpu) {
    division("let x = y as import('m').T<U>\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y as import('m').T<U> / 2;", ScriptTS, cpu);
    division("let x = y satisfies import('m').T<U>\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = y as import('m').a.b.T<U>\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: import('m').T<U>\n/re/g.exec(s);", ScriptTS, cpu);
    regex("let x: typeof import('./m').default\n/re/g.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn value_ternary_and_arrow_stay_division(cpu: DynCpu) {
    division("let x = c ? a : b\n/re/.exec(s);", ScriptTS, cpu);
    division("let x = c ? a : b\n/re/.exec(s);", ScriptJS, cpu);
    division("let f = (a) => b\n/re/.exec(s);", ScriptJS, cpu);
    division("let x: T = c ? a : b\n/re/.exec(s);", ScriptTS, cpu);
    division("export const x: T = 1\n/re/g.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn tsx_asi_then_jsx_element(cpu: DynCpu) {
    let tsx = |code: &str| kinds_of(code, ScriptTSX, cpu);
    let ks = tsx("let v: T\n<div />;");
    assert!(ks.contains(&TokenKind::JsxLt), "type-annotation ASI must open JSX: {ks:?}");
    let ks = tsx("let v\n<div />;");
    assert!(ks.contains(&TokenKind::JsxLt), "uninitialised declarator ASI must open JSX: {ks:?}");
    let ks = tsx("import R from 'r'\n<div />;");
    assert!(ks.contains(&TokenKind::JsxLt), "module-specifier ASI must open JSX: {ks:?}");
    let ks = tsx("const a = 1, div = 2;\nexport const r = a < div;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a < div stays a comparison: {ks:?}");
}

#[test_cpus]
fn ts_numeric_literal_type_forces_asi(cpu: DynCpu) {
    regex("let x: 1\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: -1\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: 1n\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: -1n\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: 0x1f\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: 1e3\n/re/.exec(s);", ScriptTS, cpu);
    regex("let a = 1, b: 2\n/re/.exec(s);", ScriptTS, cpu);
    regex("type A = 1\n/re/.exec(s);", ScriptTS, cpu);
    regex("declare function f(): 1\n/re/.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn numeric_value_stays_division(cpu: DynCpu) {
    division("let x = 1\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: number = 1\n/re/g.exec(s);", ScriptTS, cpu);
    division("x = a - 1\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: T = a - 1\n/re/g.exec(s);", ScriptTS, cpu);
    division("x = 1\n/re/g.exec(s);", ScriptJS, cpu);
    division("x = 1n\n/re/g.exec(s);", ScriptJS, cpu);
    division("x = 0x1f\n/re/g.exec(s);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_template_literal_type_forces_asi(cpu: DynCpu) {
    regex("let x: `lit`\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: `p${string}s`\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: `${number}`\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x: `a${'b'}c${number}d`\n/re/.exec(s);", ScriptTS, cpu);
    regex("type A = `lit`\n/re/.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn template_value_stays_division(cpu: DynCpu) {
    division("let x = `lit`\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x = `p${y}s`\n/re/g.exec(s);", ScriptTS, cpu);
    division("let x: T = `lit`\n/re/g.exec(s);", ScriptTS, cpu);
    division("x = `lit`\n/re/g.exec(s);", ScriptJS, cpu);
    division("x = `p${y}s`\n/re/g.exec(s);", ScriptJS, cpu);
}

#[test_cpus]
fn ts_definite_assignment_forces_asi(cpu: DynCpu) {
    regex("let x!: T\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x!: T[]\n/re/.exec(s);", ScriptTS, cpu);
    regex("let x!: 1\n/re/.exec(s);", ScriptTS, cpu);
    regex("let a = 1, b!: T\n/re/.exec(s);", ScriptTS, cpu);
    division("x! / 2;", ScriptTS, cpu);
    division("x!\n/re/g.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn restricted_production_label_precedes_regex(cpu: DynCpu) {
    regex("outer: for(;;){ break outer\n/re/.test(b); }", ScriptJS, cpu);
    regex("outer: for(;;){ continue outer\n/re/.test(b); }", ScriptJS, cpu);
    regex("outer: for(;;){ break outer\n/re/.test(b); }", ScriptTS, cpu);
    division("outer: for(;;){ break\nouter\n/re/g.test(b); }", ScriptJS, cpu);
    division("x.break / 2;", ScriptJS, cpu);
    division("x.continue / 2;", ScriptJS, cpu);
}

#[test_cpus]
fn tsx_literal_type_asi_then_jsx_element(cpu: DynCpu) {
    let tsx = |code: &str| kinds_of(code, ScriptTSX, cpu);
    for code in [
        "let v: 1\n<div />;",
        "let v: -1\n<div />;",
        "let v: 1n\n<div />;",
        "let v: `lit`\n<div />;",
        "let v: `p${string}s`\n<div />;",
        "let v!: T\n<div />;",
    ] {
        let ks = tsx(code);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    }
    let ks = kinds_of("outer: for(;;){ break outer\n<div />; }", ScriptJSX, cpu);
    assert!(ks.contains(&TokenKind::JsxLt), "labelled break ASI must open JSX: {ks:?}");
    let ks = tsx("const a = 1, div = 2;\nconst r = a - 1 < div;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a - 1 < div stays a comparison: {ks:?}");
}

#[test_cpus]
fn named_function_expression_body_then_slash_is_division(cpu: DynCpu) {
    division("const x = function foo() {} /42/i", ScriptJS, cpu);
    division("(function foo() {} /42/i)", ScriptJS, cpu);
    division("!function fn() {} /42/i;", ScriptJS, cpu);
    division("x = function* generation() {} /42/i;", ScriptJS, cpu);
    division("void async function fn() {}\n/foo/g", ScriptJS, cpu);
    division("x = async function* g() {}\n/foo/g", ScriptJS, cpu);
    regex("function foo() {}\n/42/i", ScriptJS, cpu);
    regex("function* generation() {}\n/42/i", ScriptJS, cpu);
    regex("async function fn() {}\n/42/i", ScriptJS, cpu);
}

#[test_cpus]
fn postfix_increment_then_block_then_regex(cpu: DynCpu) {
    regex("x++\n{}\n/foo/", ScriptJS, cpu);
    regex("x--\n{}\n/foo/", ScriptJS, cpu);
    regex("a[0]++\n{}\n/foo/", ScriptJS, cpu);
    division("x = y++ / 2", ScriptJS, cpu);
}

#[test_cpus]
fn class_body_after_a_heritage_list_with_type_arguments_precedes_regex(cpu: DynCpu) {
    for code in [
        "class Foo implements A, V<T> {}\n/el/.exec(s);",
        "class Foo implements A, V<T<U>> {}\n/el/.exec(s);",
        "class Foo extends y implements React.FC, V<unique symbol, true> {\n\n}\n/el/.exec(s);",
        "interface I extends A, V<T> {}\n/el/.exec(s);",
    ] {
        regex(code, ScriptTS, cpu);
    }
    division("x = class implements A, V<T> {} / 2;", ScriptTS, cpu);
    division("x = class extends A implements B, V<T<U>> {} / 2;", ScriptTS, cpu);
}

#[test_cpus]
fn function_or_class_expression_after_arrow_or_shift_is_a_value(cpu: DynCpu) {
    for code in [
        "x = a => function(){} / b / c;",
        "x = () => function y(){} / b / c;",
        "x = async () => function(){} / b / c;",
        "x = a => class {} / b / c;",
        "x = a >> function(){} / 2 / 3;",
        "x = (null >> async function (){} / n);",
        "x = a >>> function(){} / 2;",
        "x = a > function(){} / 2;",
        "x = a => class {}\n/re/.test(s);",
    ] {
        division(code, ScriptJS, cpu);
    }
    division("x = <T>function(){} / 2;", ScriptTS, cpu);
    regex("x = a => {}\n/re/.test(s);", ScriptJS, cpu);
    regex("x = () => {}\n/re/.test(s);", ScriptJS, cpu);
}

#[test_cpus]
fn function_expression_return_type_then_body_is_a_value(cpu: DynCpu) {
    for code in [
        "x = [function (): T {}\n/ 2 / 3];",
        "x = [function (): x is T {}\n/ 2 / 3];",
        "x = [function (): asserts x is T {}\n/ 2 / 3];",
        "x = [function (): this is T {}\n/ 2 / 3];",
        "x = [function (): A<B> {}\n/ 2 / 3];",
        "x = [function (): { a: T } {}\n/ 2 / 3];",
        "x = [function <T>(a: T): T {}\n/ 2 / 3];",
        "x = [async function (): Promise<T> {}\n/ 2 / 3];",
        "x = [function* (): Generator<T> {}\n/ 2 / 3];",
        "x = [function f(): T {}\n/ 2 / 3];",
        "x = function(): T {}\n/ 2 / 3;",
    ] {
        division(code, ScriptTS, cpu);
    }
    regex("function f(): T {}\n/re/.test(x);", ScriptTS, cpu);
    regex("function f(): { a: T } {}\n/re/.test(x);", ScriptTS, cpu);
    regex("let x: { a: T }\n/re/.test(x);", ScriptTS, cpu);
    let ks = kinds_of("x = [function (): T {}\n< y];", ScriptTSX, cpu);
    assert!(!ks.contains(&TokenKind::JsxLt), "{ks:?}");
    assert!(ks.contains(&TokenKind::Lt), "{ks:?}");
    let codes = diag_codes_of("x = [function (): T {}\n< y];", ScriptTSX, cpu);
    assert!(codes.is_empty(), "{codes:?}");
}

#[test_cpus]
fn label_colon_inside_a_function_expression_body_is_a_statement(cpu: DynCpu) {
    for (code, want) in [
        (
            "x = function (){\nouter: for (;;) { break outer; }\n/x*/;\n};",
            "IDENT = function ( ) { IDENT : for ( ; ; ) { break IDENT ; } REGEXP ; } ;",
        ),
        (
            "x = function (){\nouter: { break outer; }\n/x*/;\n};",
            "IDENT = function ( ) { IDENT : { break IDENT ; } REGEXP ; } ;",
        ),
        (
            "(function (){\nouter: for (;;) { break outer; }\n/x*/;\n});",
            "( function ( ) { IDENT : for ( ; ; ) { break IDENT ; } REGEXP ; } ) ;",
        ),
        (
            "x = () => {\nouter: { break outer; }\n/x*/;\n};",
            "IDENT = ( ) => { IDENT : { break IDENT ; } REGEXP ; } ;",
        ),
        (
            "x = class { m() { outer: { break outer; }\n/x*/; } };",
            "IDENT = class { IDENT ( ) { IDENT : { break IDENT ; } REGEXP ; } } ;",
        ),
        (
            "x = function (){ a: {} /re/.test(s); };",
            "IDENT = function ( ) { IDENT : { } REGEXP . IDENT ( IDENT ) ; } ;",
        ),
    ] {
        stream(code, ScriptJS, want, cpu);
    }
    division("x = function(){ return {a: {} / 2} };", ScriptJS, cpu);
    division("x = function(){ a = c ? d : {} / 2 };", ScriptJS, cpu);
    division("x = { a: {} / 2 };", ScriptJS, cpu);
}

#[test_cpus]
fn bodiless_function_signature_before_a_line_break_ends_the_statement(cpu: DynCpu) {
    for (code, want) in [
        ("declare function y()\n/[/\\]]/.x;", "declare function IDENT ( ) REGEXP . IDENT ;"),
        (
            "declare function y(a: T)\n/re/.x;",
            "declare function IDENT ( IDENT : IDENT ) REGEXP . IDENT ;",
        ),
        (
            "declare function f<T>(p: T)\n/re/.x;",
            "declare function IDENT < IDENT > ( IDENT : IDENT ) REGEXP . IDENT ;",
        ),
        (
            "export declare function y()\n/re/.x;",
            "export declare function IDENT ( ) REGEXP . IDENT ;",
        ),
        (
            "function f(a: T)\n/re/.exec(s);",
            "function IDENT ( IDENT : IDENT ) REGEXP . IDENT ( IDENT ) ;",
        ),
    ] {
        stream(code, ScriptTS, want, cpu);
    }
    let ks = kinds_of("declare function y()\n<div/>;", ScriptTSX, cpu);
    assert!(ks.contains(&TokenKind::JsxLt), "{ks:?}");
    division("x = f()\n/ 2 / 3;", ScriptTS, cpu);
    division("f<T>()\n/ 2 / 3;", ScriptTS, cpu);
    division("new Foo()\n/ 2 / 3;", ScriptTS, cpu);
    division("x = function(){}\n/ 2 / 3;", ScriptTS, cpu);
}

#[test_cpus]
fn operand_heads_before_a_value_brace(cpu: DynCpu) {
    for (code, want) in [
        (
            "x = <a b={ {k: v} / 2 }/>;",
            "IDENT = JSX_LT IDENT IDENT = { { IDENT : IDENT } / NUMBER } / JSX_TAG_END ;",
        ),
        (
            "x = <a>{ {k: v} / 2 }</a>;",
            "IDENT = JSX_LT IDENT > { { IDENT : IDENT } / NUMBER } JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "x = <a>{c}{ {k: v} / 2 }</a>;",
            "IDENT = JSX_LT IDENT > { IDENT } { { IDENT : IDENT } / NUMBER } JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "x = {}\n{ {k: v}\n/re/.test(s) }",
            "IDENT = { } { { IDENT : IDENT } REGEXP . IDENT ( IDENT ) }",
        ),
        ("{ {k: v}\n/re/.test(s) }", "{ { IDENT : IDENT } REGEXP . IDENT ( IDENT ) }"),
    ] {
        stream(code, ScriptJSX, want, cpu);
    }
    division("f(...{a: 1} / 2);", ScriptJS, cpu);
    division("x = [...function(){} / 2];", ScriptJS, cpu);
    division("f(...class {} / 2);", ScriptJS, cpu);
    division("for (const k of {a: 1} / 2) ;", ScriptJS, cpu);
    division("for (x of {} / 2) ;", ScriptJS, cpu);
    division("x = a, void {} / 2;", ScriptJS, cpu);
    division("f(a, void {} / 2);", ScriptJS, cpu);
}

#[test_cpus]
fn unicode_whitespace_before_a_glued_number(cpu: DynCpu) {
    division("{\u{2028}5. // c\n/\u{a0}b / c; }", ScriptJS, cpu);
    division("x = \u{a0}5.\n/ 2 / 3;", ScriptJS, cpu);
    division("x = \u{2028}5\n/ 2 / 3;", ScriptJS, cpu);
}

#[test_cpus]
fn function_head_walks_cross_every_return_type_shape(cpu: DynCpu) {
    division(
        "const P = function a(x: A): abstract new () => abstract new () => keyof any {}\n/ 2 / 3;",
        ScriptTS,
        cpu,
    );
    division("x = [function (): _ is Record<Promise<V>> {}\n/ 2 / 3];", ScriptTS, cpu);
    regex(
        "function y(): x is abstract new () => typeof import('m') extends boolean ? K<A, B<unique symbol>> : void[] {}\n/[a-z]+/v.x;",
        ScriptTS,
        cpu,
    );
    regex("async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;", ScriptTS, cpu);
    regex("function f(): { a: T } {}\n/re/.test(x);", ScriptTS, cpu);
    regex(
        "async function obj(): K<U, $JSX<Array<U>>> {\nclass C implements V, E<$JSX<-1>> {\n\n}\n/[\\]]/.test(s);\n}",
        ScriptTS,
        cpu,
    );
    let ks = kinds_of(
        "x = [async function $(): _ is Record<Promise<V>>{} < function Bar(arr?: any): K<this>{}];",
        ScriptTSX,
        cpu,
    );
    assert!(ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::JsxLt), "{ks:?}");
}

#[test_cpus]
fn brace_after_a_generic_return_type_is_a_body(cpu: DynCpu) {
    for (code, file_type) in [
        ("async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;", ScriptTS),
        ("async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;", ScriptTSX),
        ("function f(): () => Array<symbol> {}\n/re/.x;", ScriptTS),
        ("function f(): A | B<C> {}\n/re/.x;", ScriptTS),
        ("x = function* (): P<Q<R>> {}\nwhile (a) { async (p): B => 1\n}\n/re/.x;", ScriptTS),
    ] {
        regex(code, file_type, cpu);
    }
    division("x = f < T > {} / 2;", ScriptTS, cpu);
    division("x = c ? function* (...a): Pick<E<F<G>>>[][] {} / 2 : null;", ScriptTS, cpu);
}

#[test_cpus]
fn ts_postfix_bang_before_a_block_brace(cpu: DynCpu) {
    regex("[typeof ab]!\n{\n}\n/x/.test(s);", ScriptTS, cpu);
    regex("x!\n{}\n/x/.test(s);", ScriptTS, cpu);
    regex("f(x)!\n{}\n/x/.test(s);", ScriptTS, cpu);
    division("x = !{} / 2;", ScriptTS, cpu);
    division("x = a && !{}.b / 2;", ScriptTS, cpu);
    division("x!\n/re/g.exec(s);", ScriptTS, cpu);
}

#[test_cpus]
fn adjacent_type_atoms_end_an_annotation(cpu: DynCpu) {
    division("let x: Foo<T>\nf(y)\n/re/.test(s);", ScriptTS, cpu);
    division("let x: T\nf(y)\n/re/.test(s);", ScriptTS, cpu);
    division("let x: T[]\nf(y)\n/re/.test(s);", ScriptTS, cpu);
    division("let x: `lit`\nf(y)\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: typeof import('m').default\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: asserts y is T\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: abstract new () => T\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: readonly unique symbol[]\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: A extends infer U extends B ? C : D\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: `a${T}b${U}c`\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: { [K in keyof T as `x${K}`]: T[K] }\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: <T>(a: T) => T\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: new <T>(a: T) => T\n/re/.test(s);", ScriptTS, cpu);
    regex("let x: <T>(a: T) => T | undefined\n/re/.test(s);", ScriptTS, cpu);
    regex("let a: A, b: B, c: <T>(a: T) => T\n/re/.test(s);", ScriptTS, cpu);
    regex("declare function f<T>(a: T): <U>(b: U) => T\n/re/.test(s);", ScriptTS, cpu);
    regex("type A<T> = <T>(a: T) => T\n/re/.test(s);", ScriptTS, cpu);
    for code in [
        "declare let x: <T>(a: T) => T\n<div />;",
        "let x: <T>(a: T) => T | undefined\n<div />;",
        "let a: A, b: B, c: <T>(a: T) => T\n<div />;",
        "type A = <T>(a: T) => T\n<div />;",
        "declare function f(): <T>(a: T) => T\n<div />;",
    ] {
        let ks = kinds_of(code, ScriptTSX, cpu);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    }
    let ks = kinds_of("let P: Array<bigint, this>\n_(x)\n<(P().foo);", ScriptTSX, cpu);
    assert!(ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::JsxLt), "{ks:?}");
    let codes = diag_codes_of("let P: Array<bigint, this>\n_(x)\n<(P().foo);", ScriptTSX, cpu);
    assert!(codes.is_empty(), "{codes:?}");
}

#[test_cpus]
fn decorated_export_class_declaration(cpu: DynCpu) {
    // A decorator between `export` (or `export default`) and `class` decorates a declaration.
    regex("export @dec class C {} /re/.test(x);", ModuleJS, cpu);
    regex("export @dec() class C {} /re/.test(x);", ModuleJS, cpu);
    regex("export @a @b class C {} /re/.test(x);", ModuleJS, cpu);
    regex("export default @dec class {} /re/.test(x);", ModuleJS, cpu);
    regex("export default @dec class C {} /re/.test(x);", ModuleJS, cpu);
    division("export @dec class C {} x = {} / 2;", ModuleJS, cpu);
    division("export default (@dec class {}) / 2;", ModuleJS, cpu);
}

#[test_cpus]
fn ts_satisfies_after_a_type_operator(cpu: DynCpu) {
    // `satisfies` ends the type of a preceding `as` / `satisfies` and opens its own.
    division("let v = x as T satisfies {} / y;", ScriptTS, cpu);
    division("let v = x as {} satisfies {} / y;", ScriptTS, cpu);
    division("let v = x satisfies {} satisfies {} / y;", ScriptTS, cpu);
    division("let v = {a: 1} as const satisfies {a: 1} / y;", ScriptTS, cpu);
    division("let v = x as A<B> satisfies {} / y;", ScriptTS, cpu);
    regex("x = y as T satisfies {}\n{} /re/.test(s);", ScriptTS, cpu);
}

#[test_cpus]
fn function_name_after_a_line_break(cpu: DynCpu) {
    // Nothing restricts a line break between `function` and its name: the head goes on.
    division("x = function\nf() {} / 2;", ScriptJS, cpu);
    regex("function\nf(): T {} /re/.test(x);", ScriptTS, cpu);
    regex("function /* c */\nf() {} /re/.test(x);", ScriptJS, cpu);
    regex("async function\nf() {} /re/.test(x);", ScriptJS, cpu);
    regex("function\n*g() {} /re/.test(x);", ScriptJS, cpu);
}

#[test_cpus]
fn class_field_initializer_is_outside_yield_and_await_contexts(cpu: DynCpu) {
    // A field initializer is parsed outside the enclosing function's `yield` / `await` context,
    // so both are identifiers in it; computed keys and static blocks still see the function.
    division("async function f() { class A { x = await / 2 / 1 } }", ScriptJS, cpu);
    division("async function f() { class A { static x = await / 2 / 1 } }", ScriptJS, cpu);
    division("async function f() { class A { x = (a = await / 2 / 1) => a } }", ScriptJS, cpu);
    division("async function f() { class A { x = await / 2 / 1; y = 3 } }", ScriptJS, cpu);
    regex("async function f() { class A { [await /re/.test(x)] = 1 } }", ScriptJS, cpu);
    regex("async function f() { class A { x = 1; [await /re/.test(x)] = 1 } }", ScriptJS, cpu);
    regex("class A { static { await /re/.test(x) } }", ScriptJS, cpu);
    regex("async function f() { class A { async m() { await /re/.test(x) } } }", ScriptJS, cpu);
}

#[test_cpus]
fn of_ending_a_declaration_before_a_line_break(cpu: DynCpu) {
    // ASI ends the declaration or the break, so a regex starts the next statement.
    regex("let of\n/re/g.test(s)", ScriptJS, cpu);
    regex("var of\n/re/g.test(s)", ScriptJS, cpu);
    regex("of: for (;;) { break of\n/re/g.test(s) }", ScriptJS, cpu);
}

#[test_cpus]
fn async_call_arguments_keep_the_enclosing_await_context(cpu: DynCpu) {
    // A call to a function named async: its class argument is outside any await context.
    division("async (class { [await / 2 / 1]() {} });", ScriptJS, cpu);
}

#[test_cpus]
fn spread_class_expression_with_a_heritage_list(cpu: DynCpu) {
    // A spread is not member access, so the heritage comma belongs to the class head.
    division("x = [...class implements A, B {} / 2];", ScriptTS, cpu);
}

#[test_cpus]
fn declaration_after_a_name_or_type_ended_by_a_line_break(cpu: DynCpu) {
    // The word or postfix operator before the line break ends a value, so ASI starts a declaration.
    regex("let of\nfunction f() {}\n/re/g", ScriptJS, cpu);
    regex("var await\nfunction f() {}\n/re/g", ScriptJS, cpu);
    regex("declare function f(): void\nfunction f() {}\n/re/g", ScriptTS, cpu);
    regex("let x: void\nclass K {}\n/re/g", ScriptTS, cpu);
    regex("let x = y!\nfunction f() {}\n/re/g", ScriptTS, cpu);
}

#[test_cpus]
fn only_an_export_clause_opens_module_braces(cpu: DynCpu) {
    // Braces after export are an export clause only when they follow it directly.
    division("export = { a: 1 }\n/re/g", ScriptTS, cpu);
    division("export const x = { a: 1 } / 2;", ModuleJS, cpu);
    regex("export declare global {}\nfunction f() {}\n/re/g", ScriptTS, cpu);
}

#[test_cpus]
fn export_async_function_is_a_declaration(cpu: DynCpu) {
    // The full walk sees async between export and function: still a declaration.
    regex("export async function f() {}\n/re/g", ModuleJS, cpu);
}

#[test_cpus]
fn interface_members_on_separate_lines(cpu: DynCpu) {
    // The full walk steps through the body: a line between members does not end the interface.
    regex("interface I {\n  x: A<B<C>>\n  y: D\n}\n/re/g", ScriptTS, cpu);
}

#[test_cpus]
fn yield_and_await_inside_the_function_a_bounded_walk_starts_in(cpu: DynCpu) {
    division("function* g() { const a = 1; yield {} / 2; }", ScriptJS, cpu);
    division("function* g() { const a = 1; yield {} / 2; }", ModuleJS, cpu);
    division("async function f() { const a = 1; await {} / 2; }", ScriptJS, cpu);
    division("({ *function() { yield {} / 2 } });", ScriptJS, cpu);
    division("class C { *function() { yield {} / 2 } }", ScriptJS, cpu);
}

#[test_cpus]
fn expression_after_an_as_type(cpu: DynCpu) {
    regex("y = x as T || (() => { if (a) {} /re/ });", ScriptTS, cpu);
    regex("y = x as T ? a : function () { if (c) {} /re/ };", ScriptTS, cpu);
}

#[test_cpus]
fn from_as_a_name_before_a_string(cpu: DynCpu) {
    division("let from = 1;\nfrom\n\"y\"\n/2/1;", ScriptJS, cpu);
    division("let from = 1;\nfrom\n\"y\"\n/2/1;", ScriptTS, cpu);
}

#[test_cpus]
fn dots_before_a_slash_or_less_than(cpu: DynCpu) {
    regex("[.../re/];", ScriptJS, cpu);
    regex("f(... /*c*/ /re/);", ScriptJS, cpu);
    division("x = 1./2;", ScriptJS, cpu);
    let jsx_after_spread = kinds_of("[...<a/>];", ScriptTSX, cpu);
    assert!(jsx_after_spread.contains(&TokenKind::JsxLt), "{jsx_after_spread:?}");
    // No name after a member dot: an operator goes on, as in tsc.
    division("a./b/2;", ScriptJS, cpu);
    division("a?./b/2;", ScriptJS, cpu);
}

#[test_cpus]
fn keyword_after_a_number_with_a_trailing_dot(cpu: DynCpu) {
    regex("x = 1. in /re/;", ScriptJS, cpu);
    regex("x = 1. instanceof /re/.constructor;", ScriptJS, cpu);
}

#[test_cpus]
fn statement_after_import_attributes(cpu: DynCpu) {
    division("import a from \"b\" with { type: \"json\" }\nlet x = a\n/re/g;", ModuleJS, cpu);
    regex("import a from \"b\" with { type: \"json\" }\nlet x\n/re/g;", ModuleJS, cpu);
    regex(
        "export { a } from \"b\" with { type: \"json\" }\nfunction f() {}\n/re/g;",
        ModuleJS,
        cpu,
    );
}

#[test_cpus]
fn of_after_any_value_in_a_for_head(cpu: DynCpu) {
    regex("for (x! of /re/g.exec(s)) {}", ScriptTS, cpu);
    regex("for (var x: T of /re/g.exec(s)) {}", ScriptTS, cpu);
    regex("for (let x = 1 of /re/g.exec(s)) {}", ScriptTS, cpu);
    division("for (x = of / 2;;) {}", ScriptJS, cpu);
}

#[test_cpus]
fn walk_past_a_function_expression(cpu: DynCpu) {
    division("x = { f: function() {}, g: {} / 2 };", ScriptJS, cpu);
    division("x = { f: class {}, g: {} / 2 };", ScriptJS, cpu);
    division("x = { f: function() {}, g: [{} / 2] };", ScriptJS, cpu);
}

#[test_cpus]
fn not_equal_at_a_line_start_continues_the_expression(cpu: DynCpu) {
    division("x = c ? a\n!= b : {} / 2;", ScriptJS, cpu);
    regex("let y = a\n!== b, z\n/re/g", ScriptJS, cpu);
    regex("f = async x => x\n!== null && await /re/g", ScriptJS, cpu);
}

#[test_cpus]
fn as_at_a_line_start_is_a_name(cpu: DynCpu) {
    // tsc ends the expression before an as or satisfies on a new line.
    regex("let as;\nx = y\nas\n{}\n/re/g", ScriptTS, cpu);
    regex("let satisfies;\nx = y\nsatisfies\n{}\n/re/g", ScriptTS, cpu);
}

#[test_cpus]
fn declaration_goes_on_after_a_type_that_ends_a_line(cpu: DynCpu) {
    regex("let a: T\n= 1, b\n/re/g", ScriptTS, cpu);
    division("x = function (): T\n{} / 2;", ScriptTS, cpu);
    regex("class A { async m(): T\n{ await /re/g } }", ScriptTS, cpu);
    regex("import x = require(\"y\")\n/re/g", ScriptTS, cpu);
}

#[test_cpus]
fn as_type_ends_at_an_expression_operator(cpu: DynCpu) {
    regex("x = a as T - void /re/g;", ScriptTS, cpu);
    regex("x = c ? a as T : void /re/g;", ScriptTS, cpu);
    regex("x = a satisfies T * void /re/g;", ScriptTS, cpu);
    regex("x = a as T in void /re/g;", ScriptTS, cpu);
    regex("async function f() { x = a as T == b && await /re/g }", ScriptTS, cpu);
    regex("async function f() { x = a as T <= b\nawait /re/g }", ScriptTS, cpu);
    regex("async function f() { x = a as T[] < b\nawait /re/g }", ScriptTS, cpu);
    division("x = c ? a as T : function () {} / 2;", ScriptTS, cpu);
}
