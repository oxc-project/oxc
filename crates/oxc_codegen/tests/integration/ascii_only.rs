//! ASCII escaping and exceptions for `CodegenOptions::ascii_only`.

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::tester::{default_options, test_options};

fn ascii() -> CodegenOptions {
    CodegenOptions { ascii_only: true, ..default_options() }
}

fn ascii_min() -> CodegenOptions {
    CodegenOptions { ascii_only: true, minify: true, ..CodegenOptions::default() }
}

#[track_caller]
fn test(source: &str, expected: &str) {
    assert!(expected.is_ascii(), "expected output must itself be ASCII: {expected:?}");
    test_options(source, expected, ascii());
}

#[track_caller]
fn test_minify(source: &str, expected: &str) {
    assert!(expected.is_ascii(), "expected output must itself be ASCII: {expected:?}");
    test_options(source, expected, ascii_min());
}

#[test]
fn off_by_default() {
    test_options("let café = 'naïve';", "let café = \"naïve\";\n", default_options());
}

#[test]
fn string_literals() {
    test("let x = 'café';", "let x = \"caf\\u00E9\";\n");
    test("let x = '日本語';", "let x = \"\\u65E5\\u672C\\u8A9E\";\n");
    // Above the BMP: ES2015 code point escape.
    test("let x = '😀';", "let x = \"\\u{1F600}\";\n");
    // Existing escapes and line separators retain their values.
    test("let x = '\\u00E9';", "let x = \"\\u00E9\";\n");
    test("let x = '\u{2028}';", "let x = \"\\u2028\";\n");
    // `</script` handling is preserved inside the same literal.
    test("let x = 'a</script>é';", "let x = \"a<\\/script>\\u00E9\";\n");
    // Directives keep their raw text, with non-ASCII characters escaped in place.
    test_minify(
        "'use strict'; 'caf\\é'; let x = 'é';",
        "\"use strict\";\"caf\\u00E9\";let x=`\\u00E9`;",
    );
    // A directive containing a LineContinuation is not a Use Strict Directive and must not become one.
    test("'use\\\u{2028} strict';", "\"use\\\n strict\";\n");
}

#[test]
fn lone_surrogates_and_replacement_characters() {
    for (value, escaped) in [
        (r"\uD800\uDBFF \uDC00\uDFFF", r"\ud800\udbff \udc00\udfff"),
        ("��", r"\uFFFD\uFFFD"),
        (r"�\uD800é�\uDC00�", r"\uFFFD\ud800\u00E9\uFFFD\udc00\uFFFD"),
        // A three-byte character sharing U+FFFD's lead byte is not a surrogate marker.
        (r"\uD800\uFEFF\uFFFF😀", r"\ud800\uFEFF\uFFFF\u{1F600}"),
    ] {
        test(&format!("let x = '{value}';"), &format!("let x = \"{escaped}\";\n"));
        test_minify(&format!("let x = '{value}';"), &format!("let x=`{escaped}`;"));
    }
}

#[test]
fn identifiers() {
    test("let café = 1; café++;", "let caf\\u00E9 = 1;\ncaf\\u00E9++;\n");
    // Above the BMP an identifier escape must be a code point escape, not a surrogate pair.
    test("let 𠮷 = 1;", "let \\u{20BB7} = 1;\n");
    test(
        "x.café; x = { café: 1, 'naïve': 2 };",
        "x.caf\\u00E9;\nx = {\n\tcaf\\u00E9: 1,\n\t\"na\\u00EFve\": 2\n};\n",
    );
    test(
        "class A { #ñ = 1; m() { return this.#ñ; } }",
        "class A {\n\t#\\u00F1 = 1;\n\tm() {\n\t\treturn this.#\\u00F1;\n\t}\n}\n",
    );
    test("ñ: for (;;) break ñ;", "\\u00F1: for (;;) break \\u00F1;\n");
    test(
        "export { café as naïve } from 'x';",
        "export { caf\\u00E9 as na\\u00EFve } from \"x\";\n",
    );
    // Unrenamed destructuring keeps its shorthand property.
    test("({ ñame } = opts);", "({\\u00F1ame} = opts);\n");
    test(
        "import x from 'm' with { тип: 'json' };",
        "import x from \"m\" with { \\u0442\\u0438\\u043F: \"json\" };\n",
    );
}

#[test]
fn renamed_destructuring_assignment() {
    let source = "let ñame; ({ ñame } = opts); ({ ñame = fallback } = opts);";
    for (name, escaped) in [("a", "a"), ("é", r"\u00E9")] {
        for minify in [false, true] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
            assert!(parsed.diagnostics.is_empty());
            let semantic = SemanticBuilder::new().build(&parsed.program);
            assert!(semantic.diagnostics.is_empty());
            let mut scoping = semantic.semantic.into_scoping();
            let symbol_id = scoping.get_binding(scoping.root_scope_id(), "ñame".into()).unwrap();
            scoping.set_symbol_name(symbol_id, name.into());
            let code = Codegen::new()
                .with_options(CodegenOptions { minify, ..ascii() })
                .with_scoping(Some(scoping))
                .build(&parsed.program)
                .code;
            let expected = if minify {
                format!(
                    "let {escaped};({{\\u00F1ame:{escaped}}}=opts);({{\\u00F1ame:{escaped}=fallback}}=opts);"
                )
            } else {
                format!(
                    "let {escaped};\n({{\\u00F1ame: {escaped}}} = opts);\n({{\\u00F1ame: {escaped} = fallback}} = opts);\n"
                )
            };
            assert_eq!(code, expected);
        }
    }
}

#[test]
fn minified_identifier_boundaries() {
    for (source, expected) in [
        ("for (𐀀 of xs) {}", r"for(\u{10000} of xs){}"),
        ("𐀀 in obj; 𐀀 instanceof C;", r"\u{10000} in obj;\u{10000} instanceof C;"),
        ("𐀀 as T; 𐀀 satisfies T;", r"\u{10000} as T;\u{10000} satisfies T;"),
        ("import { 𐀀 as x } from 'm';", r#"import{\u{10000} as x}from"m";"#),
        ("export { x as 𐀀 } from 'm';", r#"export{x as \u{10000}}from"m";"#),
        // Punctuation must end the escaped-identifier boundary state.
        ("𐀀; f(𐀀); class A {}", r"\u{10000};f(\u{10000});class A{}"),
    ] {
        test_minify(source, expected);
    }
}

#[test]
fn typescript() {
    test("let x: import('m').Тип;", "let x: import(\"m\").\\u0422\\u0438\\u043F;\n");
    test("let x: A.Б;", "let x: A.\\u0411;\n");
    test("let x: import('m').А.Б;", "let x: import(\"m\").\\u0410.\\u0411;\n");
    test(
        "interface I { [ключ: string]: number }",
        "interface I {\n\t[\\u043A\\u043B\\u044E\\u0447: string]: number;\n}\n",
    );
    test("type T = `préfixe-${string}`;", "type T = `pr\\u00E9fixe-${string}`;\n");
    test("enum E { [`clé`] = 1 }", "enum E {\n\t[`cl\\u00E9`] = 1\n}\n");
}

#[test]
fn regular_expressions() {
    test("let r = /café/g;", "let r = /caf\\u00E9/g;\n");
    // Regex patterns use surrogate pair escapes, which also work without the `u`/`v` flags.
    test("let r = /😀+/u;", "let r = /\\uD83D\\uDE00+/u;\n");
    test("let r = /😀/;", "let r = /\\uD83D\\uDE00/;\n");
    // An identity-escaped non-ASCII char keeps its meaning: the backslash is consumed
    // (`/\é/` and `/\u00E9/` match the same thing; `/\\u00E9/` would not).
    test("let r = /[\\–\\—]/;", "let r = /[\\u2013\\u2014]/;\n");
    test("let r = /a\\\\é/;", "let r = /a\\\\\\u00E9/;\n");
}

#[test]
fn template_literals() {
    test("let x = `café ${y} naïve`;", "let x = `caf\\u00E9 ${y} na\\u00EFve`;\n");
    // NonEscapeCharacter: `\é` cooks to `é`; the backslash is consumed so the cooked value is kept.
    test("let x = `caf\\é`;", "let x = `caf\\u00E9`;\n");
    // LineContinuation with LS: still a LineContinuation (cooks to nothing), spelled with LF so
    // the characters around it keep lexing the same way (`$\\<LS>{` must not become `${`).
    test("let x = `a\\\u{2028}b`;", "let x = `a\\\nb`;\n");
    test("let x = `$\\\u{2028}{`;", "let x = `$\\\n{`;\n");
    test("let x = `\\0\\\u{2029}1`;", "let x = `\\0\\\n1`;\n");
    test("let x = `a\\\\é`;", "let x = `a\\\\\\u00E9`;\n");
    // Exercise both the ASCII chunk before a Unicode escape and the final suffix.
    test("let x = `</script>é</script>`;", "let x = `<\\/script>\\u00E9<\\/script>`;\n");
    test_minify("let x = `</script>é</script>`;", "let x=`<\\/script>\\u00E9<\\/script>`;");
}

#[test]
fn tagged_template_is_not_escaped() {
    // Asserted separately because the expected output is intentionally not ASCII.
    test_options("let x = tag`café ${`naïve`}`;", "let x = tag`café ${`na\\u00EFve`}`;\n", ascii());
    let source = "tag`é${tag`ü${`ñ`}ö`}ø${`á`}å`; `é`;";
    test_options(source, "tag`é${tag`ü${`\\u00F1`}ö`}ø${`\\u00E1`}å`;\n`\\u00E9`;\n", ascii());
    test_options(source, "tag`é${tag`ü${`\\u00F1`}ö`}ø${`\\u00E1`}å`;`\\u00E9`;", ascii_min());
}

#[test]
fn jsx_is_not_escaped() {
    // JSX names, attribute strings and text are preserved; only embedded JS expressions
    // receive Unicode escapes.
    test_options(
        "<Кнопка label='é' title={'ü'}>ø</Кнопка>;",
        "<Кнопка label=\"é\" title={\"\\u00FC\"}>ø</Кнопка>;\n",
        ascii(),
    );
}

mod sourcemap {
    use super::*;

    type Position = (u32, u32);

    #[track_caller]
    fn test(source: &str, minify: bool, mappings: &[(Position, Position)]) {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty());
        let map = Codegen::new()
            .with_options(CodegenOptions { minify, ..ascii() })
            .build(&parsed.program)
            .map
            .expect("sourcemap should be generated");
        for &(src, dst) in mappings {
            assert!(
                map.get_tokens().any(|token| {
                    (token.get_src_line(), token.get_src_col()) == src
                        && (token.get_dst_line(), token.get_dst_col()) == dst
                }),
                "missing mapping {src:?} -> {dst:?} (minify={minify}) for {source:?}",
            );
        }
    }

    #[test]
    fn escapes_shift_generated_columns() {
        let source = "const café = '😀'; café(next);";
        // Source columns count the astral character as two UTF-16 code units.
        test(source, false, &[((0, 19), (1, 0)), ((0, 24), (1, 10))]);
        test(source, true, &[((0, 19), (0, 28)), ((0, 24), (0, 38))]);
    }

    #[test]
    fn template_separators_collapse_generated_lines() {
        for separator in ['\u{2028}', '\u{2029}'] {
            let source = format!("const s = `é{separator}${{next}}`; after();");
            test(&source, false, &[((1, 2), (0, 25)), ((1, 10), (1, 0))]);
            test(&source, true, &[((1, 2), (0, 23)), ((1, 10), (0, 30))]);
        }
    }
}
