use std::path::{Path, PathBuf};

use oxc::{
    allocator::Allocator,
    parser::{Parser, ParserReturn},
    span::SourceType,
};
use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, Expand, FormatOptions,
    Formatter, IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteProperties, QuoteStyle,
    Semicolons, TrailingCommas, get_parse_options,
};

use crate::{
    babel::BabelCase,
    misc::MiscCase,
    suite::{Case, TestResult},
    test262::Test262Case,
    typescript::TypeScriptCase,
};

/// Different formatter options to test idempotency against
fn get_formatter_options_list() -> [FormatOptions; 3] {
    [
        FormatOptions::default(),
        // Opposite of default options
        FormatOptions {
            indent_style: IndentStyle::Tab,
            indent_width: IndentWidth::try_from(4).unwrap(),
            line_ending: LineEnding::Crlf,
            line_width: LineWidth::try_from(80).unwrap(),
            quote_style: QuoteStyle::Single,
            jsx_quote_style: QuoteStyle::Single,
            quote_properties: QuoteProperties::Consistent,
            trailing_commas: TrailingCommas::Es5,
            semicolons: Semicolons::AsNeeded,
            arrow_parentheses: ArrowParentheses::AsNeeded,
            bracket_spacing: BracketSpacing::from(false),
            bracket_same_line: BracketSameLine::from(false),
            attribute_position: AttributePosition::Multiline,
            expand: Expand::Never,
            ..Default::default()
        },
        // Different from above two options
        FormatOptions {
            indent_width: IndentWidth::try_from(8).unwrap(),
            line_width: LineWidth::try_from(120).unwrap(),
            line_ending: LineEnding::Lf,
            quote_properties: QuoteProperties::Preserve,
            trailing_commas: TrailingCommas::None,
            ..Default::default()
        },
    ]
}

/// Idempotency test
fn get_result(source_text: &str, source_type: SourceType) -> TestResult {
    let allocator = Allocator::default();
    let ParserReturn { program, errors, .. } =
        Parser::new(&allocator, source_text, source_type).with_options(get_parse_options()).parse();

    if !errors.is_empty() {
        // Skip test if input source code has parse errors
        return TestResult::Passed;
    }

    // Run the formatter with different options and compare the results.
    // Return `TestResult::Passed` if all results are the same.
    for options in get_formatter_options_list() {
        let source_text1 = Formatter::new(&allocator, options.clone()).build(&program);

        let allocator = Allocator::default();
        let ParserReturn { program, errors, .. } =
            Parser::new(&allocator, &source_text1, source_type)
                .with_options(get_parse_options())
                .parse();

        if !errors.is_empty() {
            return TestResult::ParseError(
                errors.iter().map(std::string::ToString::to_string).collect(),
                false,
            );
        }

        let source_text2 = Formatter::new(&allocator, options).build(&program);

        if source_text1 != source_text2 {
            return TestResult::Mismatch("Mismatch", source_text1, source_text2);
        }
    }

    TestResult::Passed
}

pub struct FormatterTest262Case {
    base: Test262Case,
}

impl Case for FormatterTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.should_fail() || self.base.skip_test_case()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let is_module = self.base.is_module();
        let source_type = SourceType::default().with_module(is_module);
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct FormatterBabelCase {
    base: BabelCase,
}

impl Case for FormatterBabelCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: BabelCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}

pub struct FormatterTypeScriptCase {
    base: TypeScriptCase,
}

impl Case for FormatterTypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: TypeScriptCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let units = self.base.units.clone();
        for unit in units {
            self.base.code.clone_from(&unit.content);
            let result = get_result(&unit.content, unit.source_type);
            if result != TestResult::Passed {
                self.base.result = result;
                return;
            }
        }
        self.base.result = TestResult::Passed;
    }
}

pub struct FormatterMiscCase {
    base: MiscCase,
}

impl Case for FormatterMiscCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: MiscCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        self.base.skip_test_case() || self.base.should_fail()
    }

    fn run(&mut self) {
        let source_text = self.base.code();
        let source_type = self.base.source_type();
        let result = get_result(source_text, source_type);
        self.base.set_result(result);
    }
}
