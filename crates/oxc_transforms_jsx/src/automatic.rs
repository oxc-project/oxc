use oxc_allocator::Allocator;
use oxc_allocator::{Box, Vec};
use oxc_ast::ast::{
    Argument, IdentifierName, IdentifierReference, JSXElementName, JSXFragment, JSXIdentifier,
    JSXMemberExpressionObject, StringLiteral,
};
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
            Expression::JSXFragment(jsx_fragment) => *expr = self.fold_jsx_fragment(jsx_fragment),
            _ => self.visit_expression_match(expr),
        }
    }
}

impl<'a, 'b> Automatic<'a> {
    fn fold_jsx_element(&mut self, jsx_element: &'b Box<JSXElement<'a>>) -> Expression<'a> {
        let name = self.jsx_name(&jsx_element.opening_element.name);

        let mut arguments = Vec::new_in(self.ast.allocator);
        arguments.push(Argument::Expression(name));

        self.ast.call_expression(
            jsx_element.span,
            self.ast.identifier_expression(IdentifierReference {
                span: Span::default(),
                name: Atom::from("jsx"),
            }),
            arguments,
            false,
            None,
        )
    }

    fn fold_jsx_fragment(&mut self, _jsx_fragment: &'b Box<JSXFragment<'a>>) -> Expression<'a> {
        todo!()
    }

    fn jsx_name(&self, name: &JSXElementName<'a>) -> Expression<'a> {
        match name {
            JSXElementName::Identifier(JSXIdentifier { span, name }) => {
                if *name == "this" {
                    return self.ast.this_expression(*span);
                }

                if name.starts_with(|c: char| c.is_ascii_lowercase()) {
                    return self.ast.literal_string_expression(StringLiteral {
                        span: *span,
                        value: name.clone(),
                    });
                }

                self.ast
                    .identifier_expression(IdentifierReference { span: *span, name: name.clone() })
            }
            JSXElementName::NamespacedName(jsx_namespaced_name) => {
                let value = format!(
                    "{}:{}",
                    jsx_namespaced_name.namespace.name, jsx_namespaced_name.property.name
                );

                self.ast.literal_string_expression(StringLiteral {
                    span: jsx_namespaced_name.span,
                    value: value.into(),
                })
            }
            JSXElementName::MemberExpression(member_expression) => {
                self.ast.static_member_expression(
                    member_expression.span,
                    self.fold_jsx_member_expression_object(&member_expression.object),
                    self.fold_jsx_identifier(&member_expression.property),
                    false,
                )
            }
        }
    }

    fn fold_jsx_member_expression_object(
        &self,
        obj: &JSXMemberExpressionObject<'a>,
    ) -> Expression<'a> {
        match obj {
            JSXMemberExpressionObject::Identifier(JSXIdentifier { span, name }) => {
                if *name == "this" {
                    return self.ast.this_expression(*span);
                }

                self.ast
                    .identifier_expression(IdentifierReference { span: *span, name: name.clone() })
            }
            JSXMemberExpressionObject::MemberExpression(member_expression) => {
                self.ast.static_member_expression(
                    member_expression.span,
                    self.fold_jsx_member_expression_object(&member_expression.object),
                    self.fold_jsx_identifier(&member_expression.property),
                    false,
                )
            }
        }
    }

    fn fold_jsx_identifier(&self, jsx_identifier: &JSXIdentifier) -> IdentifierName {
        IdentifierName { span: jsx_identifier.span, name: jsx_identifier.name.clone() }
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
