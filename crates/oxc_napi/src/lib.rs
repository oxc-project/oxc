mod comment;
mod error;

pub use comment::*;
pub use error::*;

use oxc_ast::{CommentKind, ast::Program};
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_syntax::module_record::ModuleRecord;

/// Convert spans to UTF-16
pub fn convert_utf8_to_utf16(
    source_text: &str,
    program: &mut Program,
    module_record: &mut ModuleRecord,
    errors: &mut Vec<OxcError>,
) -> Vec<Comment> {
    let span_converter = Utf8ToUtf16::new(source_text);
    span_converter.convert_program(program);

    // Convert comments
    let mut offset_converter = span_converter.converter();
    let comments = program
        .comments
        .iter()
        .map(|comment| {
            let value = comment.content_span().source_text(source_text).to_string();
            let mut span = comment.span;
            if let Some(converter) = offset_converter.as_mut() {
                converter.convert_span(&mut span);
            }
            Comment {
                r#type: match comment.kind {
                    CommentKind::Line => String::from("Line"),
                    CommentKind::Block => String::from("Block"),
                },
                value,
                start: span.start,
                end: span.end,
            }
        })
        .collect::<Vec<_>>();

    // Convert spans in module record to UTF-16
    span_converter.convert_module_record(module_record);

    // Convert spans in errors to UTF-16
    if let Some(mut converter) = span_converter.converter() {
        for error in errors {
            for label in &mut error.labels {
                converter.convert_offset(&mut label.start);
                converter.convert_offset(&mut label.end);
            }
        }
    }

    comments
}
