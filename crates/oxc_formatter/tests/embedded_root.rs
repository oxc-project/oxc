use oxc_allocator::Allocator;
use oxc_formatter::{ExpressionLayout, ExpressionRoot, JsEmbeddedIn, JsFormatOptions, JsRoot};
use oxc_formatter_core::{Document, FormatSession, InputKind, PrintWidth, PrinterOptions};
use oxc_span::SourceType;

fn format_expression(code: &str, without_parent: bool) -> String {
    let allocator = Allocator::default();
    let session = FormatSession::new(&allocator, InputKind::PhysicalFile);
    let root = JsRoot::Expression(ExpressionRoot {
        layout: ExpressionLayout::Bare,
        tailwind_classes: false,
        ends_at_double_brace: false,
        reads_as_statements: false,
        in_parens: false,
        without_parent,
    });
    let ir = oxc_formatter::format_to_ir(
        &session,
        code,
        SourceType::mjs(),
        JsFormatOptions::default(),
        root,
        JsEmbeddedIn::default(),
    )
    .unwrap();
    let options = PrinterOptions::default().with_print_width(PrintWidth::new(20));
    Document::new(ir.ir, Vec::new()).print(code.len(), options).unwrap().into_code()
}

#[test]
fn a_root_without_parent_indents_a_broken_binary_chain() {
    let code = "aaaaaaaaaa || bbbbbbbbbb || cccccccccc";
    assert_eq!(format_expression(code, false), "aaaaaaaaaa ||\nbbbbbbbbbb ||\ncccccccccc");
    assert_eq!(format_expression(code, true), "aaaaaaaaaa ||\n  bbbbbbbbbb ||\n  cccccccccc");
}
