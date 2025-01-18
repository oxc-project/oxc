#![allow(clippy::disallowed_methods)]
use oxc_allocator::Allocator;
use oxc_ast::{ast::*, Visit};
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::{
    lexer::{Kind, Lexer},
    Parser,
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
    let files = TestFiles::complicated()
        .files()
        .iter()
        .map(|file| {
            let source_type = SourceType::from_path(&file.file_name).unwrap();

            let mut cleaner = SourceCleaner::new(&file.source_text);
            cleaner.clean(source_type, &allocator);
            let source_text = cleaner.source_text;

            allocator.reset();

            TestFile { url: file.url.clone(), file_name: file.file_name.clone(), source_text }
        })
        .collect::<Vec<_>>();

    for file in files {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_function(id, |b| {
            // Do not include initializing allocator in benchmark.
            // User code would likely reuse the same allocator over and over to parse multiple files,
            // so we do the same here.
            let mut allocator = Allocator::default();
            b.iter(|| {
                let mut lexer = Lexer::new_for_benchmarks(&allocator, source_text, source_type);
                while lexer.next_token().kind != Kind::Eof {}
                allocator.reset();
            });
        });
    }
    group.finish();
}

criterion_group!(lexer, bench_lexer);
criterion_main!(lexer);

/// Cleaner of source text.
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
struct SourceCleaner {
    source_text: String,
    replacements: Vec<Replacement>,
}

struct Replacement {
    span: Span,
    text: String,
}

impl SourceCleaner {
    fn new(source_text: &str) -> Self {
        Self { source_text: source_text.to_string(), replacements: vec![] }
    }

    fn clean(&mut self, source_type: SourceType, allocator: &Allocator) {
        // Parse
        let source_text = self.source_text.clone();
        let parser_ret = Parser::new(allocator, &source_text, source_type).parse();
        assert!(parser_ret.errors.is_empty());
        let program = parser_ret.program;

        // Visit AST and compile list of replacements
        self.visit_program(&program);

        // Make replacements
        self.replacements.sort_unstable_by_key(|replacement| replacement.span);

        for replacement in self.replacements.iter().rev() {
            let span = replacement.span;
            self.source_text
                .replace_range(span.start as usize..span.end as usize, &replacement.text);
        }

        // Check lexer can lex it without any errors
        let mut lexer = Lexer::new_for_benchmarks(allocator, &self.source_text, source_type);
        while lexer.next_token().kind != Kind::Eof {}
        assert!(lexer.errors().is_empty());
    }

    fn replace(&mut self, span: Span, text: String) {
        self.replacements.push(Replacement { span, text });
    }
}

impl<'a> Visit<'a> for SourceCleaner {
    fn visit_reg_exp_literal(&mut self, regexp: &RegExpLiteral<'a>) {
        let RegExpPattern::Raw(pattern) = regexp.regex.pattern else { unreachable!() };
        let span = Span::sized(regexp.span.start, u32::try_from(pattern.len()).unwrap() + 2);
        let text = convert_to_string(pattern);
        self.replace(span, text);
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        let span = lit.span;
        let text = span.shrink(1).source_text(&self.source_text);
        let text = convert_to_string(text).replace('\n', " ");
        self.replace(span, text);
    }

    fn visit_jsx_text(&mut self, jsx_text: &JSXText<'a>) {
        let span = jsx_text.span;
        let text = span.source_text(&self.source_text);
        let text = convert_to_string(text).replace('\n', " ");
        self.replace(span, text);
    }
}

#[expect(clippy::naive_bytecount)]
fn convert_to_string(text: &str) -> String {
    let single_quote_count = text.as_bytes().iter().filter(|&&b| b == b'\'').count();
    let double_quote_count = text.as_bytes().iter().filter(|&&b| b == b'"').count();

    let (quote, other_quote) =
        if single_quote_count <= double_quote_count { ('\'', "\"") } else { ('"', "'") };
    let text = text.replace(quote, other_quote);
    format!("{quote}{text}{quote}")
}
