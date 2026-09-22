use crate::{error::DiagCode, token::TokenKind};

use crate::pipeline::disambiguate::tests::{
    FileType, diag_codes_of, division, gt_run_fused, gt_run_split, is_fused_gt, kinds_of, stream,
};

use crate::pipeline::disambiguate::FORWARD_SCAN_CAP;

// Reduce repeated boilerplate in tests below.
// Can reference `ScriptJS` directly, instead of `FileType::ScriptJS`.
use FileType::*;

#[test]
fn gt_run_in_expression_position_follows_the_type_argument_lookahead() {
    for follower in [
        "e",
        "1",
        "\"s\"",
        "[1]",
        "[]",
        "{}",
        "!e",
        "+e",
        "-e",
        "typeof e",
        "this",
        "new E()",
        "function () {}",
    ] {
        gt_run_fused(&format!("foo(a<b, c<d >> {follower});"));
    }
    gt_run_fused("foo(a<b, c<d, e<f >>> g);");
    gt_run_fused("x = a<b>>c;");
    gt_run_fused("x = a<b>>=c;");
    gt_run_fused("x = a<b>>>c;");
    for follower in [
        "(e)",
        "`t`",
        "== e",
        "& e",
        "| e",
        "* e",
        "/ e",
        "% e",
        "?? e",
        "as X",
        "instanceof e",
        ", e",
        ".y",
        "",
    ] {
        gt_run_split(&format!("foo(a<b, c<d >> {follower});"));
    }
    gt_run_split("foo(a<b, c<d >>\ne);");
    gt_run_split("foo(a<b, c<d >> /* c\n */ e);");
    gt_run_split("x = a<b<c>>;");
    gt_run_split("x = a<b<c>>.y;");
    gt_run_split("x = a<b<c>>(y);");
    gt_run_split("x = a<b<c>>`t`;");
}

#[test]
fn gt_run_in_type_context_always_splits() {
    for code in [
        "let x: a<b<c>>[] = y;",
        "let x: a<b<c>>[1] = y;",
        "function f(): a<b<c>> { return null! }",
        "class X extends a<b<c>> {}",
        "class X extends a<b<c>> implements D {}",
        "interface I extends B<B<string>> {}",
        "class X<T extends A<B<C>>> {}",
        "class X<T = A<B>> {}",
        "function f<T = A<B>>() {}",
        "class X { m<T = A<B>>() {} }",
        "class X { a: T; m<T = A<B>>() {} }",
        "class X { static m<T = A<B>>() {} }",
        "let o = { m<T = A<B>>() {} };",
        "let x: Map<K, Foo<Bar<T>>[]> = y;",
        "x = f<K, Foo<Bar<T>>[]>(y);",
        "var x = <Array<Base>>[d1, d2];",
        "type T = A<B<C>> | D;",
        "type T = A<B<C>> extends D ? E : F;",
        "let y = x as A<B<C>> as D;",
        "let x: (A<B<C>>[]) = y;",
        "let x: A | Foo<Bar<T>>[] = y;",
        "let f: (x: T) => Foo<Bar<T>>[] = y;",
        "let x: T extends U ? Foo<Bar<T>>[] : never = y;",
        "let x: Obj[Foo<Bar<T>>[0]] = y;",
        "let x: [A, Foo<Bar<T>>[]] = y;",
        "let x: { a: T, b: Foo<Bar<T>>[] } = y;",
        "function f(x: A | Foo<Bar<T>>[]) {}",
        "interface I { a: Map<K, Foo<Bar<T>>[]> }",
        "class X extends React.Component<Props<T>> {}",
        "let x = <Foo<Bar<T>>>y;",
        "let x = <A<B<C<D>>>>y;",
        "let x = <Map<K, Set<V>>>y;",
        "let s = <Foo<Bar<T>>>\"str\";",
        "let n = <Foo<Bar<T>>>-1;",
        "let x = <Foo<Bar<T>>>[d1, d2];",
        "let x = <Foo<Bar<T>>>{ a: 1 };",
        "class C<T extends List<List<T>> > {}",
        "let x: Map<K, List<List<T>> > = y;",
        "declare function f<T>(p: T): Foo<Bar<T>>[]\n<Foo>hello world</Foo>;",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn gt_glued_to_eq_splits_only_in_type_context() {
    let ts = |code: &str| kinds_of(code, ScriptTS);
    for code in [
        "var v : Foo<T>= 1;",
        "type X<T>= T;",
        "let v: Foo<Bar<T>>= 1;",
        "let v: Foo<Bar<Quux<T>>>= 1;",
        "let x: Map<A, 1>= y;",
    ] {
        let ks = ts(code);
        assert!(
            ks.contains(&TokenKind::Eq)
                && !ks.iter().any(|k| is_fused_gt(*k) || *k == TokenKind::Ge),
            "{code:?} closes a type-argument list: {ks:?}"
        );
    }
    assert!(ts("x = a<b>=c;").contains(&TokenKind::Ge));
    assert!(ts("if (i>=0) {}").contains(&TokenKind::Ge));
    assert!(ts("x = y >= 1;").contains(&TokenKind::Ge));
    assert!(ts("foo(a<b, c<d>>= e);").contains(&TokenKind::RShiftEq));
}

#[test]
fn gt_run_region_must_scan_as_a_type_list() {
    for code in [
        "foo(a<b + 1, c<d >> (e));",
        "foo(a<b * 2, c<d >> (e));",
        "foo(a<b(c), d<e >> (f));",
        "foo(a<b - 1, c<d >> (e));",
        "foo(a<b?.c, d<e >> (f));",
        "foo(a<b ? c : d, e<f >> (g));",
        "foo(a<b | c ? d : e, f<g >> (h));",
        "foo(a<b + 1, c<d<e >>> (f));",
        "foo(a<b, c<d + 1 >> (e));",
        "foo(a<b c, d<e >> (f));",
        "{ a<b + 1, c<d >> (e) }",
        "x; a<b + 1, c<d >> (e);",
    ] {
        gt_run_fused(code);
    }
    for code in [
        "foo(a<b.c, d<e >> (f));",
        "foo(a<b[0], c<d >> (e));",
        "foo(a<-1, c<d >> (e));",
        "foo(a<{ +readonly [K in T]+?: U }, c<d >> (e));",
        "foo(a<b < c, d<e >> (f));",
        "foo(a<`p${string}`, c<d >> (e));",
        "foo(a<typeof import('m').T, c<d >> (e));",
        "foo(a<new () => T, c<d >> (e));",
        "foo(a<[x?, ...y[]], c<d >> (e));",
        "foo(a<T extends U ? X : Y, c<d >> (e));",
        "foo(a<{ m(): T }, c<d >> (e));",
        "foo(a<(x: T) => U, c<d >> (e));",
        "foo(a<keyof T, c<d >> (e));",
        "foo(a<<T>(x: T) => T, c<d >> (e));",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn lt_lt_openers_close_with_a_split_gt_run() {
    for code in
        ["let e: Map<<T>(x: T) => T, Set<U>> = null!;", "const r = f<<T>(x: T) => Array<T>>(0);"]
    {
        let ks = kinds_of(code, ScriptTS);
        assert!(
            !ks.iter().any(|k| matches!(k, TokenKind::LShift | TokenKind::RShift)),
            "{code:?}: {ks:?}"
        );
    }
}

#[test]
fn gt_run_head_context_covers_heritage_members_and_spreads() {
    for code in [
        "interface X extends A, B<C<D>> {}",
        "class X extends A implements B, C<D<E>> {}",
        "interface X<T> extends A<T>, Pick<B<T>, Exclude<keyof C<T>, \"x\">> {\n  a: 1;\n}",
        "f(x => { const a: A<B<C>>[] = []; });",
        "f(function () {\n  const list: A<B<number>>[] = [];\n});",
        "f(() => { const {a}: A<B<C>>[] = x; });",
        "f(() => { const [a]: A<B<C>>[] = x; });",
        "const m = new Map<string, (t: string, k?: X) => Thenable<number>>();",
        "const f = promisify<(a: string, options?: ncp.Options) => Promise<void>>(util.promisify(ncp));",
        "class C { x?: Exclude<Record<U>>[]; }",
        "class C { static override 0.5?: Exclude<Record<U>>[]; }",
        "let x: [a?: X, ...Set<C<Foo>>[]];",
        "function f([a, b]: [obj?: X, ...Set<a.b.C<Foo>>[]]) {}",
        "let x: | K<Map<V<Foo>>>[] | X;",
        "declare function f(this: | K<Map<V<Foo>>>[][] | X, y?: Z);",
        "class C { #p<T = A<B<C>>>(x) {} }",
        "class C { static #p<T = A<B<C>>>(x) {} }",
        "class C { *m<T = A<B<C>>>(x) {} }",
        "class C { static async *m<T = A<B<C>>>(x) {} }",
        "class C { m() {} *g<T = A<B<C>>>(x) {} }",
        "class C { x = 1; *g<T = A<B<C>>>(x) {} }",
        "function *g<T = A<B<C>>>() {}",
        "x = function* <T = A<B<C>>>() {};",
        "let o = { *m<T = A<B<C>>>() {} };",
        "type T = { a(): [...T1, ...T2]\n  <const T1 extends A<B>, const T2 extends A<B>>(x: T1): T2 }",
        "export const f: {\n  <T2 extends ReadonlyArray<unknown>>(that: T2): <T1 extends ReadonlyArray<unknown>>(self: T1) => [...T1, ...T2]\n  <const T1 extends ReadonlyArray<unknown>, const T2 extends ReadonlyArray<unknown>>(self: T1, that: T2): [...T1, ...T2]\n} = dual(2, x);",
        "let a: T[]\n<A<B<C>>>(y);",
    ] {
        gt_run_split(code);
    }
    for code in [
        "let o = { ...a<b<c>>[0] };",
        "x = [...a<b<c>>[0]];",
        "f(...a<b<c>>[0]);",
        "x = a * b<c<d>>[0];",
        "x = a ** b<c<d>>[0];",
        "function* g() { yield* a<b<c>>[0]; }",
        "switch (x) { case {}: b<c<d>>[0]; }",
        "let o = { const: 1, [a]: b<c<d>>[0] };",
        "a, b<c<d>>[0];",
        "foo(a<(b ? c : d), e<f >> (g));",
    ] {
        gt_run_fused(code);
    }
    let ks = kinds_of("f(x => { const a: A<B<C>>[] = []; }, { a: b<c<d>>[0] });", ScriptTS);
    assert_eq!(ks.iter().filter(|k| is_fused_gt(**k)).count(), 1, "{ks:?}");
}

#[test]
fn gt_follower_after_a_line_break_still_rejects_the_shift_operands() {
    for code in [
        "let x = f<A<B<C>>>\n+1;",
        "let x = f<A<B<C>>>\n-1;",
        "let x = f<A<B<C>>>\n// c\n+1;",
        "let x = f<A<B<C>>> /* c\n */ +1;",
        "let x = f<A<B<C>>>\u{2028}+1;",
    ] {
        gt_run_fused(code);
    }
    let ks = kinds_of("let x = f<A<B<C>>>\n<div/>;", ScriptTSX);
    assert!(ks.contains(&TokenKind::URShift), "{ks:?}");
    for code in [
        "let x = f<A<B<C>>>\n++y;",
        "let x = f<A<B<C>>>\n--y;",
        "let x = f<A<B<C>>>\n[0];",
        "let x = f<A<B<C>>>\n(0);",
        "x = f<A<B<C>>>\n<< y;",
        "x = f<A<B<C>>>\n<= y;",
        "x = f<A<B<C>>>\n-= y;",
        "x = f<A<B<C>>>\n!y;",
        "x = a<b<c>> <= d;",
        "x = a<b<c>> << d;",
        "x = a<b<c>> += d;",
        "let x: Foo<Bar<T>>\n+1;",
        "type X = Foo<Bar<T>>\n+1;",
        "class C<T extends List<List<T>>\n> {}",
    ] {
        gt_run_split(code);
    }
    let ks = kinds_of("let x: Map<K, V<W<T>>>\n<div/>;", ScriptTSX);
    assert!(!ks.iter().any(|k| is_fused_gt(*k)), "{ks:?}");
}

#[test]
fn gt_run_in_class_expression_heritage_lists() {
    for code in [
        "(class implements Thenable<V<number>>, Bar<Promise<-1, Thenable<intrinsic, void>>> {\n\n})\n",
        "[class implements Array, Set<Bar<never>> {}];",
        "f(a, class implements B, C<D<E>> {});",
        "x = c ? class implements A, V<T<U>> {} : d;",
        "try {} catch (e) { interface I extends A, V<T<U>> {} }",
        "(class <const T, U,> implements Exclude<$JSX<null>>, Exclude<Pick<this>> {});",
        "class <U, V extends Map<K<L>>> {}",
        "x = function <U extends A<B<C>>>() {};",
        "x = y?.[function <U extends A<B<C>>>(_: 'lit', n) {}];",
        "let o = { [k]<T = A<B<C>>>() {} };",
        "class C { [k]<T = A<B<C>>>() {} }",
        "class C { static [k]<T = A<B<C>>>() {} }",
        "class C { m() {} [k]<T = A<B<C>>>() {} }",
        "x = y as Foo | Bar<Baz<T>>[];",
        "x = y as Foo & Bar<Baz<T>>[];",
        "let x: typeof y | Bar<Baz<T>>[];",
    ] {
        gt_run_split(code);
    }
    for code in ["x = [a, [b]<c<d>>[0]];", "x = a, b<c<d>>[0];"] {
        gt_run_fused(code);
    }
}

#[test]
fn gt_run_head_after_a_line_break_follows_asi() {
    for code in [
        "debugger\n<Map<Partial>>baz;",
        "for (;;) { break\n<Map<Partial>>baz; }",
        "for (;;) { continue\n<Map<Partial>>baz; }",
        "declare const x: bigint /*\n*/ <T<0x1f>>$;",
        "declare const 名前: bigint /*\n*/ <T<0x1f>>$;",
        "let x: bigint\n<Map<Partial>>baz;",
        "let x: Foo\n<Map<P>>baz;",
        "let x: Foo.Bar\n<Map<P>>baz;",
        "let x: Foo[]\n<Map<P>>baz;",
        "let x: Foo<Bar>\n<Map<P>>baz;",
        "let x: (Foo)\n<Map<P>>baz;",
        "let x: 1\n<Map<P>>baz;",
        "let x: \"lit\"\n<T<0x1f>>$;",
        "let x: `lit`\n<T<0x1f>>$;",
        "let x: true\n<T<0x1f>>$;",
        "let x: typeof y\n<Map<P>>baz;",
        "type X = Foo\n<Map<P>>baz;",
        "function f(): Foo\n<Map<P>>baz;",
        "if (x)\n<Map<P>>baz;",
        "if (x) <Map<P>>baz;",
        "while (x)\n<Map<P>>baz;",
        "let x = <Foo><Bar<Baz<T>>>y;",
        "let x = <Foo>\n<Bar<Baz<T>>>y;",
        "return\n<Map<P>>baz;",
    ] {
        gt_run_split(code);
    }
    for code in [
        "x = y as Foo\n<Map<P>>baz;",
        "x = y satisfies Foo\n<Map<P>>baz;",
        "x = y as Foo<Bar>\n<Map<P>>baz;",
        "let x = y\n<Map<P>>baz;",
        "let x = 1\n<Map<P>>baz;",
        "x\n<Map<P>>baz;",
        "foo()\n<Map<P>>baz;",
        "x = new Foo\n<Map<P>>baz;",
        "function f() { return y\n<Map<P>>baz; }",
        "throw y\n<Map<P>>baz;",
        "x = 1\n<Map<Partial>>baz;",
    ] {
        gt_run_fused(code);
    }
}

#[test]
fn gt_run_third_tier_head_contexts() {
    for code in [
        "_(...<Promise<Bar, intrinsic>><React.FC<Bar<1n>>>[obj]);",
        "x = [...<Record<number, Promise<K<T<Record<K>>>>>>1e3] as Foo;",
        "export default <keyof Partial<typeof import('m'), 1e3>>arr;",
        "interface I extends A.B { a: Foo<Bar<T>>[] }",
        "interface y<K> extends React.FC<T<K>>, React.FC { 'a': K<Promise<Thenable<U, \"lit\">>>[Map<T>] }",
        "interface I { m?(): Promise<Array<X>>[] }",
        "class C { m?(): Promise<Array<X>>[] }",
        "interface I { [k]?(): Promise<Array<X>>[] }",
        "x = y as Foo<Bar<T>> - 1;",
        "x = y as Foo<Bar<T>> < z;",
        "x = y as Foo<Bar<T>> + 1;",
        "x = y satisfies Record<Bar<React.FC<Map<Pick<1>>>>> - Props;",
        "throw <A<B>>el! satisfies Array<V<object>> & Pick<Exclude<1n>> - --$[s];",
        "let o = { \"k\"<T = A<B<C>>>() {} };",
        "let o = { 1<T = A<B<C>>>() {} };",
        "class C { @dec() \"\"<T = A<B<C>>>(x) {} }",
        "class C { @dec m<T = A<B<C>>>(x) {} }",
        "class C { @dec [k]<T = A<B<C>>>(x) {} }",
        "let x: [Bar<U>?, ...Foo: U<Array<1>>[]];",
        "class C<V = <x>(bar: `p${Map<T<U>>}s` | a.b.C<T<Set<V>>>[]) => R> {}",
        "f<A extends B<C, D> ? E : F<G<H>>>(x);",
        "x = a as | $JSX<Partial<T<void>>>[] | B;",
        "declare const c: V extends R<A, B> ? E : Record<Map<V<Set<1>>>>[];",
        "declare const c: V<K<T>> extends R<A, T extends B ? C : D> ? E : Record<Map<V<Set<1>>>>[];",
        "class C extends (e) { x: Foo<Bar<T>>[]; }",
        "class C extends (e) { #p!: Foo<Bar<T>>[]; }",
        "class C extends (e) { @dec #p: Record<T<T<Array>>[]>; }",
        "let x: Foo.Bar[Baz<Qux<T>>];",
    ] {
        gt_run_split(code);
    }
    for code in [
        "x = a ? (b) : c<d<e>>[0];",
        "f(a, b ? (c) : d<e<f>>[0]);",
        "let o = { x: a ? (b) : c<d<e>>[0] };",
    ] {
        gt_run_fused(code);
    }
    let ts = |code: &str| kinds_of(code, ScriptTS);
    for code in ["x = y satisfies T<string, React.FC>>>this.baz;", "x = y as Foo<Bar>>>z;"] {
        let ks = ts(code);
        assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{code:?}: {ks:?}");
        assert!(!ks.contains(&TokenKind::URShift), "{code:?}: {ks:?}");
    }
    let ks = ts("x = y as Foo<Bar<T>>>>z;");
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 2, "{ks:?}");
}

#[test]
fn this_heads_and_value_braces_before_a_run() {
    for code in [
        "class C implements this<Record, this>, K<Record<Pick<T>>> {}",
        "this<A<B>>(x);",
        "class C { m() { return this<A<B>>(x); } }",
        "interface I { a: T }\n<Map<P>>baz;",
        "function f() {}\n<Map<P>>baz;",
        "type X = A extends B<C, D extends E ? F : G> ? H : I<J<K>>[];",
    ] {
        gt_run_split(code);
    }
    for code in [
        "new Foo<this<this, a.b.C<symbol>>>(x);",
        "x = y as this<Foo<this>>[];",
        "(e)<true[a.b.C], this<Pick<boolean>>>(f);",
        "x = this<A<B>>[0];",
        "x = class extends c {}\n<React.FC<K>>ete;",
        "x = y as V | { get baz(): Partial<Array<V, object>> }\n<T<Thenable>>c;",
        "x = a - 1\n<Map<P>>baz;",
        "x = a * 'n'<b<c>>[0];",
        "x = {}\n<Map<P>>baz;",
        "x = class {}\n<Map<P>>baz;",
        "x = a > b ? c : d<e<f>>[0];",
    ] {
        gt_run_fused(code);
    }
    let ks = kinds_of("x = a >>ete<this<1n, Map>>(arr);", ScriptTS);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 2, "{ks:?}");
    for code in [
        "let Foo: -1\n\n<React.FC<undefined, this>>a;",
        "let x: -1 | -2\n<Map<P>>baz;",
        "class C { @dec() #p<V = Set<Bar<T>>>() {} }",
        "class C { @dec() static m<T = A<B<C>>>() {} }",
        "class C { @dec() async *'n'<V = Promise<Promise<T>>>() {} }",
        "let o = { *'n'<T = A<B<C>>>() {} };",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn escaped_type_parameter_name_after_lt_lt() {
    stream(
        "let s: a<<\\u{62}c>(x: T) => T>;",
        ScriptTS,
        "let IDENT : IDENT < < IDENT > ( IDENT : IDENT ) => IDENT > ;",
    );
    stream(
        "const r = f<<\\u{62}c>(a: \\u{62}c) => \\u{62}c>(y);",
        ScriptTS,
        "const IDENT = IDENT < < IDENT > ( IDENT : IDENT ) => IDENT > ( IDENT ) ;",
    );
}

#[test]
fn template_literal_type_inside_a_lt_lt_list() {
    for code in [
        "(a?.c)<<n>() => T<F<G<\"lit\">>>, readonly R<0x1f, Foo>[], `${Foo<Map<R>>}`,>(y);",
        "let e: Map<<T>(x: T) => `${T}`, U>;",
    ] {
        let ks = kinds_of(code, ScriptTS);
        assert!(!ks.contains(&TokenKind::LShift), "{code:?}: {ks:?}");
    }
}

#[test]
fn tsx_generic_function_type_in_type_position() {
    let tsx = |code: &str| kinds_of(code, ScriptTSX);
    let type_params = |code: &str| {
        let ks = tsx(code);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?} must not open JSX: {ks:?}");
    };
    let jsx = |code: &str| {
        let ks = tsx(code);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    };

    type_params("let a: <T>(x: T) => T = null!;");
    type_params("let b: { f: <T>(x: T) => T } = null!;");
    type_params("type F = <T>(x: T) => T;");
    type_params("function g(): <T>(x: T) => T { return null!; }");
    type_params("let k: (f: <T>(x: T) => T) => void = null!;");
    type_params("interface I { m: <T>(x: T) => T }");
    type_params("class C { m: <T>(x: T) => T = null!; }");
    type_params("let n: { a: { b: <T>(x: T) => T } } = null!;");
    type_params("let h = <T,>(x: T) => x;");
    type_params("type A = { m: <T>(a: T) => T };");
    type_params("type A<U> = { m: <T>(a: T) => T };");
    type_params("let z: A | <T>(a: T) => T = null!;");
    type_params("declare function f(a: <T>(a: T) => T): void;");
    type_params("function f(a: <T>(a: T) => T) {}");
    type_params("const r = f<{ m: <T>(x: T) => T }>(0);");
    type_params("const r = g<<T>(x: T) => T>(0);");
    type_params("let w = h<{ a: { b: <T>(x: T) => T } }>(1);");
    type_params("var a = <T>(x: T) => x;");

    jsx("function Badge(): JSX.Element { return <em>(new)</em>; }");
    jsx("const Note = () => <Callout>(see docs)</Callout>;");
    jsx("const p = <code>/usr/bin</code>;");
    jsx("const q = <div>\n  // Not Comment\n</div>;");
    jsx("const r = <div>\n\t/*test*/\n</div>;");
    jsx("const s = <div>(x)</div>;");
    jsx("let o2 = { m: <Foo>(bar)</Foo> };");
    jsx("f({ a: <T>(x)</T> });");
    jsx("function g() { return f(c ? a : <T>(x)</T>); }");
    jsx("function g(cb) { cb(c ? a : <Foo>(y)</Foo>); }");
    jsx("function g() { return c ? a : <T>(x)</T>; }");
    jsx("const r = f<{ m: <T>(hi)</T> }.m > (0);");
    jsx("let u = <T>(a)</T>;");
    jsx("let u = <T\n>(a)</T\n>;");
    jsx("class Panel { footer = beta ? null : <Note>(stable)</Note>; }");
    jsx("function Row(icon = flag ? null : <Badge>(new)</Badge>) { return icon; }");
    jsx("let u = <T>(a)</ /*b*/ T>;");
    jsx("let u = <T>(a)</\u{a0}T>;");
    jsx("const a = <div\u{a0}id=\"x\">hi</div>;");
    jsx("const a = <Foo extends />;");
    jsx("const a = <Foo extends>x</Foo>;");
    type_params("interface Props { onRender?: <Item>(x: Item) => Item }");
    type_params("class C { m?: <T>(x: T) => T }");
    type_params("function f(a?: X, b: <T>(x: T) => T) {}");
    type_params("class C { a = 1;\n m: <T>(x: T) => T = null!; }");
    type_params("interface I<T> { m: <U>(x: U) => U }");
    type_params("class C<T> { m: <U>(x: U) => U = null!; }");
    type_params("function f<T>(cb: <U>(x: U) => U) {}");

    let ks = tsx("f(a << b, c);");
    assert!(!ks.contains(&TokenKind::JsxLt), "shift must not open JSX: {ks:?}");
    let ks = tsx("x <<= 1;");
    assert!(!ks.contains(&TokenKind::JsxLt), "shift-assign must not open JSX: {ks:?}");
    let ks = tsx("let v = a < b > (c);");
    assert!(!ks.contains(&TokenKind::JsxLt), "comparison chain: {ks:?}");
}

#[test]
fn jsx_candidate_scans_stay_bounded() {
    let tsx = |code: &str| kinds_of(code, ScriptTSX);
    let ks = tsx("let a: <T>(x: T) => T = null!;");
    assert!(!ks.contains(&TokenKind::JsxLt), "type position: {ks:?}");
    let far = format!("let a = <T>(x){}</T>;", " ".repeat(70 * 1024));
    let ks = tsx(&far);
    assert!(ks.contains(&TokenKind::JsxLt), "no arrow after the parameters: JSX");
    let many = "const r = f<{ m: <T>(x: T) => T }>(0);\n".repeat(5000);
    let ks = tsx(&many);
    assert!(!ks.contains(&TokenKind::JsxLt), "every site is a function type: {ks:?}");
}

/// `JSXText` is `SourceCharacter but not one of {, <, > or }`, so a `}`
/// reached before any `{` or `<` proves the candidate is not an element.
/// That decides the pair which shares the prefix `const r = f<{ m: <T>(`
/// and has opposite answers, even when a same-named close tag exists
/// elsewhere in the file for the probe to find.
#[test]
fn bare_brace_in_children_rules_out_jsx() {
    let tsx = |code: &str| kinds_of(code, ScriptTSX);
    let ks = tsx("const r = f<{ m: <T>(x: T) => T }>(0);\nconst z = <T>hi</T>;");
    assert_eq!(
        ks.iter().filter(|k| **k == TokenKind::JsxLt).count(),
        2,
        "only the element on line 2 is JSX: {ks:?}"
    );
    // The other half of the pair: children are `(hi)`, which is text, so
    // this stays JSX.
    let ks = tsx("const r = f<{ m: <T>(hi)</T> }.m > (0);");
    assert!(ks.contains(&TokenKind::JsxLt), "clean children stay JSX: {ks:?}");
    // A `{` container may legitimately contain `}` inside a string, so the
    // scan stops there and leaves the answer to the probe.
    let ks = tsx("const a = <div>{\"}\"}</div>;");
    assert!(ks.contains(&TokenKind::JsxLt), "container braces are not text: {ks:?}");
}

#[test]
fn tsx_generic_function_type_is_decided_by_its_arrow() {
    let jsx_count =
        |code: &str| kinds_of(code, ScriptTSX).iter().filter(|k| **k == TokenKind::JsxLt).count();
    let tail = "\nconst z = <T>hi</T>;";
    for head in [
        "const r = f<{ m: <T>(x: T) => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: T) => { a: T } }>(0);",
        "const r = f<{ m: <T>(x: T) => T | Q<T> }>(0);",
        "const r = f<{ m: <T>(x: Array<T>) => T }>(0);",
        "const r = f<{ m: <T>(x: { a: T }) => T }>(0);",
        "const r = g(f<{ m: <T>(x: T) => Array<T> }>(0));",
        "const r = f<{ m: <T>(x: T /* ) */) => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: T)\n  => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: `a${T}b`) => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: `a${`b${T}`}c`) => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: \"(\") => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: ')') => Array<T> }>(0);",
        "const r = f<{ m: <T>(x: T /* ( */) => Array<T> }>(0);",
    ] {
        assert_eq!(jsx_count(&format!("{head}{tail}")), 2, "{head:?}");
    }
    for code in [
        "const r = f<{ m: <T>(x: T) => Array<T> }>(0);\nconst s = \"</T>\";",
        "const r = f<{ m: <T>(x: T) => Array<T> }>(0);\n// see </T>\n",
    ] {
        assert_eq!(jsx_count(code), 0, "{code:?}");
    }
    let big = format!(
        "const r = f<{{ m: <T>(x: T) => Array<T> }}>(0);\n{}",
        "const tail = 1;\n".repeat(6000)
    );
    assert_eq!(jsx_count(&big), 0, "a site more than 64 KiB from EOF");
    for code in [
        "const r = f<{ m: <T>(hi)</T> }.m > (0);",
        "const r = f<{ m: <T>(x: T) {'=>'} T</T> }.m > (0);",
        "let u = <T>(a)</T>;",
        "let u = <T>(a) /* => */</T>;",
    ] {
        assert_eq!(jsx_count(code), 2, "{code:?}");
    }
}

#[test]
fn tsx_generic_arrow_in_expression_position_is_diagnosed() {
    let diagnosed = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(
            codes.contains(&DiagCode::UnterminatedJsxElement),
            "{code:?} must be diagnosed: {codes:?}"
        );
        let ks = kinds_of(code, ScriptTSX);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?} lexes as type parameters: {ks:?}");
    };
    let silent = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(!codes.contains(&DiagCode::UnterminatedJsxElement), "{code:?} is valid: {codes:?}");
    };
    for code in [
        "var a = <T>(x: T) => x;",
        "x = { m: <T>(x: T) => T };",
        "f(<T>(x: T) => x);",
        "function g() { return <T>(x: T) => x; }",
        "const a = [<T>(x: T) => x];",
        "x = cond ? <T>(x: T) => x : y;",
        "x = (<T>(x: T) => x);",
        "<T>(x: T) => x;",
        "var j = <T>() => {}</T>;",
        "let o = { m: <T>() => {}</T> };",
    ] {
        diagnosed(code);
    }
    for code in [
        "const r = f<{ m: <T>(x: T) => T }>(0);",
        "let a: <T>(x: T) => T = null!;",
        "function f(cb: <T>(x: T) => T) {}",
        "type F = <T>(x: T) => T;",
        "let x: A | <T>(x: T) => T = null!;",
        "const r = f<(<T>(x: T) => T)>(0);",
        "class C<T = <U>(x: U) => U> {}",
        "let x: [<T>(x: T) => T] = null!;",
        "let h = <T,>(x: T) => x;",
        "const r = f<{ m: <T>(hi)</T> }.m > (0);",
        "interface I { m: <T>(x: T) => T }",
        "let x: { f: <T>(x: T) => T } = null!;",
        "let w = h<{ a: { b: <T>(x: T) => T } }>(1);",
    ] {
        silent(code);
    }
}

#[test]
fn tsx_function_expression_type_parameters_after_star_or_async() {
    for (code, want) in [
        (
            "export const foo = function* <T>() {};",
            "export const IDENT = function * < IDENT > ( ) { } ;",
        ),
        (
            "const h = async function* <T>() {};",
            "const IDENT = async function * < IDENT > ( ) { } ;",
        ),
        ("z = async function <T>() {};", "IDENT = async function < IDENT > ( ) { } ;"),
        ("x = function <T>() {};", "IDENT = function < IDENT > ( ) { } ;"),
        ("x = function* f<T>() {};", "IDENT = function * IDENT < IDENT > ( ) { } ;"),
        ("f(function* <T>(x: T) {});", "IDENT ( function * < IDENT > ( IDENT : IDENT ) { } ) ;"),
        (
            "x = function* <T>() { yield <T>(a)</T>; };",
            "IDENT = function * < IDENT > ( ) { yield JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END ; } ;",
        ),
        (
            "x = a * <T>(b)</T>;",
            "IDENT = IDENT * JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
        ),
    ] {
        stream(code, ScriptTSX, want);
    }
}

#[test]
fn tsx_call_and_construct_signatures_are_type_parameters() {
    for (code, want) in [
        ("type X = { <T>(x: T): U; }", "type IDENT = { < IDENT > ( IDENT : IDENT ) : IDENT ; }"),
        (
            "interface X { <T>(x: T): U; new <T>(x: T): U; m<T>(x: T): U; }",
            "interface IDENT { < IDENT > ( IDENT : IDENT ) : IDENT ; new < IDENT > ( IDENT : IDENT ) : IDENT ; IDENT < IDENT > ( IDENT : IDENT ) : IDENT ; }",
        ),
        ("let x: { <T>(x: T): U };", "let IDENT : { < IDENT > ( IDENT : IDENT ) : IDENT } ;"),
        (
            "type X = { a: T, <U>(x: U): V }",
            "type IDENT = { IDENT : IDENT , < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "type X = { a: T; <U>(x: U): V }",
            "type IDENT = { IDENT : IDENT ; < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "type X = { a: void\n <U>(x: U): V }",
            "type IDENT = { IDENT : void < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "type X = { a: {b: T}\n <U>(x: U): V }",
            "type IDENT = { IDENT : { IDENT : IDENT } < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "interface I extends A { <T>(x: T): U }",
            "interface IDENT extends IDENT { < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "type F = () => { <T>(x: T): U }",
            "type IDENT = ( ) => { < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "x = y as { <T>(x: T): U };",
            "IDENT = IDENT as { < IDENT > ( IDENT : IDENT ) : IDENT } ;",
        ),
        (
            "type X = [{ <T>(x: T): U }];",
            "type IDENT = [ { < IDENT > ( IDENT : IDENT ) : IDENT } ] ;",
        ),
        (
            "type X = A | { <T>(x: T): U };",
            "type IDENT = IDENT | { < IDENT > ( IDENT : IDENT ) : IDENT } ;",
        ),
        (
            "type X = { new <T>(x: T): U }",
            "type IDENT = { new < IDENT > ( IDENT : IDENT ) : IDENT }",
        ),
        (
            "declare function f(x: { <T>(x: T): U }): void;",
            "declare function IDENT ( IDENT : { < IDENT > ( IDENT : IDENT ) : IDENT } ) : void ;",
        ),
    ] {
        stream(code, ScriptTSX, want);
    }
    for code in [
        "if (a) { <T>(x)</T> }",
        "function f() { <T>(x)</T> }",
        "L: { <T>(x)</T> }",
        "switch (a) { case 1: { <T>(x)</T> } }",
        "x = () => { <T>(x)</T> };",
        "{ <T>(x)</T> }",
        "{ a, <T>(x)</T> }",
        "function f() { a; <T>(x)</T> }",
        "namespace N { <T>(x)</T> }",
        "try { <T>(x)</T> } finally {}",
        "class C { static { <T>(x)</T> } }",
        "class C { m() { <T>(x)</T> } }",
        "f(a, <T>(x)</T>);",
        "x = [a, <T>(x)</T>];",
        "if (x)\n<T>(y)</T>",
    ] {
        let ks = kinds_of(code, ScriptTSX);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        assert!(!ks.contains(&TokenKind::Lt), "{code:?} must open JSX: {ks:?}");
    }
}

#[test]
fn keyword_named_generic_members() {
    for (code, want) in [
        (
            "class C { delete<const U>(x: U): U {} }",
            "class IDENT { delete < const IDENT > ( IDENT : IDENT ) : IDENT { } }",
        ),
        (
            "class C { m() {} return<T>(x: T) {} }",
            "class IDENT { IDENT ( ) { } return < IDENT > ( IDENT : IDENT ) { } }",
        ),
        ("x = { typeof<T>(x: T) {} };", "IDENT = { typeof < IDENT > ( IDENT : IDENT ) { } } ;"),
        (
            "type X = { void<T>(x: T): U };",
            "type IDENT = { void < IDENT > ( IDENT : IDENT ) : IDENT } ;",
        ),
    ] {
        stream(code, ScriptTSX, want);
    }
    for code in [
        "{ delete <T>(x)</T> }",
        "function f() { a; typeof <T>(x)</T> }",
        "if (a) { void <T>(x)</T> }",
    ] {
        let ks = kinds_of(code, ScriptTSX);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    }
}

#[test]
fn constructor_type_parameter_annotations_are_type_regions() {
    let silent = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(codes.is_empty(), "{code:?}: {codes:?}");
        let ks = kinds_of(code, ScriptTSX);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
    };
    silent("declare function b(obj: new (bar: <T>() => U<T>, arr: T<U>) => unknown): void;");
    silent("declare function b(obj: abstract new (bar: <T>() => U) => unknown): void;");
    silent("let x: new (cb: <T>(x: T) => T) => U;");
    silent("(a, b = c): <T>(x: T) => U => 1;");
    silent("type X = { a: T; [k: string]: <a>(s) => T }");
    silent("class C extends obj<abstract new () => (<T>(a) => true)> {}");
    silent("x = f((a): <T>(x: T) => U => 1);");
    silent("let x: Foo<new ({ a, b }: <s>({ a }: {}, s) => string[]) => U>;");
    silent("let x: [c?: <arr>() => new () => false | Thenable, d];");
    silent("class C { set [fn](cb: <Props>() => T) {} }");
    silent("for (const k in (b: <obj>() => Record) => { bar; }) {}");
}

#[test]
fn optional_markers_are_not_ternaries_for_the_jsx_diagnostic() {
    let silent = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(codes.is_empty(), "{code:?}: {codes:?}");
        let ks = kinds_of(code, ScriptTSX);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
    };
    let diagnosed = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(
            codes.contains(&DiagCode::UnterminatedJsxElement),
            "{code:?} must be diagnosed: {codes:?}"
        );
    };
    for code in [
        "let x: A<(a?: <T>() => U) => V>;",
        "let x: A<(arr?: <cb>() => keyof bigint) => 'lit'>;",
        "let f: (a?: <T>() => U) => V;",
        "function f(a?: <T>() => U) {}",
        "let x: readonly [b?: <b>(cb, [a, b]: | Record<K>) => U];",
        "let x: [b?: <b>(cb) => U, c?: <d>() => V];",
        "class C { b?(): <N>(n: U) => V }",
        "class C { static b?(): <N>(n: U) => V }",
        "class C { [k]?(): <N>(n: U) => V }",
        "class C { 'k'?(): <N>(n: U) => V }",
        "class C { #p?(): <N>(n: U) => V }",
        "interface I { b?(): <N>(n: this[][], arr: U) => <fn>(s: T) => U }",
        "type X = { b?(): <N>(n: U) => V }",
        "let o: { b?(): <N>(n: U) => V };",
        "class C { a = 1; b?(): <N>(n: U) => V }",
        "class C { m?: <T>(x: T) => T }",
    ] {
        silent(code);
    }
    for code in [
        "x = c ? (a) : <T>(x: T) => x;",
        "{ c ? (a) : <T>(x: T) => x }",
        "f(c ? (a) : <T>(x: T) => x);",
        "x = c ? a : <T>(x: T) => x;",
    ] {
        diagnosed(code);
    }
}

#[test]
fn member_and_parameter_annotations_are_type_regions_for_the_jsx_diagnostic() {
    let silent = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(codes.is_empty(), "{code:?}: {codes:?}");
        let ks = kinds_of(code, ScriptTSX);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
    };
    let diagnosed = |code: &str| {
        let codes = diag_codes_of(code, ScriptTSX);
        assert!(
            codes.contains(&DiagCode::UnterminatedJsxElement),
            "{code:?} must be diagnosed: {codes:?}"
        );
    };
    for code in [
        "class C { a: T;\n  b: <y>() => U }",
        "interface I { a: T;\n  readonly b: <y>() => U }",
        "class C { a = 1; b!: <y>() => U }",
        "class C { #p!: <arr>(...x: T[]) => U }",
        "class C extends (e) { #p!: <arr>() => U }",
        "x = (class extends (e) { public b!: <bar>() => U });",
        "class C { @dec() b: <bar>() => U }",
        "class C { @dec @dec() override cb: <bar>() => K }",
        "class C { m() {} public s: <N>() => U }",
        "class C { static { x; } b: <y>() => U }",
        "class C { static [k]: <Foo>(bar) => P }",
        "class C { static [y.#p`{`]: <Foo>(bar) => P }",
        "class C { private static?: 'lit'; public out!: <arr>(s) => U }",
        "interface I { 'x': T; 'y': <y>() => U }",
        "interface I { c?(): T; b: <obj>(baz?: T) => K }",
        "type X = { a: T; [k: string]: T; b: <y>() => U }",
        "let x: new (a) => (s: T) => { _?(): <P>(b: T) => U };",
        "abstract class Foo<T = { s: P<false>, bar?(): {}, n?(): <_>() => this }> extends fn<A> {}",
        "function f<T = { n?(): <_>() => U }>() {}",
        "let x: Map<K, { n?(): <_>() => U }>;",
        "type X<T = { n?(): <_>() => U }> = T;",
        "x = function* (el, [b, c]: <T>(y: T) => U) {};",
        "x = function (el, [b, c]: <T>(y: T) => U) {};",
        "f(function* (x?: <T>() => U) {});",
        "x = (s?: <bar>(cb: T) => U) => 1;",
        "x = ({ a }, arr: <T>(obj: T) => U) => 1;",
        "f($ = ({ a }, arr: <T>(obj: T) => U) => 1);",
        "class C { m($ = ({ a }, arr: <T>(obj: T) => U) => 1) {} }",
        "[a, b] = ($, x, c: <T>(s: T) => U) => 1;",
        "(_?: <Foo>(...c: T[]) => U, ...n) => 1;",
        "x = (a: <T>() => U): R => 1;",
        "let x = async function <V, K = { 'obj': <x>(a: T) => U }>() {};",
        "class C { [a << b]() {}
 in<K>(el: T) {} }",
        "class C { [(b[class <U extends P<Q<R>>> {}] << (x)] ??= y)]() {}
 @dec() in<K>(el: T) {} }",
        "class C { x = a << b;
 m<K>(el: T) {} }",
        "let e: Map<<T>(x: T) => T, { m<K>(el: T): U }>;",
        "type X = Set<A, (B), (<arr>() => C)[]>;",
        "let arr: (c: keyof T<[Map], (<P>() => Q<M>), [bar?: K]>) => U;",
        "type X = { foo?: P; ([a, b]: T, obj: U, c: <B>([a, b]: S) => R): V }",
        "type X = { (c: <B>(a: S) => R): V }",
        "interface I { m(): T; (c: <B>(a: S) => R): V }",
        "type X = { a: T }\n{ (c: <B>(a: S) => R): V }",
        "type T = { new (b: X, s: <el>() => U): V }",
        "function* obj(a: T, b: <x>(fn?: U) => V) {}",
        "class C { foo(a = $, { s }, el: <x>(_: T) => U) {} }",
        "class C { #p([b, $]: <b>({ a }: T) => U) {} }",
        "class C { constructor(private a: T, obj: <c>() => U) {} }",
        "async function* el(fn, n: <N>({ a, b }: T, c: U) => V) {}",
        "let x: { obj([a, b]: <n>(o: T) => U): V };",
        "class C { @dec() *accessor(b: T = x++, s: <a>() => boolean): asserts x {} }",
        "x = class { constructor(p: any = f(), bar, x?: <s>(this: string) => P) {} };",
        "function f(a: T, ...b: <Foo>(c: U) => V[]) {}",
        "f(a, (s: <el>() => U) => 1);",
        "using a = f, o: <obj>() => U = g;",
        "let a = f, o: <obj>() => U;",
    ] {
        silent(code);
    }
    for code in [
        "{ a: 1; b: <T>(x: T) => x }",
        "function f() { x = 1; y: <T>(x: T) => x }",
        "L: <T>(x: T) => x;",
        "x = (a, b);\ny: <T>(x: T) => x;",
        "f(a, <T>(x: T) => x);",
        "new C(a, <T>(x: T) => x);",
        "f(a, b ? c : <T>(x: T) => x);",
    ] {
        diagnosed(code);
    }
}

#[test]
fn keyword_types_take_no_type_arguments() {
    // `any<z>` is `any` then a comparison: keyword types never take arguments.
    for kw in [
        "any",
        "unknown",
        "string",
        "number",
        "boolean",
        "symbol",
        "object",
        "never",
        "undefined",
        "null",
        "void",
        "bigint",
        "this",
        "true",
        "false",
    ] {
        gt_run_fused(&format!("x = y as {kw}<z>>w;"));
        gt_run_fused(&format!("x = y satisfies {kw} < z >> w;"));
    }
    gt_run_fused("let x: any = y as string < z >> w;");
    gt_run_split("x = y as A<z>>w;");
}

#[test]
fn function_head_name_after_a_line_break_keeps_its_return_type_list() {
    for code in [
        "function\nf<T>(): U<O<T>, C<T>> {}",
        "function //c\nf<T>(): U<O<T>> {}",
        "function /*\n*/ f<T>(): U<O<T>> {}",
        "export function\nf<T>(): U<O<T>> {}",
        "async function\nf(): Promise<Array<T>> {}",
        "class\nC<T> extends B<C<T>> {}",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn nested_type_reference_after_a_line_break_takes_no_arguments() {
    // A type reference takes no arguments across a line break, so the speculative list fails.
    for code in [
        "x = f<A\n<B>>(c);",
        "x = f<A\u{2028}<B>>(c);",
        "x = f<A /*\n*/ <B>>(c);",
        "x = f<A //c\n<B>>(c);",
        "x = f<A, B\n<C>>(d);",
        "x = new F<A\n<B>>(c);",
        "x = f<A.B\n<C>>(d);",
        "x = f<typeof a\n<C>>(d);",
    ] {
        gt_run_fused(code);
    }
    for code in [
        "x = f<A<\nB>>(c);",
        "x = f<\nA<B>>(c);",
        "x = f<A<B\n>>(c);",
        "x = f<A, /* c */ B<C>>(d);",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn tsx_type_parameter_list_signals_cross_trivia() {
    // A comment between the parameter name and its `,` / `=` / `extends` still marks a list;
    // an `extends` attribute followed by a comment still marks a tag.
    let tsx = |code: &str| kinds_of(code, ScriptTSX);
    for (code, plain) in [
        ("x = <T //c\nextends U>(a: T) => a;", "x = <T extends U>(a: T) => a;"),
        ("x = <T /*c*/ extends U>(a: T) => a;", "x = <T extends U>(a: T) => a;"),
        ("x = <T /*\n*/ extends U>(a: T) => a;", "x = <T extends U>(a: T) => a;"),
        ("x = <T //c\n,>(a: T) => a;", "x = <T,>(a: T) => a;"),
        ("x = <T /*c*/,>(a: T) => a;", "x = <T,>(a: T) => a;"),
        ("x = <T //c\n= U,>(a: T) => a;", "x = <T = U,>(a: T) => a;"),
        ("x = <const /*c*/ T extends U>(a: T) => a;", "x = <const T extends U>(a: T) => a;"),
        ("x = <T extends /*c*/ />;", "x = <T extends />;"),
        ("x = <T extends //c\n />;", "x = <T extends />;"),
    ] {
        assert_eq!(tsx(code), tsx(plain), "{code:?}");
        assert!(diag_codes_of(code, ScriptTSX).is_empty(), "{code:?}");
    }
}

#[test]
fn tsx_const_followed_by_a_line_break_is_a_jsx_tag_name() {
    // `const` is only a modifier on the same line as its parameter.
    for code in [
        "x = <const\nextends U>hi</const>;",
        "x = <const\nT extends U>hi</const>;",
        "x = <const /*\n*/ extends U>hi</const>;",
    ] {
        assert!(kinds_of(code, ScriptTSX).contains(&TokenKind::JsxLt), "{code:?}");
        assert!(diag_codes_of(code, ScriptTSX).is_empty(), "{code:?}");
    }
}

#[test]
fn keyword_type_followed_by_a_dot_is_a_member_access() {
    // `this`, `null`, `true`, `false` and `void` are whole types: `this.x` is not a type, so the
    // speculative list fails and the run is a shift. `any.x` and `string.x` are qualified names.
    for kw in ["this", "null", "true", "false", "void"] {
        gt_run_fused(&format!("x = a<b<{kw}.y>>(1);"));
    }
    for kw in ["any", "string", "undefined"] {
        gt_run_split(&format!("x = a<b<{kw}.y>>(1);"));
    }
    gt_run_split("x = a<b<this>>(1);");
    gt_run_split("x = a<b<this[1]>>(1);");
    gt_run_split("x = a<b<typeof this.y>>(1);");
}

#[test]
fn super_is_a_type_only_in_a_type_query() {
    gt_run_split("x = a<b<typeof super.y>>(1);");
    gt_run_split("x = a<b<typeof super>>(1);");
    gt_run_fused("x = a<b<super.y>>(1);");
    // A type query names a value, which takes type arguments even when it spells a keyword type.
    gt_run_split("x = c<Map<typeof this<A, 1>>>(1);");
    gt_run_fused("x = c<Map<this<A, 1>>>(1);");
}

#[test]
fn keywords_as_names_inside_a_type_list() {
    // `let` is an identifier in a type; a reserved word is refused only where a list element
    // starts (tsc's `isStartOfType`), and read as a name elsewhere: a property, a reference
    // after `=>`.
    gt_run_split("x = f<A<(let: T) => U>>(1);");
    gt_run_split("x = f<A<let>>(1);");
    gt_run_split("x = f<A<{ return: T; class?: U }>>(1);");
    gt_run_split("x = f<A<(x: T) => return>>(1);");
    gt_run_fused("x = f<A<return>>(1);");
    gt_run_fused("x = f<A<B, return>>(1);");
}

#[test]
fn escaped_identifier_follower_starts_an_expression() {
    // An identifier written with a Unicode escape follows a `>` run like any other name.
    gt_run_fused(r"x = f<T<U>>\u0061;");
    gt_run_fused(r"x = f<T<U>> \u{61};");
    gt_run_split("x = f<T<U>>\n\\u0061;");
}

#[test]
fn line_break_before_extends_inside_a_type_parameter_list() {
    // Inside a `<...>` list a line break is trivia, so no statement ends there. The template
    // literal type keeps the run from being settled without a walk.
    for code in [
        "f = <T\nextends Replace<A, `{${string}}`, B>>(x: T) => 1;",
        "f = <T\nextends Replace<A, B>>(x: T) => 1;",
        "f = <T\nextends A<B>>(x: T) => 1;",
        "f = <T /*\n*/ extends Replace<A, `{${string}}`, B>>(x: T) => 1;",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn type_argument_lists_longer_than_the_bounded_scan() {
    // Past FORWARD_SCAN_CAP the closer comes from the memoized pass.
    let members: String = (0..400).map(|i| format!("a{i}: string; ")).collect();
    let big = format!("{{ {members}}}");
    assert!(big.len() > FORWARD_SCAN_CAP);
    gt_run_split(&format!("f<A<{big}>>(x);"));
    gt_run_split(&format!("new F<A<{big}>>();"));
    gt_run_split(&format!("a?.f<A<{big}>>();"));
    gt_run_split(&format!("f<A<{big}>>`t`;"));
    gt_run_split(&format!("x = f<A<{big}>>;"));
    division(&format!("x = f<{big}> / 2 / 1;"), ScriptTS);
    let values: String = (0..400).map(|i| format!("a{i}: 1 + 1, ")).collect();
    gt_run_fused(&format!("x = a < b < {{ {values}}} >> c;"));
}

#[test]
fn lt_lt_openers_around_lists_longer_than_the_bounded_scan() {
    let members: String = (0..6000).map(|i| format!("a{i}: string; ")).collect();
    let big = format!("{{ {members}}}");
    for code in [
        format!("let x: Array<<T>(x: T) => {big}> = y;"),
        format!("let x: Array<<T extends {big}>(x: T) => T> = y;"),
    ] {
        let ks = kinds_of(&code, ScriptTS);
        assert!(!ks.contains(&TokenKind::LShift), "{} tokens", ks.len());
    }
}

#[test]
fn lt_lt_openers_around_a_parameter_list_longer_than_the_bounded_scan() {
    let members: String = (0..6000).map(|i| format!("a{i}: string; ")).collect();
    let code = format!("let x: Array<<T>(a: {{ {members}}}) => T> = y;");
    let ks = kinds_of(&code, ScriptTS);
    assert!(!ks.contains(&TokenKind::LShift), "{} tokens", ks.len());
}

#[test]
fn tsx_function_type_with_trivia_before_its_parameters() {
    let plain = kinds_of("let f: <T> (x: T) => T;", ScriptTSX);
    for code in [
        "let f: <T> /*c*/ (x: T) => T;",
        "let f: <T> // c\n(x: T) => T;",
        "let f: <T>\u{a0}(x: T) => T;",
    ] {
        assert_eq!(kinds_of(code, ScriptTSX), plain, "{code:?}");
    }
}

#[test]
fn jsx_element_type_arguments_with_deeply_nested_template_types() {
    let deep = format!("{}T{}", "`${".repeat(33), "}`".repeat(33));
    let code = format!("x = <Foo<{deep}> />;");
    assert!(diag_codes_of(&code, ScriptTSX).is_empty(), "{code}");
    let ks = kinds_of(&code, ScriptTSX);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
    assert!(ks.contains(&TokenKind::JsxTagEnd), "{ks:?}");
}

#[test]
fn tsx_template_type_with_deeply_nested_substitutions_in_expression_type_arguments() {
    let deep = format!("{}U{}", "`${".repeat(10), "}`".repeat(10));
    let code = format!("x = f<`${{<T>(x: T) => {deep}}}`>(1);");
    assert!(diag_codes_of(&code, ScriptTSX).is_empty(), "{code}");
}
