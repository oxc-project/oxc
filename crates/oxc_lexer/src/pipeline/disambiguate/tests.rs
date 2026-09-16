use crate::{LexOptions, Lexer, PAD, token::TokenKind};

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

pub(super) fn diag_codes_of(code: &str, file_type: FileType) -> Vec<u16> {
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
    let ks = kinds_of("x = a > b<c<d>>>>(e);", ScriptTS);
    assert_eq!(ks.iter().filter(|k| **k == TokenKind::Gt).count(), 3, "{ks:?}");
    assert!(ks.contains(&TokenKind::RShift), "{ks:?}");
}
