use std::borrow::Cow;

use cow_utils::CowUtils;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_parser::{
    Parser,
    lexer::{Kind, Lexer},
};
use oxc_span::SourceType;
use oxc_tasks_common::{TestFile, TestFiles};

fn bench_lexer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("lexer");

    // Lexer lacks awareness of JS grammar, so it gets confused by a few things without the parser
    // driving it. So simplify the input for it, by replacing these syntaxes with plain strings.
    // This ensures lexing completes without generating any errors, which is more realistic.
    //
    // It's unfortunate that this benchmark doesn't exercise the code paths for these syntaxes,
    // but this is the closest we can get to a realistic benchmark of lexer in isolation.
    let mut allocator = Allocator::default();
    let files = TestFiles::minimal()
        .files()
        .iter()
        .map(|file| {
            let source_type = SourceType::from_path(&file.file_name).unwrap();

            let source_text = clean(&file.source_text, source_type, &allocator);
            allocator.reset();

            TestFile {
                url: file.url.clone(),
                file_name: file.file_name.clone(),
                source_text,
                source_type,
            }
        })
        .collect::<Vec<_>>();

    for file in files {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = file.source_type;
        group.bench_function(id, |b| {
            // Do not include initializing allocator in benchmark.
            // User code would likely reuse the same allocator over and over to parse multiple files,
            // so we do the same here.
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_whole_file(&allocator, source_text, source_type);
                allocator.reset();
            });
        });
    }
    group.finish();
}

criterion_group!(lexer, bench_lexer);
criterion_main!(lexer);

// `#[inline(always)]` to ensure this is inlined into benchmark.
// It's also used in `SourceCleaner` below.
#[expect(clippy::inline_always)]
#[inline(always)]
fn lex_whole_file<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Lexer<'a> {
    let mut lexer = Lexer::new_for_benchmarks(allocator, source_text, source_type);
    if lexer.first_token().kind() != Kind::Eof {
        // Use `next_token_for_benchmarks` instead of `next_token`, to work around problem
        // where `next_token` wasn't inlined here.
        // `next_token_for_benchmarks` is identical to `next_token`, but is marked `#[inline(always)]`.
        while lexer.next_token_for_benchmarks().kind() != Kind::Eof {}
    }
    lexer
}

/// Clean source text.
///
/// Purpose is to allow lexer to complete without any errors.
/// Usually sources Oxc is asked to parse will not produce lexer errors, and generating diagnostics is
/// fairly expensive, so is unrealistic for benchmarking purposes.
///
/// Certain syntax will parse without error, but the lexer alone does not have the context to understand
/// they're fine. Notably this includes syntax where the lexer only consumes the first character and
/// parser would then call back into lexer to complete the job.
///
/// So replace these syntaxes with strings so that lexer can complete without error:
/// * `RegExpLiteral`
/// * `TemplateLiteral`
/// * `JSXText`
fn clean<'a>(source_text: &'a str, source_type: SourceType, allocator: &'a Allocator) -> String {
    // Parse
    let parser_ret = Parser::new(allocator, source_text, source_type).parse();
    assert!(parser_ret.errors.is_empty());
    let program = parser_ret.program;

    // Visit AST and compile list of replacements
    let mut cleaner = SourceCleaner::new(source_text);
    cleaner.visit_program(&program);
    let mut replacements = cleaner.replacements;

    // Make replacements
    replacements.sort_unstable_by_key(|replacement| replacement.span);

    let mut clean_source_text = String::with_capacity(cleaner.clean_source_text_len);
    let mut last_index = 0;
    for Replacement { span, text } in replacements {
        clean_source_text.push_str(&source_text[last_index..span.start as usize]);
        clean_source_text.push_str(&text);
        last_index = span.end as usize;
    }
    clean_source_text.push_str(&source_text[last_index..]);

    // Check lexer can lex it without any errors
    let lexer = lex_whole_file(allocator, &clean_source_text, source_type);
    assert!(lexer.errors().is_empty());

    clean_source_text
}

struct SourceCleaner<'a> {
    source_text: &'a str,
    replacements: Vec<Replacement>,
    clean_source_text_len: usize,
}

struct Replacement {
    span: Span,
    text: String,
}

impl<'a> SourceCleaner<'a> {
    fn new(source_text: &'a str) -> Self {
        Self { source_text, replacements: vec![], clean_source_text_len: source_text.len() }
    }

    fn replace(&mut self, span: Span, text: String) {
        self.clean_source_text_len += text.len() - span.size() as usize;
        self.replacements.push(Replacement { span, text });
    }
}

impl<'a> Visit<'a> for SourceCleaner<'a> {
    fn visit_reg_exp_literal(&mut self, regexp: &RegExpLiteral<'a>) {
        let pattern_text = regexp.regex.pattern.text.as_str();
        let span = Span::sized(regexp.span.start, u32::try_from(pattern_text.len()).unwrap() + 2);
        let text = convert_to_string(pattern_text);
        self.replace(span, text);
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        let span = lit.span;
        let text = span.shrink(1).source_text(self.source_text);
        let text = convert_to_string(&text.cow_replace('\n', " "));
        self.replace(span, text);
    }

    fn visit_jsx_text(&mut self, jsx_text: &JSXText<'a>) {
        let span = jsx_text.span;
        let text = span.source_text(self.source_text);
        let text = convert_to_string(&text.cow_replace('\n', " "));
        self.replace(span, text);
    }
}

#[expect(clippy::naive_bytecount)]
fn convert_to_string(text: &str) -> String {
    let single_quote_count = text.as_bytes().iter().filter(|&&b| b == b'\'').count();
    let double_quote_count = text.as_bytes().iter().filter(|&&b| b == b'"').count();

    let (quote, other_quote) =
        if single_quote_count <= double_quote_count { ('\'', "\"") } else { ('"', "'") };

    #[expect(clippy::disallowed_methods)]
    let text = if single_quote_count == 0 || double_quote_count == 0 {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(text.replace(quote, other_quote))
    };

    format!("{quote}{text}{quote}")
}
