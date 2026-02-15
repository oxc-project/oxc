use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::Span;
use oxc_syntax::module_record::ModuleRecord;
use vue_oxc_toolkit::VueOxcParser;

use crate::loader::JavaScriptSource;

pub struct LinterParseResult<'a> {
    pub program: Program<'a>,
    pub irregular_whitespaces: Box<[Span]>,
    pub module_record: ModuleRecord<'a>,
}

impl<'a> LinterParseResult<'a> {
    pub fn new(
        program: Program<'a>,
        irregular_whitespaces: Box<[Span]>,
        module_record: ModuleRecord<'a>,
    ) -> Self {
        Self { program, irregular_whitespaces, module_record }
    }
}

macro_rules! parse_options {
    () => {
        ParseOptions {
            parse_regular_expression: true,
            allow_return_outside_function: true,
            ..ParseOptions::default()
        }
    };
}

pub fn parse_javascript_source<'a>(
    allocator: &'a Allocator,
    source: JavaScriptSource<'a>,
) -> (Result<LinterParseResult<'a>, Vec<OxcDiagnostic>>, JavaScriptSource<'a>) {
    let ret = Parser::new(allocator, source.source_text, source.source_type)
        .with_options(parse_options!())
        .parse();

    if !ret.errors.is_empty() {
        return (Err(if ret.is_flow_language { vec![] } else { ret.errors }), source);
    }

    (Ok(LinterParseResult::new(ret.program, ret.irregular_whitespaces, ret.module_record)), source)
}

pub fn parse_vue_source<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
) -> Vec<(Result<LinterParseResult<'a>, Vec<OxcDiagnostic>>, JavaScriptSource<'a>)> {
    let ret = VueOxcParser::new(allocator, source_text).with_options(parse_options!()).parse();
    let source = JavaScriptSource::new(source_text, ret.program.source_type);

    if !ret.errors.is_empty() {
        return vec![(Err(ret.errors), source)];
    }

    vec![(
        Ok(LinterParseResult::new(ret.program, ret.irregular_whitespaces, ret.module_record)),
        source,
    )]
}
