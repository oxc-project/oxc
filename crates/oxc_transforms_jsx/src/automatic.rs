use oxc_allocator::Allocator;
use oxc_allocator::{Box, Vec};
use oxc_ast::ast::{
    Argument, BindingIdentifier, IdentifierName, IdentifierReference, ImportDeclarationSpecifier,
    ImportSpecifier, JSXChild, JSXElementName, JSXExpression, JSXFragment, JSXIdentifier,
    JSXMemberExpressionObject, JSXText, ModuleDeclaration, ModuleDeclarationKind, ModuleExportName,
    Program, Statement, StringLiteral,
};
use oxc_ast::Atom;
use oxc_ast::Span;
use oxc_ast::{
    ast::{Expression, JSXElement},
    visit_mut::VisitMut,
    AstBuilder,
};

pub struct AutomaticOptions {
    pub import_source: Atom,
    pub development: bool,
}

impl Default for AutomaticOptions {
    fn default() -> Self {
        Self { import_source: "react".into(), development: false }
    }
}

#[derive(Default)]
struct AutomaticState {
    import_jsx: Option<IdentifierReference>,
    import_jsxs: Option<IdentifierReference>,
}

pub struct Automatic<'a> {
    ast: AstBuilder<'a>,
    options: AutomaticOptions,
    state: AutomaticState,
}

impl<'a> Automatic<'a> {
    pub fn new(allocator: &'a Allocator, options: AutomaticOptions) -> Self {
        Self { ast: AstBuilder::new(allocator), options, state: Default::default() }
    }

    pub fn build<'b>(mut self, program: &'b mut Program<'a>) {
        self.visit_program(program);
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

    fn visit_statements(&mut self, stmts: &'b mut Vec<'a, Statement<'a>>) {
        stmts.iter_mut().for_each(|stmt| self.visit_statement(stmt));

        let mut specifiers = self.ast.new_vec();
        if let Some(import_jsx) = self.state.import_jsx.take() {
            specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier {
                span: Span::default(),
                imported: ModuleExportName::Identifier(IdentifierName {
                    span: Span::default(),
                    name: Atom::from("jsx"),
                }),
                local: Bridge::from_identifier_reference(import_jsx),
            }))
        }
        if let Some(import_jsxs) = self.state.import_jsxs.take() {
            specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(ImportSpecifier {
                span: Span::default(),
                imported: ModuleExportName::Identifier(IdentifierName {
                    span: Span::default(),
                    name: Atom::from("jsxs"),
                }),
                local: Bridge::from_identifier_reference(import_jsxs),
            }))
        }

        if !specifiers.is_empty() {
            let source = format!("{}/jsx-runtime", self.options.import_source);
            let source = StringLiteral { span: Span::default(), value: source.into() };
            let import_declaration = self.ast.import_declaration(specifiers, source, None, None);

            stmts.insert(
                0,
                Statement::ModuleDeclaration(self.ast.alloc(ModuleDeclaration {
                    span: Span::default(),
                    kind: ModuleDeclarationKind::ImportDeclaration(import_declaration),
                })),
            )
        }
    }
}

impl<'a, 'b> Automatic<'a> {
    fn fold_jsx_element(&mut self, jsx_element: &'b Box<JSXElement<'a>>) -> Expression<'a> {
        let name = self.fold_jsx_element_name(&jsx_element.opening_element.name);

        let mut arguments = self.ast.new_vec();
        arguments.push(Argument::Expression(name));

        let callee = if Utils::count_children(&jsx_element.children) > 1 {
            self.import_jsxs()
        } else {
            self.import_jsx()
        };

        self.ast.call_expression(
            jsx_element.span,
            self.ast.identifier_expression(callee),
            arguments,
            false,
            None,
        )
    }

    fn fold_jsx_fragment(&mut self, _jsx_fragment: &'b Box<JSXFragment<'a>>) -> Expression<'a> {
        todo!()
    }

    fn fold_jsx_element_name(&self, name: &JSXElementName<'a>) -> Expression<'a> {
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
                    Bridge::from_jsx_identifier(&member_expression.property),
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
                    Bridge::from_jsx_identifier(&member_expression.property),
                    false,
                )
            }
        }
    }
}

impl<'a, 'b> Automatic<'a> {
    fn import_jsx(&mut self) -> IdentifierReference {
        self.state
            .import_jsx
            .get_or_insert_with(|| IdentifierReference {
                span: Span::default(),
                name: Atom::from("_jsx"),
            })
            .clone()
    }

    fn import_jsxs(&mut self) -> IdentifierReference {
        self.state
            .import_jsxs
            .get_or_insert_with(|| IdentifierReference {
                span: Span::default(),
                name: Atom::from("_jsxs"),
            })
            .clone()
    }
}

struct Bridge;

impl Bridge {
    fn from_jsx_identifier(jsx_identifier: &JSXIdentifier) -> IdentifierName {
        IdentifierName { span: jsx_identifier.span, name: jsx_identifier.name.clone() }
    }

    fn from_identifier_reference(identifier_reference: IdentifierReference) -> BindingIdentifier {
        BindingIdentifier { span: identifier_reference.span, name: identifier_reference.name }
    }
}

struct Utils;

impl Utils {
    fn count_children(jsx_child: &[JSXChild]) -> usize {
        jsx_child
            .iter()
            .filter(|child| match child {
                JSXChild::Text(jsx_text) => !Self::jsx_text_to_str(jsx_text).is_empty(),
                JSXChild::Element(..) => true,
                JSXChild::Fragment(..) => true,
                JSXChild::ExpressionContainer(e) => match e.expression {
                    JSXExpression::Expression(..) => true,
                    JSXExpression::EmptyExpression(..) => false,
                },
                JSXChild::Spread(..) => true,
            })
            .count()
    }

    fn jsx_text_to_str(_jsx_text: &JSXText) -> Atom {
        // TODO:
        "".into()
    }
}

#[test]
fn run() {
    use oxc_allocator::Allocator;
    use oxc_ast::SourceType;
    use oxc_parser::Parser;
    use oxc_printer::{Printer, PrinterOptions};

    let allocator = Allocator::default();
    let source_text = "const a = <main><aside/><section></section></main>";
    let mut source_type = SourceType::default();
    let source_type = source_type.with_jsx(true);
    let ret = Parser::new(&allocator, source_text, *source_type).parse();
    let mut program = allocator.alloc(ret.program);

    let automatic = Automatic::new(&allocator, Default::default());
    automatic.build(&mut program);

    let printer = Printer::new(
        source_text.len(),
        PrinterOptions { minify_whitespace: false, indentation: 4 },
    );

    let result = printer.build(program);
    println!("{}", result);
}
