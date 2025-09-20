use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter, get_supported_source_type};
use oxc_parser::{ParseOptions, Parser};
use tower_lsp_server::{
    UriExt,
    lsp_types::{Position, Range, TextEdit, Uri},
};

use crate::LSP_MAX_INT;

pub struct ServerFormatter;

impl ServerFormatter {
    pub fn new() -> Self {
        Self {}
    }

    #[expect(clippy::unused_self)]
    pub fn run_single(&self, uri: &Uri, content: Option<String>) -> Option<Vec<TextEdit>> {
        let path = uri.to_file_path()?;
        let source_type = get_supported_source_type(&path)?;
        let source_text = if let Some(content) = content {
            content
        } else {
            std::fs::read_to_string(&path).ok()?
        };

        let allocator = Allocator::new();
        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(ParseOptions {
                parse_regular_expression: false,
                // Enable all syntax features
                allow_v8_intrinsics: true,
                allow_return_outside_function: true,
                // `oxc_formatter` expects this to be false
                preserve_parens: false,
            })
            .parse();

        if !ret.errors.is_empty() {
            return None;
        }

        let options = FormatOptions::default();
        let code = Formatter::new(&allocator, options).build(&ret.program);

        // nothing has changed
        if code == source_text {
            return Some(vec![]);
        }

        Some(vec![TextEdit::new(
            Range::new(Position::new(0, 0), Position::new(LSP_MAX_INT, 0)),
            code,
        )])
    }
}
