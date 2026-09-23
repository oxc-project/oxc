use crate::{LexOptions, Lexer, PAD, error::DiagCode, token::TokenKind};

/// Minimal version of `SourceType` just for tests.
///
/// Only includes variants which tests actually use.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FileType {
    ScriptJS,
    ScriptTS,
    ScriptJSX,
    ScriptTSX,
    ModuleJS,
}

impl FileType {
    /// Check if this [`FileType`] is TS or TSX.
    pub const fn is_ts(self) -> bool {
        matches!(self, Self::ScriptTS | Self::ScriptTSX)
    }

    /// Check if this [`FileType`] is JSX or TSX.
    pub const fn is_jsx(self) -> bool {
        matches!(self, Self::ScriptJSX | Self::ScriptTSX)
    }

    /// Check if this [`FileType`] is an ESM module.
    pub const fn is_module(self) -> bool {
        matches!(self, Self::ModuleJS)
    }

    /// Create [`LexOptions`] for lexing this file type.
    pub fn options(self) -> LexOptions {
        LexOptions {
            source_type_module: self.is_module(),
            jsx: self.is_jsx(),
            ts: self.is_ts(),
            ..Default::default()
        }
    }
}

// Reduce repeated boilerplate in tests below.
// Can reference `ScriptJS` directly, instead of `FileType::ScriptJS`.
use FileType::*;

pub(super) fn kinds_of(code: &str, file_type: FileType) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, file_type.options());
    lx.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

/// Assert that the token stream for `code` contains at least one `RegExp` token
/// and does not contain any `Slash` or `SlashEq` tokens.
#[track_caller]
pub(super) fn regex(code: &str, file_type: FileType) {
    let ks = kinds_of(code, file_type);
    assert!(
        !ks.iter().any(|kind| matches!(kind, TokenKind::Slash | TokenKind::SlashEq)),
        "expected no division in {code:?}: kinds {ks:?}"
    );
    assert!(ks.contains(&TokenKind::RegExp), "expected a regex in {code:?}: kinds {ks:?}");
}

/// Assert that the token stream for `code` contains at least one `Slash` token
/// and does not contain any `RegExp` tokens.
#[track_caller]
pub(super) fn division(code: &str, file_type: FileType) {
    let ks = kinds_of(code, file_type);
    assert!(!ks.contains(&TokenKind::RegExp), "expected division in {code:?}: kinds {ks:?}");
    assert!(ks.contains(&TokenKind::Slash), "expected a `/` in {code:?}: kinds {ks:?}");
}

#[test]
fn long_walks_resolve_exactly() {
    let spine = ["T"; 300].join(" & ");
    regex(&format!("let x: {spine}\n/re/g.exec(s);"), ScriptTS);
    let body: String = (0..400).map(|i| format!("  a{i}: T;\n")).collect();
    regex(&format!("let x: {{\n{body}}} & U\n/re/g.exec(s);"), ScriptTS);
    let args: String = (0..3000).map(|i| format!("a{i}, ")).collect();
    division(&format!("f({args}0) / 2"), ScriptJS);
    regex(&format!("if ({args}0) /re/.test(s)"), ScriptJS);
    let deep = 2000usize;
    let nested = format!("{}x{}", "[".repeat(deep), "]".repeat(deep));
    division(&format!("y = {nested} / 2"), ScriptJS);
    let blocks = format!("{}x\n{}", "{".repeat(deep), "}\n/a/\n".repeat(deep));
    let ks = kinds_of(&blocks, ScriptJS);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RegExp).count(), deep, "{}", ks.len());
    let heritage: String = (0..100).map(|i| format!("I{i}, ")).collect();
    division(&format!("x = class extends A implements {heritage}J {{}} / 2"), ScriptTS);
    let members: String = (0..2000).map(|i| format!("  m{i}(): Foo<Bar<T>> {{}}\n")).collect();
    let ks = kinds_of(&format!("class C {{\n{members}  m<T = A<B>>() {{}}\n}}"), ScriptTS);
    assert!(
        !ks.iter().any(|k| matches!(k, TokenKind::RShift | TokenKind::URShift)),
        "{}",
        ks.len()
    );
}

#[test]
fn jsx_tag_name_after_comment() {
    let tsx = |code: &str| kinds_of(code, ScriptTSX);
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
    let ks = kinds_of("x\u{200b}\ny", ScriptJS);
    assert_eq!(
        ks.iter().filter(|&&k| k == TokenKind::Ident).count(),
        2,
        "U+200B must separate tokens: {ks:?}"
    );
    division("let a = 1;\u{200b}let b = 2 / 3;", ScriptJS);
}

#[test]
fn unicode_next_line_is_whitespace() {
    division("var a = 1;\u{85}var b = 2 / 3;", ScriptJS);
    let ks = kinds_of("var a = 1;\u{85}var b = 2;", ScriptJS);
    assert_eq!(
        ks.iter().filter(|&&k| k == TokenKind::KwVar).count(),
        2,
        "U+0085 must separate tokens: {ks:?}"
    );
}

#[test]
fn jsx_self_close_allows_whitespace() {
    let jsx = |code: &str| kinds_of(code, ScriptJSX);
    for code in [
        "const a = <N x=\"v\"/>;\nconst b = 1;",
        "const a = <N x=\"v\" / >;\nconst b = 1;",
        "const a = <N x=\"v\" /\n>;\nconst b = 1;",
        "const a = <N x=\"v\"\t/\t>;\nconst b = 1;",
    ] {
        let ks = jsx(code);
        assert!(ks.contains(&TokenKind::JsxTagEnd), "{code:?} must self-close: kinds {ks:?}");
        assert!(
            ks.iter().filter(|&&k| k == TokenKind::KwConst).count() == 2,
            "{code:?} must not swallow the next statement: kinds {ks:?}"
        );
    }
    let ks = jsx("const a = <N x=\"v\" / y>;");
    assert!(!ks.contains(&TokenKind::JsxTagEnd), "lone slash: kinds {ks:?}");
}

pub(super) fn diag_codes_of(code: &str, file_type: FileType) -> Vec<DiagCode> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut lx = Lexer::new();
    lx.lex(&buf, n, file_type.options());
    lx.lanes.diags.iter().map(|d| d.code).collect()
}

pub(super) fn is_fused_gt(k: TokenKind) -> bool {
    matches!(k, TokenKind::RShift | TokenKind::URShift | TokenKind::RShiftEq | TokenKind::URShiftEq)
}

#[track_caller]
pub(super) fn gt_run_fused(code: &str) {
    let ks = kinds_of(code, ScriptTS);
    assert!(ks.iter().any(|k| is_fused_gt(*k)), "{code:?} must fuse the `>` run: {ks:?}");
}

#[track_caller]
pub(super) fn gt_run_split(code: &str) {
    let ks = kinds_of(code, ScriptTS);
    assert!(
        !ks.iter().any(|k| is_fused_gt(*k) || *k == TokenKind::Ge),
        "{code:?} must split the `>` run: {ks:?}"
    );
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
        let ks = kinds_of(code, ScriptTSX);
        assert!(!ks.iter().any(|k| is_fused_gt(*k)) || code.contains(">> "), "{code:?}: {ks:?}");
        assert!(!ks.contains(&TokenKind::LShift), "{code:?}: {ks:?}");
    }
    let ks = kinds_of("const d = <div<A<B>>>x</div>;", ScriptTSX);
    assert!(ks.contains(&TokenKind::JsxText), "{ks:?}");
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 3, "{ks:?}");
    let ks = kinds_of("const f = <Foo<A<B>> bar={a >> b} />;", ScriptTSX);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
    let ks = kinds_of("x = <Foo<A<B>>/>\ny = a >> (b);", ScriptTSX);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::RShift).count(), 1, "{ks:?}");
}

#[test]
fn tsx_element_type_arguments_with_generic_function_type() {
    let ks = kinds_of("<Component<<T>(v: T) => void> />", ScriptTSX);
    assert!(!ks.contains(&TokenKind::LShift), "{ks:?}");
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
    let ks = kinds_of("const a = <Box<(x: T) => Foo<T>> prop={1} />;", ScriptTSX);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::JsxLt).count(), 1, "{ks:?}");
}

fn names_of(code: &str, file_type: FileType) -> String {
    kinds_of(code, file_type)
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

fn spans_of(code: &str, file_type: FileType) -> Vec<(u32, u32)> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, file_type.options());
    let kinds = &lx.kinds()[..count];
    assert!(lx.spans.len() >= count);

    kinds
        .iter()
        .zip(&lx.spans)
        .filter(|(kind, _)| !kind.is_trivia())
        .map(|(_, &span)| (span.start, span.end))
        .collect()
}

#[track_caller]
pub(super) fn stream(code: &str, file_type: FileType, want: &str) {
    assert_eq!(names_of(code, file_type), want, "{code:?}");
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
        stream(code, ScriptJSX, want);
    }
    stream(
        "const a = <Foo<T> key=<Bar<U> x=\"1\"/> />;",
        ScriptTSX,
        "const IDENT = JSX_LT IDENT < IDENT > IDENT = JSX_LT IDENT < IDENT > IDENT = STRING / JSX_TAG_END / JSX_TAG_END ;",
    );
}

#[test]
fn jsx_child_element_name_after_trivia() {
    stream(
        "const a = <div>\n  x<br />\n  < br />\n  y\n</div>;",
        ScriptTSX,
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
        stream(code, ScriptJSX, want);
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
        stream(code, ScriptJSX, want);
    }
    assert_eq!(spans_of("x = <a--b/>;", ScriptJSX)[3], (5, 9));
    assert_eq!(spans_of("y = <a-/>;", ScriptJSX)[3], (5, 7));
    stream(
        "<Foo<-1> data-x=\"1\"/>;",
        ScriptTSX,
        "JSX_LT IDENT < - NUMBER > IDENT = STRING / JSX_TAG_END ;",
    );
}

#[test]
fn escaped_identifiers_are_identifiers_in_every_walk() {
    regex("let \\u{62}c\n/re/.x;", ScriptJS);
    regex("let \\u{62}c: T\n/re/.x;", ScriptTS);
    regex("declare const \\u{62}c: Set<T>\n/re/.x;", ScriptTS);
    regex("var a = 1, \\u{62}c\n/re/.x;", ScriptJS);
    regex("for (;;) { break \\u{6f}uter\n/re/.test(b); }", ScriptJS);
    regex("type \\u{41} = T\n/re/.exec(s);", ScriptTS);
    regex("declare function \\u{66}(): T\n/re/.exec(s);", ScriptTS);
    division("x = \\u{62}c\n/re/g.exec(s);", ScriptJS);
    division("\\u{62}c / 2;", ScriptJS);
    division("x.\\u{62}c / 2;", ScriptJS);
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
        stream(code, ScriptJS, want);
    }
    assert_eq!(spans_of("x = 010.5;", ScriptJS)[2], (4, 7));
    assert_eq!(spans_of("x = 010.5;", ScriptJS)[3], (7, 9));
}

#[test]
fn type_context_before_a_declaration_keyword() {
    gt_run_split("x = () => {}\n<Map<P>>baz;");
    gt_run_split("x = async () => {}\n<Map<P>>baz;");
    regex("declare function f(): Foo<T>\nclass C {}\n/re/.test(s);", ScriptTS);
    regex("let x: Foo<T>\nfunction f() {}\n/re/.test(s);", ScriptTS);
    division("x = <T>\nfunction(){} / 2;", ScriptTS);
    stream(
        "class C<T> extends B implements I, void {}\n/=/.test(s);",
        ScriptTS,
        "class IDENT < IDENT > extends IDENT implements IDENT , void { } REGEXP . IDENT ( IDENT ) ;",
    );
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
    let ks = kinds_of("x = [f].#p <\n{ _ }| 's'>= b;", ScriptTS);
    assert!(ks.contains(&TokenKind::Ge), "{ks:?}");
    regex("type Foo = | {} | bigint\n<import('m').baz<Bar<T<P>>>>/'/.x;", ScriptTS);
    regex("let x: T\n<U>/re/.test(s);", ScriptTS);
    division("let x = y\n<T>/re/.source;", ScriptTS);
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
        ScriptTS,
    );
    gt_run_fused("x = y satisfies A\n<Set<K>, Partial<U>>ab;");
    gt_run_split("x = y as A<B<C>> as D;");
    gt_run_fused("x = b()!\n<Array<Set<unknown>>>Foo;");
    gt_run_fused("x = c! < 0.5>>> a;");
    division("export default { a: 1 } / obj(x);", ScriptJS);
    regex("x = y as Pick | this<U<V<W>>>\n/'/.x;", ScriptTS);
    regex("x = y satisfies this<U<V<W>>>\n/'/.x;", ScriptTS);
    division("x = this<A<B>> / 2;", ScriptTS);
    regex("x ? a : [b][c]()\n{}\n/y/.exec(s);", ScriptJS);
    regex("f(class { accessor x = y })\n{ }\n/</.test(s);", ScriptJS);
    // `b<c<d>>` is not a type-argument list: its closing `>` is glued to another `>` and rescans as
    // `>>` (tsc's `reScanGreaterToken`), so every `<` compares and the run is `>>>` + `>`.
    let ks = kinds_of("x = a > b<c<d>>>>(e);", ScriptTS);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 2, "{ks:?}");
    assert!(ks.contains(&TokenKind::URShift), "{ks:?}");
}

#[test]
fn gt_runs_in_members_separated_only_by_line_breaks() {
    // Without a `;` or `,` between members, the line break after the run ends the member: the
    // name on the next line is another member, not the operand of a comparison, and a `<` there
    // opens a signature's type parameters.
    for code in [
        "declare class C {\n  a: A\n  b: B<C<void>>\n  constructor(r: R)\n}",
        "type T = {\n  a: A<'x', B<T, 'x'>>\n  b: A<'x', B<T, 'x'>>\n}",
        "declare class C {\n  then: A<B<C>>['then']\n  finally: A<B<C>>\n}",
        "let o: {\n  in: A<B<C>>\n  of: A<B<C>>\n} = y;",
        "declare class C {\n  a(): void\n  b<T = null>(o: O<T>): P<Q<T>>\n}",
        "interface I {\n  <T, V = W>(a: A, v?: V): P<Q<T>>\n  <T, V = W>(o: O<V>): P<Q<T>>\n}",
        "interface I {\n  <T>(a: A): B extends C ? D : () => E<T>\n  <M>(b: B): E<F<M>>\n}",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn gt_runs_after_arrows_mapped_types_and_import_types_inside_lists() {
    // An arrow in a conditional type's branch, a mapped type with an `as` clause, or an
    // `import("m")` chain inside a type-argument list does not end the list: the run after it
    // still closes it.
    for code in [
        "type P<T> = T extends U ? (a: A) => B<T> : (a: A) => B<C<T>>;",
        "type P<T> = R<T> extends Q<\n  infer U\n>\n  ? (...a: A<T>) => B<U>\n  : (...a: A<T>) => B<C<T>>;",
        "type T = A<B<C, D<E, { [K in keyof S as K extends F ? never : K]: S[K] }> & G<H>>>;",
        "declare const c: import(\"m\").A<B<import(\"m\").C<D<import(\"m\").E<F, G.H & G.I>>, J<import(\"m\").K, never>>>, \"ref\"> & import(\"m\").L<F>;",
        "declare const p: {\n  a: {\n    b: () => P<import(\"m\").Q<import(\"n\").R<import(\"o\").S>>>;\n  };\n};",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn gt_runs_closing_lists_that_open_in_any_context() {
    // The token after the run fails TypeScript's expression speculation (`{`, a name), but the
    // list can only be a list: a type parameter list `<T extends`, a name after a keyword only a
    // type follows, a return type, a dotted name in a heritage clause.
    for code in [
        "const f = <T extends A<B>>(x: T) => x;",
        "class C {\n  f = <T extends A<B>>(x: T) => x;\n}",
        "class C extends A<B<C>> {}",
        "class C extends a.b.A<B<C>> {}",
        "class C implements I<J<K>> {}",
        "interface I<A, B extends C<D>> {}",
        "class C {\n  m(): A<B<C>> {}\n  static n(): A<B<C>> {}\n  async o(): A<B<C>> {}\n}",
        "class C {\n  keyof(): A<B<C>> {}\n}",
        "function f(): A<B<C>> {}",
        "const f = function (): A<B<C>> {};",
        "const o = { m(): A<B<C>> {} };",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn relational_heads_that_look_like_type_references() {
    // The same tokens read as an expression: a `case` label, a property named `function`.
    for code in
        ["switch (v) {\n  case (x): a < b < c >> d;\n}", "c ? o.function(x) : a < b < c >> d;"]
    {
        gt_run_fused(code);
    }
}

#[test]
fn gt_runs_after_parameter_lists_inside_types() {
    // A run after a parameter list inside a type: rest and optional parameters, object types as
    // parameter types, and multi-line type parameter lists.
    for code in [
        "type F<T extends (...a: any) => any> = (...a: P<T>) => Q<R<T>>;",
        "declare class C {\n  m<T, V>(d: D, v?: X<V>): P<Q<T>>\n}",
        "type P = A<B<C<D.E<F, import(\"m\").G>>>, H<import(\"m\").I, never>>;",
        "declare class C {\n  m(r: R<T>, { a }?: {\n    a?: boolean;\n  }): P<void>;\n  n(o?: {\n    b: boolean;\n  }): P<Q<R, import(\"m\").S<T, U>>>;\n}",
        "interface I {\n  <\n    A extends (...a: any[]) => any,\n    B = C<A>\n  >(a: A): <D extends E<A>>(d?: D, ...r: F<B>) => G<D, H<A>>\n\n  <J, K = L<J>>(j: J): J\n}",
        "interface I {\n  <A>(a: A): B<A>\n\n  <C, D = E<C>>( // note\n    c: C\n  ): C\n}",
    ] {
        gt_run_split(code);
    }
}

#[test]
fn jsx_child_tag_after_comment_or_unicode_space() {
    // Trivia between `<` and `/`, or around a closing name, leaves the same tag.
    for (code, plain) in [
        ("x = <a>x</*c*//a>;", "x = <a>x</a>;"),
        ("x = <a>x<//c\n/a>;", "x = <a>x</a>;"),
        ("x = <a></*c*/b/></a>;", "x = <a><b/></a>;"),
        ("x = <a><//c\nb/></a>;", "x = <a><b/></a>;"),
        ("x = <a>x<\u{a0}/a>;", "x = <a>x</a>;"),
        ("x = <a>x<\u{2028}/a>;", "x = <a>x</a>;"),
        ("x = <a>x</\u{a0}a>;", "x = <a>x</a>;"),
        ("x = <a>x</a\u{a0}>;", "x = <a>x</a>;"),
        ("x = <a/\u{a0}>;", "x = <a/>;"),
        ("x = <a/\u{3000}>;", "x = <a/>;"),
        ("x = <a b=\"1\"/\u{feff}>;", "x = <a b=\"1\"/>;"),
        ("x = <a / /*c*/ >;", "x = <a/>;"),
        ("x = <>x<\u{a0}/>;", "x = <>x</>;"),
        ("x = <>x</\u{a0}>;", "x = <>x</>;"),
    ] {
        assert_eq!(kinds_of(code, ScriptJSX), kinds_of(plain, ScriptJSX), "{code:?}");
        assert!(diag_codes_of(code, ScriptJSX).is_empty(), "{code:?}");
    }
}

#[test]
fn jsx_member_tag_name_matches_across_trivia() {
    // A comment or Unicode whitespace before the `.` of a member name (`<A/*c*/.B>`) hides no
    // mismatch: the closing tag names the same element.
    for code in [
        "x = <A/*c*/.B>x</A.B>;",
        "x = <A\u{a0}.B>x</A.B>;",
        "x = <A\u{2029}.B>x</A.B>;",
        "x = <A /*c*/ .B>x</A.B>;",
        "x = <a/*c*/:b>x</a:b>;",
        "x = <A.B>x</A/*c*/.B>;",
    ] {
        assert!(diag_codes_of(code, ScriptTSX).is_empty(), "{code:?}");
    }
    for code in ["x = <A/*c*/.B>x</A.C>;", "x = <A\u{a0}.B>x</A>;"] {
        assert!(!diag_codes_of(code, ScriptTSX).is_empty(), "{code:?}");
    }
}

#[test]
fn tsx_template_type_in_expression_type_arguments_is_not_jsx() {
    // Asked from the JSX carve, the list `f<`${<T>(x: T) => T}`>` still has a raw template
    // tail; the speculation must read the literal whole, so the `<T>` inside is a function
    // type, not an unterminated element.
    for code in [
        "x = f<`${<T>(x: T) => T}`>(1);",
        "x = f<`a${<T>(x: T) => T}b`>(1);",
        "x = f<[`${<T>(x: T) => T}`]>(1);",
        "x = f<A | `${<T>(x: T) => T}`>(1);",
        "x = f<`${<T>(x: T) => T}${<U>(y: U) => U}`>(1);",
        "x = f<`${`${<T>(x: T) => T}`}`>(1);",
    ] {
        assert!(diag_codes_of(code, ScriptTSX).is_empty(), "{code:?}");
    }
    gt_run_split("x = f<A<`${<T>(x: T) => T}`>>(1);");
}

#[test]
fn bracket_bitmap_follows_the_token_starts() {
    // The bracket word is built while the string in it is still raw text.
    regex("if (a) /x/; if (s = \"(\") /re/.test(x);", ScriptJS);
    gt_run_split("x = (a) / 2; y = f<B<(a: \"((\") => void>>(1);");
}
