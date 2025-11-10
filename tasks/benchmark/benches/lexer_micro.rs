// Micro-benchmarks for lexer performance optimization
//
// These benchmarks target specific lexer operations to help identify
// optimization opportunities and measure improvements.

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_parser::lexer::{Kind, Lexer};
use oxc_span::SourceType;

fn bench_whitespace_skip(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("whitespace");

    // Test various whitespace patterns
    let patterns = [
        ("single_space", " ident"),
        ("multiple_spaces", "    ident"),
        ("tabs", "\t\t\tident"),
        ("mixed_ws", "  \t  \t  ident"),
        ("newlines", "\n\n\nident"),
        ("heavy_ws", "                                        ident"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_identifier_scan(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("identifier");

    // Test identifier scanning patterns
    let patterns = [
        ("short_ascii", "a"),
        ("medium_ascii", "identifier"),
        ("long_ascii", "thisIsAVeryLongIdentifierName"),
        ("with_digits", "var123abc"),
        ("with_underscores", "_private_variable_name_"),
        ("with_dollar", "$jquery_style_var$"),
        ("keyword_let", "let"),
        ("keyword_function", "function"),
        ("keyword_async", "async"),
        // Unicode identifiers (cold path)
        ("unicode_start", "αlpha"),
        ("unicode_mixed", "café"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_string_literals(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("string_literal");

    let patterns = [
        ("empty_double", r#""""#),
        ("empty_single", "''"),
        ("short_double", r#""hello""#),
        ("short_single", "'hello'"),
        ("medium_double", r#""this is a medium length string""#),
        ("long_double", r#""this is a very long string that goes on and on and on and contains lots of text""#),
        ("with_escape", r#""hello\nworld""#),
        ("many_escapes", r#""line1\nline2\nline3\nline4\nline5""#),
        ("unicode_escape", r#""unicode: \u0041""#),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_numeric_literals(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("numeric_literal");

    let patterns = [
        ("zero", "0"),
        ("single_digit", "7"),
        ("integer", "12345"),
        ("large_integer", "123456789012345"),
        ("float_simple", "3.14"),
        ("float_exp", "1.23e10"),
        ("float_neg_exp", "1.23e-10"),
        ("hex", "0xFF"),
        ("binary", "0b1010"),
        ("octal", "0o777"),
        ("bigint", "9007199254740991n"),
        ("separator", "1_000_000"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_template_literals(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("template_literal");

    let patterns = [
        ("empty", "``"),
        ("simple", "`hello`"),
        ("with_substitution", "`hello ${name}`"),
        ("multiple_substitutions", "`${a} ${b} ${c}`"),
        ("multiline", "`line1\nline2\nline3`"),
        ("with_escape", "`hello\\nworld`"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_comment_skip(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("comment");

    let patterns = [
        ("single_line_short", "// comment\n"),
        ("single_line_long", "// this is a very long comment that goes on and on\n"),
        ("multi_line_short", "/* comment */"),
        ("multi_line_long", "/* this is a very\nlong comment\nwith multiple lines */"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_punctuation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("punctuation");

    let patterns = [
        ("single_char", "( ) { } [ ] ; , . :"),
        ("double_char", "== != <= >= && ||"),
        ("triple_char", "=== !== ... >>> <<= >>="),
        ("arrows", "=> -> :: <=>"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_mixed_tokens(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("mixed");

    // Realistic code patterns
    let patterns = [
        ("variable_declaration", "const x = 42;"),
        ("function_call", "console.log('hello', 123);"),
        ("object_literal", "const obj = { a: 1, b: 2, c: 3 };"),
        ("array_literal", "const arr = [1, 2, 3, 4, 5];"),
        ("if_statement", "if (x > 0) { return true; }"),
        ("arrow_function", "const fn = (a, b) => a + b;"),
        ("class_method", "class Foo { method() { return 42; } }"),
    ];

    for (name, source) in patterns {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, SourceType::default());
                allocator.reset();
            });
        });
    }
    group.finish();
}

// Helper function to lex entire source
#[inline(always)]
fn lex_all<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Lexer<'a> {
    let mut lexer = Lexer::new_for_benchmarks(allocator, source_text, source_type);
    if lexer.first_token().kind() != Kind::Eof {
        while lexer.next_token_for_benchmarks().kind() != Kind::Eof {}
    }
    lexer
}

criterion_group!(
    lexer_micro,
    bench_whitespace_skip,
    bench_identifier_scan,
    bench_string_literals,
    bench_numeric_literals,
    bench_template_literals,
    bench_comment_skip,
    bench_punctuation,
    bench_mixed_tokens,
);
criterion_main!(lexer_micro);
