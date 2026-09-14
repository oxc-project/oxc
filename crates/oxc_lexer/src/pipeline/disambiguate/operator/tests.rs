use crate::token::TokenKind;

use super::super::tests::{FileType, diag_codes_of, division, kinds_of, regex, stream};

#[test]
fn debugger_precedes_regex() {
    regex("debugger\n/re/.test(x);", FileType::ScriptJS);
    division("x.debugger / 2;", FileType::ScriptJS);
}

#[test]
fn for_of_head_precedes_regex() {
    regex("for (x of /re/) ;", FileType::ScriptJS);
    regex("for ([a, b] of /re/) ;", FileType::ScriptJS);
    regex("for ({a} of /re/) ;", FileType::ScriptJS);
    regex("for (const x of /re/) ;", FileType::ScriptJS);
    regex("for await (x of /re/) ;", FileType::ScriptJS);
    regex("for ([a, of] of /re/) ;", FileType::ScriptJS);
    division("var of = 1; of / 2;", FileType::ScriptJS);
    division("instance/of/g;", FileType::ScriptJS);
    division("for (of / 2;;) ;", FileType::ScriptJS);
    division("for (of of of / 2) ;", FileType::ScriptJS);
    division("f(x, of / 2);", FileType::ScriptJS);
}

#[test]
fn unicode_ident_postfix_is_division() {
    division("\u{53d8}\u{91cf}++ / b;", FileType::ScriptJS);
    division("\u{53d8}\u{91cf} ++ / b;", FileType::ScriptJS);
    regex("a\u{2028}++/re/.lastIndex;", FileType::ScriptJS);
    regex("a\u{2029}++/re/.lastIndex;", FileType::ScriptJS);
    division("a\u{00a0}++ / b;", FileType::ScriptJS);
    division("a\u{200a}++ / b;", FileType::ScriptJS);
    division("a\u{feff}++ / b;", FileType::ScriptJS);
    division("a\u{200d}b++ / 2;", FileType::ScriptJS);
}

#[test]
fn operand_keywords_before_value_braces() {
    division("x = typeof {} / 2;", FileType::ScriptJS);
    division("x = void {} / 2;", FileType::ScriptJS);
    division("f(new class {} / 2);", FileType::ScriptJS);
    division("return function(){} / 2;", FileType::ScriptJS);
    division("throw {} / 2;", FileType::ScriptJS);
    division("if (k in {} / 2) ;", FileType::ScriptJS);
    division("switch (x) { case class {} / 2: break; }", FileType::ScriptJS);
    regex("return\nclass C {} /re/.test(x);", FileType::ScriptJS);
    regex("export default class C {} /re/.test(x);", FileType::ScriptJS);
}

#[test]
fn object_literal_heritage() {
    division("(class C extends {valueOf(){}} {} / 2);", FileType::ScriptJS);
    division("(class extends {a:1}.constructor {} / 2);", FileType::ScriptJS);
    regex("x = class {}\n{} /re/.test(s);", FileType::ScriptJS);
    regex("x = class C extends {a:1} {}\n{} /re/.test(s);", FileType::ScriptJS);
}

#[test]
fn ts_implements_heritage() {
    division("(class C implements I, J {} / 2);", FileType::ScriptTS);
    division("(class C extends B implements I, J {} / 2);", FileType::ScriptTS);
    regex("class C implements I, J {} /re/.test(x);", FileType::ScriptTS);
    regex("x = class A {}, y\n{} /re/.test(s);", FileType::ScriptTS);
}

#[test]
fn decorated_class_expression() {
    division("x = @dec class {} / 2;", FileType::ScriptTS);
    division("x = @ns.dec() class {} / 2;", FileType::ScriptTS);
    division("f(@a @b(1) class {} / 2);", FileType::ScriptTS);
    regex("@dec class C {} /re/.test(x);", FileType::ScriptTS);
}

#[test]
fn ts_angle_before_brace() {
    division("(class C<T> {} / 2);", FileType::ScriptTS);
    division("(class C extends B<T> {} / 2);", FileType::ScriptTS);
    division("(class C<T extends {a: 1}> {} / 2);", FileType::ScriptTS);
    division("(class C<T = X> {} / 2);", FileType::ScriptTS);
    division("x = f < T > {} / re / g;", FileType::ScriptTS);
    division("x = a < b + c > {} / 2;", FileType::ScriptTS);
    division("(a > {} / 2);", FileType::ScriptTS);
    regex("class C<T> {} /re/.test(x);", FileType::ScriptTS);
    regex("interface I<T> {} /re/.test(x);", FileType::ScriptTS);
    regex("declare class C<T> {} /re/.test(x);", FileType::ScriptTS);
    regex("class C extends B<T> {}\n/re/.test(x);", FileType::ScriptTS);
}

#[test]
fn brace_tail_before_as() {
    division("let v = {a: 1} as T / y;", FileType::ScriptTS);
    division("let v = {} as A<B> / y;", FileType::ScriptTS);
    division("let v = {} satisfies T / y;", FileType::ScriptTS);
    division("let v = ({} as T) / y;", FileType::ScriptTS);
}

#[test]
fn template_literal_types_cross() {
    division("(class C<T extends `a${X}`> {} / 2);", FileType::ScriptTS);
    division("(class C<T extends `a${`b${Y}`}`> {} / 2);", FileType::ScriptTS);
    division("let v = x as A<`a${B}`> / y;", FileType::ScriptTS);
    regex("class C<T extends `a${X}`> {} /re/.test(x);", FileType::ScriptTS);
}

#[test]
fn hidden_trivia_segments() {
    division("a\u{00a0}++ / b;", FileType::ScriptJS);
    division("a\u{200a}++ / b;", FileType::ScriptJS);
    division("a\u{feff}++ / b;", FileType::ScriptJS);
    division("x = a \u{00a0}++ / b;", FileType::ScriptJS);
    regex("return\u{00a0}++/re/.lastIndex;", FileType::ScriptJS);
    division("x.\u{00a0}return++ / 2;", FileType::ScriptJS);
    regex("a\u{2028}++/re/.lastIndex;", FileType::ScriptJS);
    regex("f(a,\u{00a0}++/re/.lastIndex);", FileType::ScriptJS);
}

#[test]
fn ts_as_type_brace() {
    division("let v = x as {} / y;", FileType::ScriptTS);
    division("let v = x as {a: 1} / y;", FileType::ScriptTS);
    division("let v = f() as {} / y;", FileType::ScriptTS);
    division("let v = x satisfies {} / y;", FileType::ScriptTS);
    regex("f();\nas: {} /re/.test(s);", FileType::ScriptTS);
}

#[test]
fn ts_as_angle_form() {
    division("let v = x as A<B> / y;", FileType::ScriptTS);
    division("let v = x as a.b.C<D> / y;", FileType::ScriptTS);
    division("let v = f() as A<B<C>> / y;", FileType::ScriptTS);
    division("let v = x satisfies A<B> / y;", FileType::ScriptTS);
    regex("let q = a > /re/.source.length;", FileType::ScriptTS);
    regex("let q = (x < y) > /re/.source;", FileType::ScriptTS);
    regex("g(x => /re/.test(x));", FileType::ScriptTS);
}

#[test]
fn ts_return_type_function_expr() {
    division("x = function(): T {} / 2;", FileType::ScriptTS);
    division("x = function(): a.b.T {} / 2;", FileType::ScriptTS);
    division("x = function(): Map<A, B> {} / 2;", FileType::ScriptTS);
    division("x = async function(): T {} / 2;", FileType::ScriptTS);
    regex("function f(): T {} /re/.test(x);", FileType::ScriptTS);
    regex("function f(): Map<A, B> {} /re/.test(x);", FileType::ScriptTS);
    regex("L: {} /re/.test(x);", FileType::ScriptTS);
    regex("switch (x) { case f(y): {} /re/.test(s); }", FileType::ScriptTS);
}

#[test]
fn async_function_expression_value() {
    division("x = async function(){} / 2;", FileType::ScriptJS);
    regex("async function f(){} /re/.test(x);", FileType::ScriptJS);
}

#[test]
fn slash_dense_chains() {
    let count = |code: &str, want_re: usize, want_slash: usize| {
        let ks = kinds_of(code, FileType::ScriptJS);
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

#[test]
fn colon_member_and_ternary_values() {
    division("({a: function(){} / 2, b: 1});", FileType::ScriptJS);
    division("({a: {} / 2});", FileType::ScriptJS);
    division("({a: {b: {} / 2}});", FileType::ScriptJS);
    division("({a: class {} / 2});", FileType::ScriptJS);
    division("x = c ? y : {} / 2;", FileType::ScriptJS);
    division("x = c ? d ? a : b : {} / 2;", FileType::ScriptJS);
    regex("L: {} /re/.test(x);", FileType::ScriptJS);
    regex("{ L: function f(){} /re/ }", FileType::ScriptJS);
    regex("switch (x) { case a: function f(){} /re/ }", FileType::ScriptJS);
    regex("switch (x) { case f(y), z: {} /re/.test(s); }", FileType::ScriptJS);
    regex("c ? a : b\nL: {} /re/.test(s);", FileType::ScriptJS);
    regex("if (c) {} L: {} /re/.test(s);", FileType::ScriptJS);
}

#[test]
fn postfix_incdec_then_slash_is_division() {
    division("a++ / b;", FileType::ScriptJS);
    division("a-- / b;", FileType::ScriptJS);
    division("a ++ / b;", FileType::ScriptJS);
    division("x[i]++ / n;", FileType::ScriptJS);
    division("f(x)++ / n;", FileType::ScriptJS);
    division("a.return++ / 2;", FileType::ScriptJS);
    division("obj.#f++ / 2;", FileType::ScriptJS);
    division("a = b++/c/g;", FileType::ScriptJS);
    division("a++\n/ b / c;", FileType::ScriptJS);
}

#[test]
fn prefix_incdec_then_slash_is_regex() {
    regex("++/re/.lastIndex;", FileType::ScriptJS);
    regex("x = ++/re/.lastIndex;", FileType::ScriptJS);
    regex("(a, ++/re/.lastIndex);", FileType::ScriptJS);
    regex("f(++/re/.lastIndex);", FileType::ScriptJS);
    regex("return++/re/.lastIndex;", FileType::ScriptJS);
    regex("a + ++/re/.lastIndex;", FileType::ScriptJS);
    regex("a ** ++/re/.lastIndex;", FileType::ScriptJS);
}

#[test]
fn line_terminator_forces_prefix() {
    regex("a\n++/re/.lastIndex;", FileType::ScriptJS);
    regex("a\r\n++/re/.lastIndex;", FileType::ScriptJS);
    regex("a\u{2028}++/re/.lastIndex;", FileType::ScriptJS);
    regex("a\u{2029}++/re/.lastIndex;", FileType::ScriptJS);
    regex("a /* x\ny */ ++/re/.lastIndex;", FileType::ScriptJS);
    division("a /* xy */ ++ / b;", FileType::ScriptJS);
}

#[test]
fn incdec_runs_keep_maximal_munch() {
    regex("a+++/re/;", FileType::ScriptJS);
    regex("a---/re/;", FileType::ScriptJS);
    division("a++/b/;", FileType::ScriptJS);
}

#[test]
fn default_and_extends_precede_regex() {
    regex("export default /^x$/;", FileType::ScriptJS);
    regex("export default /re/.source;", FileType::ScriptJS);
    regex("class C extends /re/.constructor {}", FileType::ScriptJS);
    division("x.default / 2;", FileType::ScriptJS);
    division("x.extends / 2;", FileType::ScriptJS);
}

#[test]
fn statement_head_paren_then_regex() {
    regex("if (x) /re/.test(y);", FileType::ScriptJS);
    regex("if (f(x)) /re/.test(y);", FileType::ScriptJS);
    regex("while (x) /re/.exec(y);", FileType::ScriptJS);
    regex("for (;;) /re/.test(x);", FileType::ScriptJS);
    regex("with (o) /re/.test(x);", FileType::ScriptJS);
    regex("for await (x of y) /re/.test(x);", FileType::ScriptJS);
    regex("do x; while (y) /re/.test(z);", FileType::ScriptJS);
    regex("if (a) while (b) /re/.test(c);", FileType::ScriptJS);
}

#[test]
fn value_paren_then_slash_stays_division() {
    division("f(x) / 2;", FileType::ScriptJS);
    division("(a + b) / 2;", FileType::ScriptJS);
    division("x.if(a) / 2;", FileType::ScriptJS);
    division("x?.while(a) / 2;", FileType::ScriptJS);
    division("if (a) (b) / c / d;", FileType::ScriptJS);
    division("await (x) / 2;", FileType::ScriptJS);
}

#[test]
fn class_expression_brace_then_slash_is_division() {
    division("(class {} / 2);", FileType::ScriptJS);
    division("(class C {} / 2);", FileType::ScriptJS);
    division("(class extends B {} / 2);", FileType::ScriptJS);
    division("(class C extends B {} / 2);", FileType::ScriptJS);
    division("(class C extends f(B) {} / 2);", FileType::ScriptJS);
    division("(class C extends a.b[0] {} / 2);", FileType::ScriptJS);
    division("x = class {} / 2;", FileType::ScriptJS);
    division("f(class {m(){}} / 2);", FileType::ScriptJS);
    division("`${class {} / 2}`;", FileType::ScriptJS);
}

#[test]
fn class_declaration_brace_then_slash_is_regex() {
    regex("class C {} /re/.test(x);", FileType::ScriptJS);
    regex("class C extends B {}\n/re/.test(x);", FileType::ScriptJS);
    regex("{ class C {} } /re/.test(x);", FileType::ScriptJS);
}

#[test]
fn block_braces_keep_the_regex_answer() {
    regex("{} /re/.test(x);", FileType::ScriptJS);
    regex(";{} /re/.test(x);", FileType::ScriptJS);
    regex("L: {} /re/.test(x);", FileType::ScriptJS);
    regex("if(a){}else{} /'/.test(s);\nconst t='x';", FileType::ScriptJS);
    regex("x = () => {}\n/re/.test(s);", FileType::ScriptJS);
}

#[test]
fn value_braces_keep_the_division_answer() {
    division("f({} / 2);", FileType::ScriptJS);
    division("x = function(){} / 2;", FileType::ScriptJS);
}

#[test]
fn plain_contexts_unchanged() {
    division("a / b;", FileType::ScriptJS);
    division("1n / 2;", FileType::ScriptJS);
    regex("a + /re/g;", FileType::ScriptJS);
    regex("x = /re/;", FileType::ScriptJS);
    regex("f(/re/);", FileType::ScriptJS);
}

#[test]
fn ts_postfix_bang_unchanged() {
    let ks = kinds_of("x! / 2;", FileType::ScriptTS);
    assert!(!ks.contains(&TokenKind::RegExp), "x! / 2 must stay division: {ks:?}");
}

#[test]
fn bare_gt_object_rhs_is_division() {
    division("x = f < T > {} / re / g;", FileType::ScriptJS);
    division("x = a > {} / 2;", FileType::ScriptJS);
    division("x = a >> {} / 2;", FileType::ScriptJS);
    division("x = a >>> {} / 2;", FileType::ScriptJS);
    division("x = a-- > {} / 2;", FileType::ScriptJS);
    division("x = a >\n{} / 2;", FileType::ScriptJS);
}

#[test]
fn arrow_block_bodies_still_regex() {
    regex("x = y => {}\n/re/.test(s);", FileType::ScriptJS);
    regex("x = async () => {}\n/re/.test(s);", FileType::ScriptJS);
}

#[test]
fn ts_angle_close_resolved() {
    let ks = kinds_of("class C<T> {} /re/.test(x);", FileType::ScriptTS);
    assert!(ks.contains(&TokenKind::RegExp), "TS class decl with type params: {ks:?}");
    let ks = kinds_of("x = f < T > {} / re / g;", FileType::ScriptTS);
    assert!(!ks.contains(&TokenKind::RegExp), "TS relational re-read must divide: {ks:?}");
}

#[test]
fn unicode_ident_tail_resolved() {
    division("\u{53d8}\u{91cf}++ / b;", FileType::ScriptJS);
    regex("a\u{2028}++/re/.lastIndex;", FileType::ScriptJS);
}

#[test]
fn of_trade_resolved() {
    regex("for (x of /re/) ;", FileType::ScriptJS);
    division("var of = 1; of / 2;", FileType::ScriptJS);
    division("instance/of/g;", FileType::ScriptJS);
}

#[test]
fn jsx_operand_positions() {
    let jsx = |code: &str| kinds_of(code, FileType::ScriptJSX);
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

#[test]
fn ts_declarator_type_annotation_forces_asi() {
    regex("let x: T\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: string\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: number\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: A | B\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: T[]\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: (T)\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: typeof y\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: a.b.C\n/re/.exec(s);", FileType::ScriptTS);
    regex("var x: T\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: T\n/re/gimsuy.exec(s);", FileType::ScriptTS);
    regex("let a = 1, b: T\n/re/.exec(s);", FileType::ScriptTS);
}

#[test]
fn ts_declarator_type_annotation_already_resolved() {
    regex("let x: Array<T>\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: {a: T}\n/re/.exec(s);", FileType::ScriptTS);
    regex("interface I { a: T }\n/re/.exec(s);", FileType::ScriptTS);
}

#[test]
fn ts_initialised_declarator_stays_division() {
    division("let x= T\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: number = 1\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: T = y\n/re/g.exec(s);", FileType::ScriptTS);
    division("let a = b\n/hi/g.exec(c);", FileType::ScriptJS);
    division("let a = b, c = d\n/hi/g.exec(e);", FileType::ScriptJS);
}

#[test]
fn ts_type_alias_forces_asi() {
    regex("type A = T\n/re/.exec(s);", FileType::ScriptTS);
    regex("type A = B | C\n/re/.exec(s);", FileType::ScriptTS);
    regex("declare function f(): T\n/re/.exec(s);", FileType::ScriptTS);
}

#[test]
fn declarator_without_initializer_forces_asi() {
    regex("let x\n/re/.exec(s);", FileType::ScriptJS);
    regex("var a\n/re/.test(b);", FileType::ScriptJS);
    regex("let x\n/re/.exec(s);", FileType::ScriptTS);
    regex("let a, b\n/re/.test(c);", FileType::ScriptJS);
    regex("let a = 1, b\n/re/.test(c);", FileType::ScriptJS);
}

#[test]
fn module_specifier_forces_asi() {
    regex("import y from 'y'\n/re/.exec(s);", FileType::ScriptJS);
    regex("import 'y'\n/re/.exec(s);", FileType::ScriptJS);
    regex("export * from 'y'\n/re/.exec(s);", FileType::ScriptJS);
    regex("export {a} from 'y'\n/re/.exec(s);", FileType::ScriptJS);
    division("x = 'y' / 2;", FileType::ScriptJS);
    division("x = f('y') / 2;", FileType::ScriptJS);
}

#[test]
fn restricted_production_keywords_precede_regex() {
    regex("for(;;){ break\n/re/.test(b); }", FileType::ScriptJS);
    regex("for(;;){ continue\n/re/.test(b); }", FileType::ScriptJS);
    division("x.break / 2;", FileType::ScriptJS);
    division("x.continue / 2;", FileType::ScriptJS);
}

#[test]
fn ts_return_type_keyword_name_is_not_operand() {
    regex("function f(): void {}\n/re/.test(x);", FileType::ScriptTS);
    regex("function f(): void {} /re/.test(x);", FileType::ScriptTS);
    division("x = function(): void {} / 2;", FileType::ScriptTS);
    division("x = void {} / 2;", FileType::ScriptJS);
    division("x = typeof {} / 2;", FileType::ScriptJS);
}

#[test]
fn ts_composite_type_forms_force_asi() {
    regex("let x: A extends B ? C : D\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: (a: T) => U\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: 'lit'\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: [A, B]\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: readonly A[]\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: keyof T\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: Map<A, B<C>>\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: T | null\n/re/.exec(s);", FileType::ScriptTS);
    regex("declare const x: T\n/re/.exec(s);", FileType::ScriptTS);
    regex("export const x: T\n/re/.exec(s);", FileType::ScriptTS);
}

#[test]
fn ts_pathological_type_annotation_forces_asi() {
    regex(
        r#"let x: {
  readonly [K in keyof T as `get${Capitalize<K & string>}`]-?:
    T[K] extends infer U extends (...a: [x: string, ...r: number[]]) => infer R
      ? new (m: typeof import("./m").default) => U extends { a: infer V } ? V : R
      : { [k: string]: readonly string[] }
} & (abstract new () => void) & T
/re/g.exec(s)"#,
        FileType::ScriptTS,
    );
    regex(
        "let x: {[K in keyof T as `g${K & string}`]-?: T[K]}\n/re/g.exec(s);",
        FileType::ScriptTS,
    );
    regex("let x: T extends infer U extends F ? A : B\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: typeof import(\"./m\").default\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: abstract new () => void\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: (...a: [x: string, ...r: number[]]) => R\n/re/g.exec(s);", FileType::ScriptTS);
    division(
        r#"let x = y as {
  readonly [K in keyof T as `get${Capitalize<K & string>}`]-?: T[K]
} & (abstract new () => void) & T
/re/g.exec(s)"#,
        FileType::ScriptTS,
    );
}

#[test]
fn ts_type_literal_brace_forces_asi() {
    regex("type A = { a: T }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("type A = {}\n/re/g.exec(s);", FileType::ScriptTS);
    regex("type A<T> = { a: T }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("type A = { [K in keyof T]: T[K] }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("type A = { (): T }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: A | { a: T }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: A & { a: T }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: A extends B ? C : { a: T }\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: A extends B ? { a: T } : C\n/re/g.exec(s);", FileType::ScriptTS);
    regex("declare function f(): A | { a: T }\n/re/g.exec(s);", FileType::ScriptTS);
    division("x = {a: 1}\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = {a: 1}\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: T = {a: 1}\n/re/g.exec(s);", FileType::ScriptTS);
    division("x = class {}\n/re/g.exec(s);", FileType::ScriptTS);
    division("let v = {} as T\n/re/g.exec(s);", FileType::ScriptTS);
    division("f({} / 2);", FileType::ScriptTS);
}

#[test]
fn ts_as_void_is_division() {
    division("let x = y as void\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y as void / 2;", FileType::ScriptTS);
    division("let x = y satisfies void\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: T = y as void\n/re/g.exec(s);", FileType::ScriptTS);
    regex("x = void /re/.source;", FileType::ScriptTS);
    regex("let x: void\n/re/g.exec(s);", FileType::ScriptTS);
    regex("function f(): void {}\n/re/.test(x);", FileType::ScriptTS);
    division("x.as / 2;", FileType::ScriptTS);
}

#[test]
fn unicode_line_separator_after_token() {
    division("a++\u{2028}/re/g.exec(s);", FileType::ScriptJS);
    division("a++\u{2029}/re/g.exec(s);", FileType::ScriptJS);
    division("a--\u{2028}/re/g.exec(s);", FileType::ScriptJS);
    division("a++\u{00a0}/re/g.exec(s);", FileType::ScriptJS);
    division("let x = y as void\u{2028}/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y as void\u{2029}/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y as void\u{00a0}/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y! as void\u{2028}/re/g.exec(s);", FileType::ScriptTS);
    regex("a\u{2028}++/re/.lastIndex;", FileType::ScriptJS);
    regex("a\u{2029}++/re/.lastIndex;", FileType::ScriptJS);
    regex("return\u{00a0}++/re/.lastIndex;", FileType::ScriptJS);
    regex("let x: T\u{2028}/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: {a: T}\u{2028}/re/g.exec(s);", FileType::ScriptTS);
}

#[test]
fn ts_non_null_before_as_is_division() {
    division("let x = y! as {a: T}\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y! as {}\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y! as Array<T>\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y! as a.b.C<D>\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y! as import('m').T<U>\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y!! as {a: T} / 2;", FileType::ScriptTS);
    division("let x = f(y)! as {a: T} / 2;", FileType::ScriptTS);
    division("let x = y! satisfies {a: T} / 2;", FileType::ScriptTS);
    division("let x = !y\n/re/g.test(s);", FileType::ScriptTS);
    regex("x = !/re/.test(s);", FileType::ScriptTS);
    regex("if (!a) /re/.test(s);", FileType::ScriptTS);
}

#[test]
fn ts_import_type_chain_head() {
    division("let x = y as import('m').T<U>\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y as import('m').T<U> / 2;", FileType::ScriptTS);
    division("let x = y satisfies import('m').T<U>\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = y as import('m').a.b.T<U>\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: import('m').T<U>\n/re/g.exec(s);", FileType::ScriptTS);
    regex("let x: typeof import('./m').default\n/re/g.exec(s);", FileType::ScriptTS);
}

#[test]
fn value_ternary_and_arrow_stay_division() {
    division("let x = c ? a : b\n/re/.exec(s);", FileType::ScriptTS);
    division("let x = c ? a : b\n/re/.exec(s);", FileType::ScriptJS);
    division("let f = (a) => b\n/re/.exec(s);", FileType::ScriptJS);
    division("let x: T = c ? a : b\n/re/.exec(s);", FileType::ScriptTS);
    division("export const x: T = 1\n/re/g.exec(s);", FileType::ScriptTS);
}

#[test]
fn tsx_asi_then_jsx_element() {
    let tsx = |code: &str| kinds_of(code, FileType::ScriptTSX);
    let ks = tsx("let v: T\n<div />;");
    assert!(ks.contains(&TokenKind::JsxLt), "type-annotation ASI must open JSX: {ks:?}");
    let ks = tsx("let v\n<div />;");
    assert!(ks.contains(&TokenKind::JsxLt), "uninitialised declarator ASI must open JSX: {ks:?}");
    let ks = tsx("import R from 'r'\n<div />;");
    assert!(ks.contains(&TokenKind::JsxLt), "module-specifier ASI must open JSX: {ks:?}");
    let ks = tsx("const a = 1, div = 2;\nexport const r = a < div;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a < div stays a comparison: {ks:?}");
}

#[test]
fn ts_numeric_literal_type_forces_asi() {
    regex("let x: 1\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: -1\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: 1n\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: -1n\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: 0x1f\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: 1e3\n/re/.exec(s);", FileType::ScriptTS);
    regex("let a = 1, b: 2\n/re/.exec(s);", FileType::ScriptTS);
    regex("type A = 1\n/re/.exec(s);", FileType::ScriptTS);
    regex("declare function f(): 1\n/re/.exec(s);", FileType::ScriptTS);
}

#[test]
fn numeric_value_stays_division() {
    division("let x = 1\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: number = 1\n/re/g.exec(s);", FileType::ScriptTS);
    division("x = a - 1\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: T = a - 1\n/re/g.exec(s);", FileType::ScriptTS);
    division("x = 1\n/re/g.exec(s);", FileType::ScriptJS);
    division("x = 1n\n/re/g.exec(s);", FileType::ScriptJS);
    division("x = 0x1f\n/re/g.exec(s);", FileType::ScriptJS);
}

#[test]
fn ts_template_literal_type_forces_asi() {
    regex("let x: `lit`\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: `p${string}s`\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: `${number}`\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x: `a${'b'}c${number}d`\n/re/.exec(s);", FileType::ScriptTS);
    regex("type A = `lit`\n/re/.exec(s);", FileType::ScriptTS);
}

#[test]
fn template_value_stays_division() {
    division("let x = `lit`\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x = `p${y}s`\n/re/g.exec(s);", FileType::ScriptTS);
    division("let x: T = `lit`\n/re/g.exec(s);", FileType::ScriptTS);
    division("x = `lit`\n/re/g.exec(s);", FileType::ScriptJS);
    division("x = `p${y}s`\n/re/g.exec(s);", FileType::ScriptJS);
}

#[test]
fn ts_definite_assignment_forces_asi() {
    regex("let x!: T\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x!: T[]\n/re/.exec(s);", FileType::ScriptTS);
    regex("let x!: 1\n/re/.exec(s);", FileType::ScriptTS);
    regex("let a = 1, b!: T\n/re/.exec(s);", FileType::ScriptTS);
    division("x! / 2;", FileType::ScriptTS);
    division("x!\n/re/g.exec(s);", FileType::ScriptTS);
}

#[test]
fn restricted_production_label_precedes_regex() {
    regex("outer: for(;;){ break outer\n/re/.test(b); }", FileType::ScriptJS);
    regex("outer: for(;;){ continue outer\n/re/.test(b); }", FileType::ScriptJS);
    regex("outer: for(;;){ break outer\n/re/.test(b); }", FileType::ScriptTS);
    division("outer: for(;;){ break\nouter\n/re/g.test(b); }", FileType::ScriptJS);
    division("x.break / 2;", FileType::ScriptJS);
    division("x.continue / 2;", FileType::ScriptJS);
}

#[test]
fn tsx_literal_type_asi_then_jsx_element() {
    let tsx = |code: &str| kinds_of(code, FileType::ScriptTSX);
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
    let ks = kinds_of("outer: for(;;){ break outer\n<div />; }", FileType::ScriptJSX);
    assert!(ks.contains(&TokenKind::JsxLt), "labelled break ASI must open JSX: {ks:?}");
    let ks = tsx("const a = 1, div = 2;\nconst r = a - 1 < div;");
    assert!(!ks.contains(&TokenKind::JsxLt), "a - 1 < div stays a comparison: {ks:?}");
}

#[test]
fn named_function_expression_body_then_slash_is_division() {
    division("const x = function foo() {} /42/i", FileType::ScriptJS);
    division("(function foo() {} /42/i)", FileType::ScriptJS);
    division("!function fn() {} /42/i;", FileType::ScriptJS);
    division("x = function* generation() {} /42/i;", FileType::ScriptJS);
    division("void async function fn() {}\n/foo/g", FileType::ScriptJS);
    division("x = async function* g() {}\n/foo/g", FileType::ScriptJS);
    regex("function foo() {}\n/42/i", FileType::ScriptJS);
    regex("function* generation() {}\n/42/i", FileType::ScriptJS);
    regex("async function fn() {}\n/42/i", FileType::ScriptJS);
}

#[test]
fn postfix_increment_then_block_then_regex() {
    regex("x++\n{}\n/foo/", FileType::ScriptJS);
    regex("x--\n{}\n/foo/", FileType::ScriptJS);
    regex("a[0]++\n{}\n/foo/", FileType::ScriptJS);
    division("x = y++ / 2", FileType::ScriptJS);
}

#[test]
fn class_body_after_a_heritage_list_with_type_arguments_precedes_regex() {
    for code in [
        "class Foo implements A, V<T> {}\n/el/.exec(s);",
        "class Foo implements A, V<T<U>> {}\n/el/.exec(s);",
        "class Foo extends y implements React.FC, V<unique symbol, true> {\n\n}\n/el/.exec(s);",
        "interface I extends A, V<T> {}\n/el/.exec(s);",
    ] {
        regex(code, FileType::ScriptTS);
    }
    division("x = class implements A, V<T> {} / 2;", FileType::ScriptTS);
    division("x = class extends A implements B, V<T<U>> {} / 2;", FileType::ScriptTS);
}

#[test]
fn function_or_class_expression_after_arrow_or_shift_is_a_value() {
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
        division(code, FileType::ScriptJS);
    }
    division("x = <T>function(){} / 2;", FileType::ScriptTS);
    regex("x = a => {}\n/re/.test(s);", FileType::ScriptJS);
    regex("x = () => {}\n/re/.test(s);", FileType::ScriptJS);
}

#[test]
fn function_expression_return_type_then_body_is_a_value() {
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
        division(code, FileType::ScriptTS);
    }
    regex("function f(): T {}\n/re/.test(x);", FileType::ScriptTS);
    regex("function f(): { a: T } {}\n/re/.test(x);", FileType::ScriptTS);
    regex("let x: { a: T }\n/re/.test(x);", FileType::ScriptTS);
    let ks = kinds_of("x = [function (): T {}\n< y];", FileType::ScriptTSX);
    assert!(!ks.contains(&TokenKind::JsxLt), "{ks:?}");
    assert!(ks.contains(&TokenKind::Lt), "{ks:?}");
    let codes = diag_codes_of("x = [function (): T {}\n< y];", FileType::ScriptTSX);
    assert!(codes.is_empty(), "{codes:?}");
}

#[test]
fn label_colon_inside_a_function_expression_body_is_a_statement() {
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
        stream(code, FileType::ScriptJS, want);
    }
    division("x = function(){ return {a: {} / 2} };", FileType::ScriptJS);
    division("x = function(){ a = c ? d : {} / 2 };", FileType::ScriptJS);
    division("x = { a: {} / 2 };", FileType::ScriptJS);
}

#[test]
fn bodiless_function_signature_before_a_line_break_ends_the_statement() {
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
        stream(code, FileType::ScriptTS, want);
    }
    let ks = kinds_of("declare function y()\n<div/>;", FileType::ScriptTSX);
    assert!(ks.contains(&TokenKind::JsxLt), "{ks:?}");
    division("x = f()\n/ 2 / 3;", FileType::ScriptTS);
    division("f<T>()\n/ 2 / 3;", FileType::ScriptTS);
    division("new Foo()\n/ 2 / 3;", FileType::ScriptTS);
    division("x = function(){}\n/ 2 / 3;", FileType::ScriptTS);
}

#[test]
fn operand_heads_before_a_value_brace() {
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
        stream(code, FileType::ScriptJSX, want);
    }
    division("f(...{a: 1} / 2);", FileType::ScriptJS);
    division("x = [...function(){} / 2];", FileType::ScriptJS);
    division("f(...class {} / 2);", FileType::ScriptJS);
    division("for (const k of {a: 1} / 2) ;", FileType::ScriptJS);
    division("for (x of {} / 2) ;", FileType::ScriptJS);
    division("x = a, void {} / 2;", FileType::ScriptJS);
    division("f(a, void {} / 2);", FileType::ScriptJS);
}

#[test]
fn unicode_whitespace_before_a_glued_number() {
    division("{\u{2028}5. // c\n/\u{a0}b / c; }", FileType::ScriptJS);
    division("x = \u{a0}5.\n/ 2 / 3;", FileType::ScriptJS);
    division("x = \u{2028}5\n/ 2 / 3;", FileType::ScriptJS);
}

#[test]
fn function_head_walks_cross_every_return_type_shape() {
    division(
        "const P = function a(x: A): abstract new () => abstract new () => keyof any {}\n/ 2 / 3;",
        FileType::ScriptTS,
    );
    division("x = [function (): _ is Record<Promise<V>> {}\n/ 2 / 3];", FileType::ScriptTS);
    regex(
        "function y(): x is abstract new () => typeof import('m') extends boolean ? K<A, B<unique symbol>> : void[] {}\n/[a-z]+/v.x;",
        FileType::ScriptTS,
    );
    regex(
        "async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;",
        FileType::ScriptTS,
    );
    regex("function f(): { a: T } {}\n/re/.test(x);", FileType::ScriptTS);
    regex(
        "async function obj(): K<U, $JSX<Array<U>>> {\nclass C implements V, E<$JSX<-1>> {\n\n}\n/[\\]]/.test(s);\n}",
        FileType::ScriptTS,
    );
    let ks = kinds_of(
        "x = [async function $(): _ is Record<Promise<V>>{} < function Bar(arr?: any): K<this>{}];",
        FileType::ScriptTSX,
    );
    assert!(ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::JsxLt), "{ks:?}");
}

#[test]
fn brace_after_a_generic_return_type_is_a_body() {
    for (code, file_type) in [
        (
            "async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;",
            FileType::ScriptTS,
        ),
        (
            "async function fn($, x): N is <a>() => Array<symbol> {\n}\n/a{1,2}/u.x;",
            FileType::ScriptTSX,
        ),
        ("function f(): () => Array<symbol> {}\n/re/.x;", FileType::ScriptTS),
        ("function f(): A | B<C> {}\n/re/.x;", FileType::ScriptTS),
        (
            "x = function* (): P<Q<R>> {}\nwhile (a) { async (p): B => 1\n}\n/re/.x;",
            FileType::ScriptTS,
        ),
    ] {
        regex(code, file_type);
    }
    division("x = f < T > {} / 2;", FileType::ScriptTS);
    division("x = c ? function* (...a): Pick<E<F<G>>>[][] {} / 2 : null;", FileType::ScriptTS);
}

#[test]
fn ts_postfix_bang_before_a_block_brace() {
    regex("[typeof ab]!\n{\n}\n/x/.test(s);", FileType::ScriptTS);
    regex("x!\n{}\n/x/.test(s);", FileType::ScriptTS);
    regex("f(x)!\n{}\n/x/.test(s);", FileType::ScriptTS);
    division("x = !{} / 2;", FileType::ScriptTS);
    division("x = a && !{}.b / 2;", FileType::ScriptTS);
    division("x!\n/re/g.exec(s);", FileType::ScriptTS);
}

#[test]
fn adjacent_type_atoms_end_an_annotation() {
    division("let x: Foo<T>\nf(y)\n/re/.test(s);", FileType::ScriptTS);
    division("let x: T\nf(y)\n/re/.test(s);", FileType::ScriptTS);
    division("let x: T[]\nf(y)\n/re/.test(s);", FileType::ScriptTS);
    division("let x: `lit`\nf(y)\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: typeof import('m').default\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: asserts y is T\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: abstract new () => T\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: readonly unique symbol[]\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: A extends infer U extends B ? C : D\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: `a${T}b${U}c`\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: { [K in keyof T as `x${K}`]: T[K] }\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: <T>(a: T) => T\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: new <T>(a: T) => T\n/re/.test(s);", FileType::ScriptTS);
    regex("let x: <T>(a: T) => T | undefined\n/re/.test(s);", FileType::ScriptTS);
    regex("let a: A, b: B, c: <T>(a: T) => T\n/re/.test(s);", FileType::ScriptTS);
    regex("declare function f<T>(a: T): <U>(b: U) => T\n/re/.test(s);", FileType::ScriptTS);
    regex("type A<T> = <T>(a: T) => T\n/re/.test(s);", FileType::ScriptTS);
    for code in [
        "declare let x: <T>(a: T) => T\n<div />;",
        "let x: <T>(a: T) => T | undefined\n<div />;",
        "let a: A, b: B, c: <T>(a: T) => T\n<div />;",
        "type A = <T>(a: T) => T\n<div />;",
        "declare function f(): <T>(a: T) => T\n<div />;",
    ] {
        let ks = kinds_of(code, FileType::ScriptTSX);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    }
    let ks = kinds_of("let P: Array<bigint, this>\n_(x)\n<(P().foo);", FileType::ScriptTSX);
    assert!(ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::JsxLt), "{ks:?}");
    let codes = diag_codes_of("let P: Array<bigint, this>\n_(x)\n<(P().foo);", FileType::ScriptTSX);
    assert!(codes.is_empty(), "{codes:?}");
}
