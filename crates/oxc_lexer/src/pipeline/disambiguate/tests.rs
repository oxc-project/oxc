use crate::{Lexer, PAD, error::diag_code, options::default_options, token::TokenKind};

pub(super) fn kinds_of(code: &str, ts: bool, jsx: bool) -> Vec<TokenKind> {
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

fn first_slash_kind(code: &str, ts: bool) -> Option<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.ts = ts;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    let kinds = lx.kinds()[..count].to_vec();
    (0..count)
        .filter(|&i| !kinds[i].is_trivia() && buf[lx.spans[i].start as usize] == b'/')
        .map(|i| kinds[i])
        .next()
}

#[track_caller]
pub(super) fn regex(code: &str, ts: bool) {
    let ks = kinds_of(code, ts, false);
    assert_eq!(
        first_slash_kind(code, ts),
        Some(TokenKind::RegExp),
        "expected the first `/` to open a regex in {code:?}: kinds {ks:?}"
    );
}

#[track_caller]
pub(super) fn division(code: &str, ts: bool) {
    let ks = kinds_of(code, ts, false);
    assert!(!ks.contains(&TokenKind::RegExp), "expected division in {code:?}: kinds {ks:?}");
    assert!(ks.contains(&TokenKind::Slash), "expected a `/` in {code:?}: kinds {ks:?}");
}

#[test]
fn jsx_replay_oracle() {
    let jsx = |code: &str| kinds_of(code, false, true);
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
fn tsx_generic_function_type_in_type_position() {
    let tsx = |code: &str| kinds_of(code, true, true);
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
    let tsx = |code: &str| kinds_of(code, true, true);
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
    let tsx = |code: &str| kinds_of(code, true, true);
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
fn long_walks_resolve_exactly() {
    let spine = ["T"; 300].join(" & ");
    regex(&format!("let x: {spine}\n/re/g.exec(s);"), true);
    let body: String = (0..400).map(|i| format!("  a{i}: T;\n")).collect();
    regex(&format!("let x: {{\n{body}}} & U\n/re/g.exec(s);"), true);
    let args: String = (0..3000).map(|i| format!("a{i}, ")).collect();
    division(&format!("f({args}0) / 2"), false);
    regex(&format!("if ({args}0) /re/.test(s)"), false);
    let deep = 2000usize;
    let nested = format!("{}x{}", "[".repeat(deep), "]".repeat(deep));
    division(&format!("y = {nested} / 2"), false);
    let blocks = format!("{}x\n{}", "{".repeat(deep), "}\n/a/\n".repeat(deep));
    let ks = kinds_of(&blocks, false, false);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RegExp).count(), deep, "{}", ks.len());
    let heritage: String = (0..100).map(|i| format!("I{i}, ")).collect();
    division(&format!("x = class extends A implements {heritage}J {{}} / 2"), true);
    let members: String = (0..2000).map(|i| format!("  m{i}(): Foo<Bar<T>> {{}}\n")).collect();
    let ks = kinds_of(&format!("class C {{\n{members}  m<T = A<B>>() {{}}\n}}"), true, false);
    assert!(
        !ks.iter().any(|k| matches!(k, TokenKind::RShift | TokenKind::URShift)),
        "{}",
        ks.len()
    );
}

#[test]
fn jsx_tag_name_after_comment() {
    let tsx = |code: &str| kinds_of(code, true, true);
    for code in [
        "var x = </**/div></div>;",
        "var x = < /*a*/ ></ /*b*/>;",
        "var x = <//c\ndiv></div>;",
        "var x = < /*a*/ div /*b*/ />;",
        "var x = <></>;",
        "var x = < ></ >;",
        "var x = <></ /*b*/>;",
        "var x = <div></ /*b*/ div>;",
        "var x = <div></div /*b*/>;",
        "var x = <div></ //c\ndiv>;",
    ] {
        let ks = tsx(code);
        assert!(!ks.contains(&TokenKind::RegExp), "{code:?} must not invent a regex: {ks:?}");
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    }
    let ks = tsx("var x = a < b / c;");
    assert!(!ks.contains(&TokenKind::JsxLt), "comparison must stay relational: {ks:?}");
}

#[test]
fn unicode_zero_width_space_is_whitespace() {
    let ks = kinds_of("x\u{200b}\ny", false, false);
    assert_eq!(
        ks.iter().filter(|&&k| k == TokenKind::Ident).count(),
        2,
        "U+200B must separate tokens: {ks:?}"
    );
    division("let a = 1;\u{200b}let b = 2 / 3;", false);
}

#[test]
fn unicode_next_line_is_whitespace() {
    division("var a = 1;\u{85}var b = 2 / 3;", false);
    let ks = kinds_of("var a = 1;\u{85}var b = 2;", false, false);
    assert_eq!(
        ks.iter().filter(|&&k| k == TokenKind::KwVar).count(),
        2,
        "U+0085 must separate tokens: {ks:?}"
    );
}

#[test]
fn jsx_self_close_allows_whitespace() {
    let tsx = |code: &str| kinds_of(code, false, true);
    for code in [
        "const a = <N x=\"v\"/>;\nconst b = 1;",
        "const a = <N x=\"v\" / >;\nconst b = 1;",
        "const a = <N x=\"v\" /\n>;\nconst b = 1;",
        "const a = <N x=\"v\"\t/\t>;\nconst b = 1;",
    ] {
        let ks = tsx(code);
        assert!(ks.contains(&TokenKind::JsxTagEnd), "{code:?} must self-close: kinds {ks:?}");
        assert!(
            ks.iter().filter(|&&k| k == TokenKind::KwConst).count() == 2,
            "{code:?} must not swallow the next statement: kinds {ks:?}"
        );
    }
    let ks = tsx("const a = <N x=\"v\" / y>;");
    assert!(!ks.contains(&TokenKind::JsxTagEnd), "lone slash: kinds {ks:?}");
}

pub(super) fn diag_codes_of(code: &str, ts: bool, jsx: bool) -> Vec<u16> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.ts = ts;
    opts.jsx = jsx;
    let mut lx = Lexer::new();
    lx.lex(&buf, n, opts);
    lx.lanes.diags.iter().map(|d| d.code).collect()
}

fn is_fused_gt(k: TokenKind) -> bool {
    matches!(k, TokenKind::RShift | TokenKind::URShift | TokenKind::RShiftEq | TokenKind::URShiftEq)
}

#[track_caller]
fn gt_run_fused(code: &str) {
    let ks = kinds_of(code, true, false);
    assert!(ks.iter().any(|k| is_fused_gt(*k)), "{code:?} must fuse the `>` run: {ks:?}");
}

#[track_caller]
fn gt_run_split(code: &str) {
    let ks = kinds_of(code, true, false);
    assert!(
        !ks.iter().any(|k| is_fused_gt(*k) || *k == TokenKind::Ge),
        "{code:?} must split the `>` run: {ks:?}"
    );
}

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
    let ts = |code: &str| kinds_of(code, true, false);
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
        let ks = kinds_of(code, true, false);
        assert!(
            !ks.iter().any(|k| matches!(k, TokenKind::LShift | TokenKind::RShift)),
            "{code:?}: {ks:?}"
        );
    }
}

#[test]
fn tsx_generic_function_type_is_decided_by_its_arrow() {
    let jsx_count =
        |code: &str| kinds_of(code, true, true).iter().filter(|k| **k == TokenKind::JsxLt).count();
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
        let codes = diag_codes_of(code, true, true);
        assert!(
            codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
            "{code:?} must be diagnosed: {codes:?}"
        );
        let ks = kinds_of(code, true, true);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?} lexes as type parameters: {ks:?}");
    };
    let silent = |code: &str| {
        let codes = diag_codes_of(code, true, true);
        assert!(
            !codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
            "{code:?} is valid: {codes:?}"
        );
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
    let ks = kinds_of("f(x => { const a: A<B<C>>[] = []; }, { a: b<c<d>>[0] });", true, false);
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
    let ks = kinds_of("let x = f<A<B<C>>>\n<div/>;", true, true);
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
    let ks = kinds_of("let x: Map<K, V<W<T>>>\n<div/>;", true, true);
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
    let ts = |code: &str| kinds_of(code, true, false);
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
    let ks = kinds_of("x = a >>ete<this<1n, Map>>(arr);", true, false);
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
fn jsx_element_type_argument_runs_never_reach_coalesce() {
    for code in [
        "const d = <div<A<B>>>x</div>;",
        "const d = <div<Missing<AlsoMissing>>></div>;",
        "const e = <Link<React.ImgHTMLAttributes<HTMLElement>> element=\"img\" src=\"src\" />;",
        "const f = <Foo<A<B>> bar={a >> b} />;",
        "x = <Foo<A<B>>/>\ny = a >> (b);",
    ] {
        let ks = kinds_of(code, true, true);
        assert!(!ks.iter().any(|k| is_fused_gt(*k)) || code.contains(">> "), "{code:?}: {ks:?}");
        assert!(!ks.contains(&TokenKind::LShift), "{code:?}: {ks:?}");
    }
    let ks = kinds_of("const d = <div<A<B>>>x</div>;", true, true);
    assert!(ks.contains(&TokenKind::JsxText), "{ks:?}");
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 3, "{ks:?}");
    let ks = kinds_of("const f = <Foo<A<B>> bar={a >> b} />;", true, true);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
    let ks = kinds_of("x = <Foo<A<B>>/>\ny = a >> (b);", true, true);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
}

#[test]
fn tsx_element_type_arguments_with_generic_function_type() {
    let ks = kinds_of("<Component<<T>(v: T) => void> />", true, true);
    assert!(!ks.contains(&TokenKind::LShift), "{ks:?}");
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
    let ks = kinds_of("const a = <Box<(x: T) => Foo<T>> prop={1} />;", true, true);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
}

fn names_of(code: &str, ts: bool, jsx: bool) -> String {
    kinds_of(code, ts, jsx)
        .iter()
        .map(|k| match k {
            TokenKind::Number
            | TokenKind::Decimal
            | TokenKind::Float
            | TokenKind::Binary
            | TokenKind::Octal
            | TokenKind::Hex => "NUMBER",
            TokenKind::StringCooked => "STRING",
            TokenKind::IdentEscaped => "IDENT",
            _ => k.name(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn spans_of(code: &str, ts: bool, jsx: bool) -> Vec<(u32, u32)> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.ts = ts;
    opts.jsx = jsx;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    let kinds = lx.kinds()[..count].to_vec();
    (0..count)
        .filter(|&i| !kinds[i].is_trivia())
        .map(|i| (lx.spans[i].start, lx.spans[i].end))
        .collect()
}

#[track_caller]
pub(super) fn stream(code: &str, ts: bool, jsx: bool, want: &str) {
    assert_eq!(names_of(code, ts, jsx), want, "{code:?}");
}

#[test]
fn jsx_element_as_attribute_value_opens_a_nested_frame() {
    for (code, want) in [
        (
            "<App foo=<div>bar</div> />;",
            "JSX_LT IDENT IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END / JSX_TAG_END ;",
        ),
        (
            "const a = <Foo key=<T></T>>{x}\n  <b />\n</Foo>;",
            "const IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT > JSX_LT / IDENT JSX_TAG_END > { IDENT } JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "const a = <Foo key=<T/> other=\"x\">{x}</Foo>;",
            "const IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END IDENT = STRING > { IDENT } JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "x = <a b=<c>{d}</c>>{e}<f/></a>;",
            "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT > { IDENT } JSX_LT / IDENT JSX_TAG_END > { IDENT } JSX_LT IDENT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "const a = <Foo prop=<Bar><Baz /></Bar> />;",
            "const IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT > JSX_LT IDENT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END / JSX_TAG_END ;",
        ),
        (
            "x = <a b=<>t</> />;",
            "IDENT = JSX_LT IDENT IDENT = JSX_LT > JSX_TEXT JSX_LT / JSX_TAG_END / JSX_TAG_END ;",
        ),
        (
            "x = <a b= /* c */ <i/> d=\"1\"/>;",
            "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END IDENT = STRING / JSX_TAG_END ;",
        ),
        (
            "x = <a b=\n<i/>\n/>;",
            "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END / JSX_TAG_END ;",
        ),
        (
            "x = <a b=<c d=<e/>/>/>;",
            "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END / JSX_TAG_END / JSX_TAG_END ;",
        ),
        (
            "x = <a b=<c/>/>\ny = z / 2;",
            "IDENT = JSX_LT IDENT IDENT = JSX_LT IDENT / JSX_TAG_END / JSX_TAG_END IDENT = IDENT / NUMBER ;",
        ),
        (
            "x = <a b=\"<\" c={1 < 2} />;",
            "IDENT = JSX_LT IDENT IDENT = STRING IDENT = { NUMBER < NUMBER } / JSX_TAG_END ;",
        ),
    ] {
        stream(code, false, true, want);
    }
    stream(
        "const a = <Foo<T> key=<Bar<U> x=\"1\"/> />;",
        true,
        true,
        "const IDENT = JSX_LT IDENT < IDENT > IDENT = JSX_LT IDENT < IDENT > IDENT = STRING / JSX_TAG_END / JSX_TAG_END ;",
    );
}

#[test]
fn jsx_child_element_name_after_trivia() {
    stream(
        "const a = <div>\n  x<br />\n  < br />\n  y\n</div>;",
        true,
        true,
        "const IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
    );
    for (code, want) in [
        (
            "x = <a>\n  <b/>\n  < c />\n  <d\n  />\n</a>;",
            "IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT IDENT / JSX_TAG_END JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;",
        ),
        ("x = <a>< /a>;", "IDENT = JSX_LT IDENT > JSX_LT / IDENT JSX_TAG_END ;"),
        (
            "x = <a>< /* c */ b/></a>;",
            "IDENT = JSX_LT IDENT > JSX_LT IDENT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "x = <a>< ></ ></a>;",
            "IDENT = JSX_LT IDENT > JSX_LT > JSX_LT / JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
        ),
        (
            "x = <a>t< b>u</ b ></a>;",
            "IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END JSX_LT / IDENT JSX_TAG_END ;",
        ),
    ] {
        stream(code, false, true, want);
    }
}

#[test]
fn jsx_names_glue_every_hyphen() {
    for (code, want) in [
        ("x = <a--b/>;", "IDENT = JSX_LT IDENT / JSX_TAG_END ;"),
        ("y = <a-/>;", "IDENT = JSX_LT IDENT / JSX_TAG_END ;"),
        (
            "z = <a b-c-=\"1\" d--e=\"2\"/>;",
            "IDENT = JSX_LT IDENT IDENT = STRING IDENT = STRING / JSX_TAG_END ;",
        ),
        ("<div-\n    // comment\n/>;", "JSX_LT IDENT / JSX_TAG_END ;"),
        ("x = <a--b>t</a--b>;", "IDENT = JSX_LT IDENT > JSX_TEXT JSX_LT / IDENT JSX_TAG_END ;"),
        (
            "x = <a b={c - d} e={-1}/>;",
            "IDENT = JSX_LT IDENT IDENT = { IDENT - IDENT } IDENT = { - NUMBER } / JSX_TAG_END ;",
        ),
        ("x = <a-b:c-d/>;", "IDENT = JSX_LT IDENT : IDENT / JSX_TAG_END ;"),
    ] {
        stream(code, false, true, want);
    }
    assert_eq!(spans_of("x = <a--b/>;", false, true)[3], (5, 9));
    assert_eq!(spans_of("y = <a-/>;", false, true)[3], (5, 7));
    stream(
        "<Foo<-1> data-x=\"1\"/>;",
        true,
        true,
        "JSX_LT IDENT < - NUMBER > IDENT = STRING / JSX_TAG_END ;",
    );
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
        stream(code, true, true, want);
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
        stream(code, true, true, want);
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
        let ks = kinds_of(code, true, true);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
        assert!(!ks.contains(&TokenKind::Lt), "{code:?} must open JSX: {ks:?}");
    }
}

#[test]
fn escaped_type_parameter_name_after_lt_lt() {
    stream(
        "let s: a<<\\u{62}c>(x: T) => T>;",
        true,
        false,
        "let IDENT : IDENT < < IDENT > ( IDENT : IDENT ) => IDENT > ;",
    );
    stream(
        "const r = f<<\\u{62}c>(a: \\u{62}c) => \\u{62}c>(y);",
        true,
        false,
        "const IDENT = IDENT < < IDENT > ( IDENT : IDENT ) => IDENT > ( IDENT ) ;",
    );
}

#[test]
fn escaped_identifiers_are_identifiers_in_every_walk() {
    regex("let \\u{62}c\n/re/.x;", false);
    regex("let \\u{62}c: T\n/re/.x;", true);
    regex("declare const \\u{62}c: Set<T>\n/re/.x;", true);
    regex("var a = 1, \\u{62}c\n/re/.x;", false);
    regex("for (;;) { break \\u{6f}uter\n/re/.test(b); }", false);
    regex("type \\u{41} = T\n/re/.exec(s);", true);
    regex("declare function \\u{66}(): T\n/re/.exec(s);", true);
    division("x = \\u{62}c\n/re/g.exec(s);", false);
    division("\\u{62}c / 2;", false);
    division("x.\\u{62}c / 2;", false);
    gt_run_split("type \\u{41} = Foo<Bar<T>>[];");
    gt_run_split("let \\u{62}c: Foo<Bar<T>>[] = y;");
}

#[test]
fn legacy_octal_literal_ends_before_a_dot() {
    for (code, want) in [
        ("x = 010.5;", "IDENT = NUMBER NUMBER ;"),
        ("x = 010.toString();", "IDENT = NUMBER . IDENT ( ) ;"),
        ("x = 00.5;", "IDENT = NUMBER NUMBER ;"),
        ("x = 07.;", "IDENT = NUMBER . ;"),
        ("x = 08.5;", "IDENT = NUMBER ;"),
        ("x = 019.5;", "IDENT = NUMBER ;"),
        ("x = 09e1;", "IDENT = NUMBER ;"),
        ("x = 0.5;", "IDENT = NUMBER ;"),
        ("x = 0x10.5;", "IDENT = NUMBER NUMBER ;"),
        ("x = 010n;", "IDENT = NUMBER IDENT ;"),
        ("x = 08n;", "IDENT = NUMBER IDENT ;"),
    ] {
        stream(code, false, false, want);
    }
    assert_eq!(spans_of("x = 010.5;", false, false)[2], (4, 7));
    assert_eq!(spans_of("x = 010.5;", false, false)[3], (7, 9));
}

#[test]
fn type_context_before_a_declaration_keyword() {
    gt_run_split("x = () => {}\n<Map<P>>baz;");
    gt_run_split("x = async () => {}\n<Map<P>>baz;");
    regex("declare function f(): Foo<T>\nclass C {}\n/re/.test(s);", true);
    regex("let x: Foo<T>\nfunction f() {}\n/re/.test(s);", true);
    division("x = <T>\nfunction(){} / 2;", true);
    stream(
        "class C<T> extends B implements I, void {}\n/=/.test(s);",
        true,
        false,
        "class IDENT < IDENT > extends IDENT implements IDENT , void { } REGEXP . IDENT ( IDENT ) ;",
    );
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
        stream(code, true, true, want);
    }
    for code in [
        "{ delete <T>(x)</T> }",
        "function f() { a; typeof <T>(x)</T> }",
        "if (a) { void <T>(x)</T> }",
    ] {
        let ks = kinds_of(code, true, true);
        assert!(ks.contains(&TokenKind::JsxLt), "{code:?} must open JSX: {ks:?}");
    }
}

#[test]
fn template_literal_type_inside_a_lt_lt_list() {
    for code in [
        "(a?.c)<<n>() => T<F<G<\"lit\">>>, readonly R<0x1f, Foo>[], `${Foo<Map<R>>}`,>(y);",
        "let e: Map<<T>(x: T) => `${T}`, U>;",
    ] {
        let ks = kinds_of(code, true, false);
        assert!(!ks.contains(&TokenKind::LShift), "{code:?}: {ks:?}");
    }
}

#[test]
fn gt_run_heads_after_asi_and_this() {
    gt_run_split("let cb\n<Promise<V<Promise<-1>>>>cb;");
    gt_run_split("declare const Foo: `x${`y${Record<A<B>>}`}`\n<Array<Thenable<T>>>a;");
    gt_run_split(
        "(f() satisfies Thenable<`lit`> extends 0x1f ? keyof (X) : U<Set<Record<V<X<Map<X<K<1.5>>, Foo>>>>, Foo>> > el);",
    );
    gt_run_fused("x = y satisfies this<$JSX<-1>>\n[class <K, V> extends (a) {}];");
    gt_run_fused("x = a[0].b<c<d>>[0];");
    let ks = kinds_of("x = [f].#p <\n{ _ }| 's'>= b;", true, false);
    assert!(ks.contains(&TokenKind::Ge), "{ks:?}");
    regex("type Foo = | {} | bigint\n<import('m').baz<Bar<T<P>>>>/'/.x;", true);
    regex("let x: T\n<U>/re/.test(s);", true);
    division("let x = y\n<T>/re/.source;", true);
}

#[test]
fn relational_heads_before_a_balanced_run() {
    gt_run_split("x = { a: 1, ...<T<U<V>>>[baz] };");
    gt_run_split("f(...<Thenable<K>>[], false);");
    gt_run_split("x = c ? function* (...a): Pick<E<F<G>>>[][] {} : null;");
    gt_run_fused("x = (y)--<z ? 1 : $<A, B<C>>().p>> w;");
    gt_run_fused("x = y satisfies Exclude<never>\n<typeof obj | Exclude<K<'lit'>>>(baz);");
    gt_run_fused("x = y as Pick | this<U<V<W>>>\n/'/.x;");
    gt_run_fused("const x = Foo >>> (n)`\n`\n<U<Bar>>foo;");
    gt_run_split("f(this<A<B>>(x));");
    gt_run_split("return this<A<B>>(x);");
    gt_run_split("interface x extends this<Thenable<Record<Record>>>, U {}");
    gt_run_split("class C<V, T> implements this<Array<undefined>>, T {}");
    gt_run_split("class C implements this<Map<\"lit\">>, this {}");
    regex(
        "let c: T = y satisfies A<B<C>> extends infer U ? U : import('m').a<(V<never>)[Set]>\nnamespace Foo { class X {} }\n/foo/.exec(s);",
        true,
    );
    gt_run_fused("x = y satisfies A\n<Set<K>, Partial<U>>ab;");
    gt_run_split("x = y as A<B<C>> as D;");
    gt_run_fused("x = b()!\n<Array<Set<unknown>>>Foo;");
    gt_run_fused("x = c! < 0.5>>> a;");
    division("export default { a: 1 } / obj(x);", false);
    regex("x = y as Pick | this<U<V<W>>>\n/'/.x;", true);
    regex("x = y satisfies this<U<V<W>>>\n/'/.x;", true);
    division("x = this<A<B>> / 2;", true);
    regex("x ? a : [b][c]()\n{}\n/y/.exec(s);", false);
    regex("f(class { accessor x = y })\n{ }\n/</.test(s);", false);
    let ks = kinds_of("x = a > b<c<d>>>>(e);", true, false);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 3, "{ks:?}");
    assert!(ks.contains(&TokenKind::RShift), "{ks:?}");
}

#[test]
fn constructor_type_parameter_annotations_are_type_regions() {
    let silent = |code: &str| {
        let codes = diag_codes_of(code, true, true);
        assert!(codes.is_empty(), "{code:?}: {codes:?}");
        let ks = kinds_of(code, true, true);
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
        let codes = diag_codes_of(code, true, true);
        assert!(codes.is_empty(), "{code:?}: {codes:?}");
        let ks = kinds_of(code, true, true);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
    };
    let diagnosed = |code: &str| {
        let codes = diag_codes_of(code, true, true);
        assert!(
            codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
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
        let codes = diag_codes_of(code, true, true);
        assert!(codes.is_empty(), "{code:?}: {codes:?}");
        let ks = kinds_of(code, true, true);
        assert!(!ks.contains(&TokenKind::JsxLt), "{code:?}: {ks:?}");
    };
    let diagnosed = |code: &str| {
        let codes = diag_codes_of(code, true, true);
        assert!(
            codes.contains(&diag_code::UNTERMINATED_JSX_ELEMENT),
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
fn replay_hops_return_types_and_type_parameters() {
    regex("x = async (): T => { await /re/; };", true);
    regex("x = async (): typeof cb => { await /re/; };", true);
    regex("var $: <baz>() => 1n | T = async (): typeof cb => { await /<div>/ };", true);
    division("x = (): T => { var await = 1; return await /2/g; };", true);
    regex("x = async function f(): T { await /re/; };", true);
    regex("x = async function (): Promise<T> { await /re/; };", true);
    regex("x = async (): Promise<T> => { await /re/; };", true);
    regex("x = function* <T>(): Generator<T> { yield /re/; };", true);
    regex("class C { async m(): Promise<T> { await /re/; } }", true);
    regex("switch (async function f(): typeof import('m') { await /}/; }) {}", true);
    division("x = function (): T { var await = 1; return await /2/g; };", true);
    division("x = (): T => { var await = 1; return await /2/g; };", true);
}
