use oxc_allocator::Allocator;
use oxc_allocator::{Box, Vec};
use oxc_ast::ast::IdentifierReference;
use oxc_ast::Atom;
use oxc_ast::Span;
use oxc_ast::{
    ast::{Expression, JSXElement},
    visit_mut::VisitMut,
    AstBuilder,
};

pub struct Automatic<'a> {
    pub import_source: Atom,
    pub development: bool,
    ast: AstBuilder<'a>,
}

impl<'a> Automatic<'a> {
    pub fn new(
        allocator: &'a Allocator,
        import_source: Option<Atom>,
        development: Option<bool>,
    ) -> Self {
        Self {
            import_source: import_source.unwrap_or_else(|| Atom::from("React")),
            development: development.unwrap_or_default(),
            ast: AstBuilder::new(allocator),
        }
    }
}

impl<'a, 'b> VisitMut<'a, 'b> for Automatic<'a> {
    fn visit_expression(&mut self, expr: &'b mut Expression<'a>) {
        match expr {
            Expression::JSXElement(jsx_element) => *expr = self.fold_jsx_element(jsx_element),
            Expression::JSXFragment(_) => todo!(),
            _ => self.visit_expression_match(expr),
        }
    }
}

impl<'a, 'b> Automatic<'a> {
    fn fold_jsx_element(&mut self, jsx_element: &'b Box<JSXElement<'a>>) -> Expression<'a> {
        self.ast.call_expression(
            jsx_element.span,
            self.ast.identifier_expression(IdentifierReference {
                span: Span::default(),
                name: Atom::from("jsx"),
            }),
            Vec::new_in(self.ast.allocator),
            false,
            None,
        )
    }
}

#[test]
fn run() {
    use oxc_allocator::Allocator;
    use oxc_ast::SourceType;
    use oxc_parser::Parser;
    use oxc_printer::{Printer, PrinterOptions};

    let allocator = Allocator::default();
    let source_text = "const a = <div />";
    let mut source_type = SourceType::default();
    let source_type = source_type.with_jsx(true);
    let ret = Parser::new(&allocator, source_text, *source_type).parse();
    let mut program = allocator.alloc(ret.program);

    let mut automatic = Automatic::new(&allocator, None, None);
    automatic.visit_program(&mut program);

    let printer = Printer::new(
        source_text.len(),
        PrinterOptions { minify_whitespace: false, indentation: 4 },
    );

    let result = printer.build(program);
    println!("{}", result);
}
