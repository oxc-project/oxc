//! `CodegenOptions::ascii_only`: every emitted byte is ASCII and the program means the same.

use oxc_codegen::CodegenOptions;

use crate::tester::{default_options, test_options};

fn ascii() -> CodegenOptions {
    CodegenOptions { ascii_only: true, ..default_options() }
}

fn ascii_min() -> CodegenOptions {
    CodegenOptions { ascii_only: true, minify: true, ..CodegenOptions::default() }
}

#[track_caller]
fn t(source: &str, expected: &str) {
    assert!(expected.is_ascii(), "expected output must itself be ASCII: {expected:?}");
    test_options(source, expected, ascii());
}

#[track_caller]
fn t_min(source: &str, expected: &str) {
    assert!(expected.is_ascii(), "expected output must itself be ASCII: {expected:?}");
    test_options(source, expected, ascii_min());
}

#[test]
fn off_by_default() {
    test_options("let café = 'naïve';", "let café = \"naïve\";\n", default_options());
}

#[test]
fn string_literals() {
    t("let x = 'café';", "let x = \"caf\\u00E9\";\n");
    t("let x = '日本語';", "let x = \"\\u65E5\\u672C\\u8A9E\";\n");
    // Above the BMP: code point escape, as esbuild emits for ES2015+.
    t("let x = '😀';", "let x = \"\\u{1F600}\";\n");
    // Already-escaped input and LS/PS are unaffected.
    t("let x = '\\u00E9';", "let x = \"\\u00E9\";\n");
    t("let x = '\u{2028}';", "let x = \"\\u2028\";\n");
    // `</script` handling is preserved inside the same literal.
    t("let x = 'a</script>é';", "let x = \"a<\\/script>\\u00E9\";\n");
    // Directives keep their raw text, with non-ASCII characters escaped in place.
    t_min("'use strict'; 'caf\\é'; let x = 'é';", "\"use strict\";\"caf\\u00E9\";let x=`\\u00E9`;");
    // A directive containing a LineContinuation is not a Use Strict Directive and must not become one.
    t("'use\\\u{2028} strict';", "\"use\\\n strict\";\n");
}

#[test]
fn identifiers() {
    t("let café = 1; café++;", "let caf\\u00E9 = 1;\ncaf\\u00E9++;\n");
    // Above the BMP an identifier escape must be a code point escape, not a surrogate pair.
    t("let 𠮷 = 1;", "let \\u{20BB7} = 1;\n");
    t(
        "x.café; x = { café: 1, 'naïve': 2 };",
        "x.caf\\u00E9;\nx = {\n\tcaf\\u00E9: 1,\n\t\"na\\u00EFve\": 2\n};\n",
    );
    t(
        "class A { #ñ = 1; m() { return this.#ñ; } }",
        "class A {\n\t#\\u00F1 = 1;\n\tm() {\n\t\treturn this.#\\u00F1;\n\t}\n}\n",
    );
    t("ñ: for (;;) break ñ;", "\\u00F1: for (;;) break \\u00F1;\n");
    t("export { café as naïve } from 'x';", "export { caf\\u00E9 as na\\u00EFve } from \"x\";\n");
    // Destructuring assignment target printed via the non-shorthand path.
    t("({ ñame } = opts);", "({\\u00F1ame} = opts);\n");
}

#[test]
fn typescript() {
    t("let x: import('m').Тип;", "let x: import(\"m\").\\u0422\\u0438\\u043F;\n");
    t("let x: A.Б;", "let x: A.\\u0411;\n");
    t(
        "interface I { [ключ: string]: number }",
        "interface I {\n\t[\\u043A\\u043B\\u044E\\u0447: string]: number;\n}\n",
    );
    t("type T = `préfixe-${string}`;", "type T = `pr\\u00E9fixe-${string}`;\n");
    t("enum E { [`clé`] = 1 }", "enum E {\n\t[`cl\\u00E9`] = 1\n}\n");
}

#[test]
fn regular_expressions() {
    t("let r = /café/g;", "let r = /caf\\u00E9/g;\n");
    // In a regex an astral char is a surrogate pair (no `\u{}` without the `u` flag).
    t("let r = /😀+/u;", "let r = /\\uD83D\\uDE00+/u;\n");
    t("let r = /😀/;", "let r = /\\uD83D\\uDE00/;\n");
    // An identity-escaped non-ASCII char keeps its meaning: the backslash is consumed
    // (`/\é/` and `/\u00E9/` match the same thing; `/\\u00E9/` would not).
    t("let r = /[\\–\\—]/;", "let r = /[\\u2013\\u2014]/;\n");
    t("let r = /a\\\\é/;", "let r = /a\\\\\\u00E9/;\n");
}

#[test]
fn template_literals() {
    t("let x = `café ${y} naïve`;", "let x = `caf\\u00E9 ${y} na\\u00EFve`;\n");
    // NonEscapeCharacter: `\é` cooks to `é`; the backslash is consumed so the cooked value is kept.
    t("let x = `caf\\é`;", "let x = `caf\\u00E9`;\n");
    // LineContinuation with LS: still a LineContinuation (cooks to nothing), spelled with LF so
    // the characters around it keep lexing the same way (`$\\<LS>{` must not become `${`).
    t("let x = `a\\\u{2028}b`;", "let x = `a\\\nb`;\n");
    t("let x = `$\\\u{2028}{`;", "let x = `$\\\n{`;\n");
}

#[test]
fn tagged_template_is_not_escaped() {
    // Asserted separately because the expected output is intentionally not ASCII.
    test_options("let x = tag`café ${`naïve`}`;", "let x = tag`café ${`na\\u00EFve`}`;\n", ascii());
}

#[test]
fn jsx_is_not_escaped() {
    // JSX has no escape syntax: element names, attribute strings and text are printed as
    // written; only embedded JS expressions are escaped.
    test_options(
        "<Кнопка label='é' title={'ü'}>ø</Кнопка>;",
        "<Кнопка label=\"é\" title={\"\\u00FC\"}>ø</Кнопка>;\n",
        ascii(),
    );
}
